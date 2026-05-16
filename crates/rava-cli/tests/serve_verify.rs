use std::collections::BTreeMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

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
    let address = free_loopback_address()?;
    let mut server = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args(["serve", "verify", "--addr", &address])
        .spawn()?;

    let response = match post_json_when_ready(&address, "/verify/action", body) {
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

fn response_body(response: &str) -> Result<&str, Box<dyn Error>> {
    response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .ok_or_else(|| "missing HTTP response body".into())
}

fn terminate(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
