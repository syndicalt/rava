use std::collections::BTreeMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn serve_verify_accepts_valid_action_request_over_http() -> Result<(), Box<dyn Error>> {
    let body = accepted_request_body()?;
    let response = run_server_request(&body)?;

    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    let response_body = response_body(&response)?;
    let json: serde_json::Value = serde_json::from_str(response_body)?;
    assert_eq!(
        json.get("accepted").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(json.get("rejection"), Some(&serde_json::Value::Null));
    assert_eq!(
        json.get("service").and_then(serde_json::Value::as_str),
        Some("rava-verifier-preview-v0")
    );
    Ok(())
}

#[test]
fn serve_verify_returns_rejection_code_for_denied_action() -> Result<(), Box<dyn Error>> {
    let mut body = accepted_request_body()?;
    body["action"]["constraints"]["amount_usd"]["integer"] = serde_json::json!(900);

    let response = run_server_request(&body)?;

    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    let response_body = response_body(&response)?;
    let json: serde_json::Value = serde_json::from_str(response_body)?;
    assert_eq!(
        json.get("accepted").and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert_eq!(
        json.pointer("/rejection/code")
            .and_then(serde_json::Value::as_str),
        Some("action_signature_invalid")
    );
    assert_eq!(
        json.pointer("/rejection/subject"),
        Some(&serde_json::Value::Null)
    );
    Ok(())
}

#[test]
fn serve_verify_exposes_health_endpoint_with_limits() -> Result<(), Box<dyn Error>> {
    let response = run_server_raw_request(
        &["--max-request-bytes", "4096"],
        "GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )?;

    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    let response_body = response_body(&response)?;
    let json: serde_json::Value = serde_json::from_str(response_body)?;
    assert_eq!(
        json.get("service").and_then(serde_json::Value::as_str),
        Some("rava-verifier-preview-v0")
    );
    assert_eq!(
        json.get("status").and_then(serde_json::Value::as_str),
        Some("ok")
    );
    assert_eq!(
        json.get("max_request_bytes")
            .and_then(serde_json::Value::as_u64),
        Some(4096)
    );
    Ok(())
}

#[test]
fn serve_verify_rejects_request_bodies_over_configured_limit() -> Result<(), Box<dyn Error>> {
    let request = format!(
        "POST /verify/action HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        4096
    );

    let response = run_server_raw_request(&["--max-request-bytes", "128"], &request)?;

    assert!(
        response.starts_with("HTTP/1.1 413 Payload Too Large"),
        "{response}"
    );
    let response_body = response_body(&response)?;
    let json: serde_json::Value = serde_json::from_str(response_body)?;
    assert_eq!(
        json.get("error").and_then(serde_json::Value::as_str),
        Some("request body exceeds max_request_bytes")
    );
    Ok(())
}

#[test]
fn serve_verify_rejects_oversized_request_headers() -> Result<(), Box<dyn Error>> {
    let request = format!(
        "GET /healthz HTTP/1.1\r\nHost: localhost\r\nX-Oversized: {}",
        "a".repeat(16 * 1024)
    );

    let response = run_server_raw_request(&[], &request)?;

    assert!(
        response.starts_with("HTTP/1.1 431 Request Header Fields Too Large"),
        "{response}"
    );
    let response_body = response_body(&response)?;
    let json: serde_json::Value = serde_json::from_str(response_body)?;
    assert_eq!(
        json.get("error").and_then(serde_json::Value::as_str),
        Some("request headers exceed max header bytes")
    );
    Ok(())
}

#[test]
fn serve_verify_with_replay_store_records_accepted_action_and_rejects_replay(
) -> Result<(), Box<dyn Error>> {
    let replay_store = temp_file_path("rava-serve-replay");
    let replay_store_arg = replay_store.to_string_lossy().into_owned();
    let body = accepted_request_body()?;
    let action_id = string_field(&body["action"], "id")?.to_owned();

    let responses = run_server_requests_with_args(
        &["--replay-store", &replay_store_arg],
        &[body.clone(), body],
    )?;

    assert_accepted_response(&responses[0])?;
    assert_rejection_code(&responses[1], "action_replayed")?;
    let replay_json: serde_json::Value = serde_json::from_slice(&std::fs::read(&replay_store)?)?;
    assert_eq!(
        replay_json
            .get("action_ids")
            .and_then(serde_json::Value::as_array)
            .map(|ids| ids.iter().any(|id| id.as_str() == Some(action_id.as_str()))),
        Some(true)
    );

    std::fs::remove_file(replay_store)?;
    Ok(())
}

#[test]
fn serve_verify_with_revocation_store_rejects_revoked_capability() -> Result<(), Box<dyn Error>> {
    let revocation_store = temp_file_path("rava-serve-revocations");
    let revocation_store_arg = revocation_store.to_string_lossy().into_owned();
    let body = accepted_request_body()?;
    let capability_id = body["capability_chain"]
        .as_array()
        .and_then(|capabilities| capabilities.last())
        .and_then(|capability| capability.get("id"))
        .and_then(serde_json::Value::as_str)
        .ok_or("missing final capability id")?
        .to_owned();
    std::fs::write(
        &revocation_store,
        serde_json::to_vec_pretty(&serde_json::json!({
            "revoked_ids": [capability_id]
        }))?,
    )?;

    let response =
        run_server_request_with_args(&["--revocation-store", &revocation_store_arg], &body)?;

    assert_rejection_code(&response, "capability_revoked")?;
    std::fs::remove_file(revocation_store)?;
    Ok(())
}

fn accepted_request_body() -> Result<serde_json::Value, Box<dyn Error>> {
    let vector = repository_root().join("test-vectors/v0/flight-booking");
    let action: serde_json::Value =
        serde_json::from_slice(&std::fs::read(vector.join("action.json"))?)?;
    let capability_chain: serde_json::Value =
        serde_json::from_slice(&std::fs::read(vector.join("capability-chain.json"))?)?;
    let keys: serde_json::Value =
        serde_json::from_slice(&std::fs::read(vector.join("keys.json"))?)?;
    let actor_key = string_field(&keys, "actor_public_key_hex")?;
    let issuer_keys = keys
        .get("issuer_public_keys")
        .and_then(serde_json::Value::as_object)
        .ok_or("missing issuer_public_keys")?
        .iter()
        .map(|(issuer, key)| {
            key.as_str()
                .map(|public_key| (issuer.clone(), public_key.to_owned()))
                .ok_or_else(|| format!("issuer key for {issuer} is not a string"))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    Ok(serde_json::json!({
        "action": action,
        "capability_chain": capability_chain,
        "actor_public_key_hex": actor_key,
        "issuer_public_keys": issuer_keys,
        "now_unix": 1650000000
    }))
}

fn run_server_request(body: &serde_json::Value) -> Result<String, Box<dyn Error>> {
    run_server_request_with_args(&[], body)
}

fn run_server_request_with_args(
    extra_args: &[&str],
    body: &serde_json::Value,
) -> Result<String, Box<dyn Error>> {
    let responses = run_server_requests_with_args(extra_args, std::slice::from_ref(body))?;
    responses
        .into_iter()
        .next()
        .ok_or_else(|| "missing server response".into())
}

fn run_server_requests_with_args(
    extra_args: &[&str],
    bodies: &[serde_json::Value],
) -> Result<Vec<String>, Box<dyn Error>> {
    let address = free_loopback_address()?;
    let mut args = vec!["serve", "verify", "--addr", &address];
    args.extend_from_slice(extra_args);
    let mut server = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args(args)
        .spawn()?;

    let mut responses = Vec::new();
    for body in bodies {
        match post_json_when_ready(&address, "/verify/action", body) {
            Ok(response) => responses.push(response),
            Err(error) => {
                terminate(&mut server);
                return Err(error);
            }
        }
    }
    terminate(&mut server);
    Ok(responses)
}

fn run_server_raw_request(extra_args: &[&str], request: &str) -> Result<String, Box<dyn Error>> {
    let address = free_loopback_address()?;
    let mut args = vec!["serve", "verify", "--addr", &address];
    args.extend_from_slice(extra_args);
    let mut server = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args(args)
        .spawn()?;

    let response = match raw_request_when_ready(&address, request) {
        Ok(response) => response,
        Err(error) => {
            terminate(&mut server);
            return Err(error);
        }
    };
    terminate(&mut server);
    Ok(response)
}

fn free_loopback_address() -> Result<String, Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.to_string())
}

fn string_field<'a>(value: &'a serde_json::Value, key: &str) -> Result<&'a str, Box<dyn Error>> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("missing string field {key:?}").into())
}

fn post_json_when_ready(
    address: &str,
    path: &str,
    body: &serde_json::Value,
) -> Result<String, Box<dyn Error>> {
    let started = Instant::now();
    let body = serde_json::to_string(body)?;
    loop {
        match TcpStream::connect(address) {
            Ok(mut stream) => {
                let request = format!(
                    "POST {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(request.as_bytes())?;
                let mut response = String::new();
                stream.read_to_string(&mut response)?;
                return Ok(response);
            }
            Err(error) if started.elapsed() < Duration::from_secs(5) => {
                let _ = error;
                thread::sleep(Duration::from_millis(25));
            }
            Err(error) => return Err(error.into()),
        }
    }
}

fn raw_request_when_ready(address: &str, request: &str) -> Result<String, Box<dyn Error>> {
    let started = Instant::now();
    loop {
        match TcpStream::connect(address) {
            Ok(mut stream) => {
                match stream.write_all(request.as_bytes()) {
                    Ok(()) => {}
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset
                        ) => {}
                    Err(error) => return Err(error.into()),
                }
                let mut response = String::new();
                stream.read_to_string(&mut response)?;
                return Ok(response);
            }
            Err(error) if started.elapsed() < Duration::from_secs(5) => {
                let _ = error;
                thread::sleep(Duration::from_millis(25));
            }
            Err(error) => return Err(error.into()),
        }
    }
}

fn response_body(response: &str) -> Result<&str, Box<dyn Error>> {
    response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .ok_or_else(|| "missing HTTP response body".into())
}

fn assert_accepted_response(response: &str) -> Result<(), Box<dyn Error>> {
    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    let response_body = response_body(response)?;
    let json: serde_json::Value = serde_json::from_str(response_body)?;
    assert_eq!(
        json.get("accepted").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    Ok(())
}

fn assert_rejection_code(response: &str, expected_code: &str) -> Result<(), Box<dyn Error>> {
    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    let response_body = response_body(response)?;
    let json: serde_json::Value = serde_json::from_str(response_body)?;
    assert_eq!(
        json.get("accepted").and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert_eq!(
        json.pointer("/rejection/code")
            .and_then(serde_json::Value::as_str),
        Some(expected_code)
    );
    Ok(())
}

fn temp_file_path(prefix: &str) -> PathBuf {
    let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{counter}.json", std::process::id()))
}

fn terminate(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
