use std::collections::BTreeMap;
use std::error::Error;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
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
    let replay_store = temp_file_path("rava-serve-health-replay");
    let replay_store_arg = replay_store.to_string_lossy().into_owned();
    let audit_log = temp_file_path("rava-serve-health-audit");
    let audit_log_arg = audit_log.to_string_lossy().into_owned();
    let response = run_server_raw_request_with_env(
        &[
            "--max-request-bytes",
            "4096",
            "--replay-store",
            &replay_store_arg,
            "--require-replay-store",
            "--audit-log",
            &audit_log_arg,
            "--require-audit-log",
            "--auth-token-env",
            "RAVA_TEST_AUTH_TOKEN",
            "--require-auth-token-env",
            "--caller-id",
            "tenant-a",
            "--require-caller-id",
            "--rate-limit-per-minute",
            "120",
            "--require-rate-limit-per-minute",
            "--metrics",
            "--require-metrics",
        ],
        &[("RAVA_TEST_AUTH_TOKEN", "secret-token")],
        "GET /healthz HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer secret-token\r\nConnection: close\r\n\r\n",
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
    assert_eq!(
        json.get("replay_store_configured")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        json.get("require_replay_store")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        json.get("audit_log_configured")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        json.get("require_audit_log")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        json.get("auth_required")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        json.get("require_auth_token_env")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        json.get("caller_id_configured")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        json.get("require_caller_id")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        json.get("rate_limit_per_minute")
            .and_then(serde_json::Value::as_u64),
        Some(120)
    );
    assert_eq!(
        json.get("require_rate_limit_per_minute")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        json.get("metrics_configured")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        json.get("require_metrics")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    Ok(())
}

#[test]
fn serve_verify_with_auth_token_env_rejects_missing_authorization() -> Result<(), Box<dyn Error>> {
    let response = run_server_raw_request_with_env(
        &["--auth-token-env", "RAVA_TEST_AUTH_TOKEN"],
        &[("RAVA_TEST_AUTH_TOKEN", "secret-token")],
        "GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )?;

    assert!(
        response.starts_with("HTTP/1.1 401 Unauthorized"),
        "{response}"
    );
    let response_body = response_body(&response)?;
    let json: serde_json::Value = serde_json::from_str(response_body)?;
    assert_eq!(
        json.get("error").and_then(serde_json::Value::as_str),
        Some("authorization required")
    );
    assert!(
        !response.contains("secret-token"),
        "auth response must not echo token: {response}"
    );
    Ok(())
}

#[test]
fn serve_verify_with_auth_token_env_accepts_matching_bearer_token() -> Result<(), Box<dyn Error>> {
    let response = run_server_raw_request_with_env(
        &["--auth-token-env", "RAVA_TEST_AUTH_TOKEN"],
        &[("RAVA_TEST_AUTH_TOKEN", "secret-token")],
        "GET /healthz HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer secret-token\r\nConnection: close\r\n\r\n",
    )?;

    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    let response_body = response_body(&response)?;
    let json: serde_json::Value = serde_json::from_str(response_body)?;
    assert_eq!(
        json.get("auth_required")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    Ok(())
}

#[test]
fn serve_verify_rejects_caller_id_without_auth_token_env() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "serve",
            "verify",
            "--addr",
            "127.0.0.1:0",
            "--caller-id",
            "tenant-a",
        ])
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--caller-id requires --auth-token-env"),
        "{stderr}"
    );
    Ok(())
}

#[test]
fn serve_verify_rejects_invalid_caller_id_label() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "serve",
            "verify",
            "--addr",
            "127.0.0.1:0",
            "--auth-token-env",
            "RAVA_TEST_AUTH_TOKEN",
            "--caller-id",
            "tenant a",
        ])
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid caller-id"), "{stderr}");
    Ok(())
}

#[test]
fn serve_verify_requires_caller_id_requires_caller_id() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "serve",
            "verify",
            "--addr",
            "127.0.0.1:0",
            "--auth-token-env",
            "RAVA_TEST_AUTH_TOKEN",
            "--require-caller-id",
        ])
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("require-caller-id requires caller-id"),
        "{stderr}"
    );
    Ok(())
}

#[test]
fn serve_verify_with_rate_limit_rejects_excess_requests() -> Result<(), Box<dyn Error>> {
    let responses = run_server_raw_requests(
        &["--rate-limit-per-minute", "1"],
        &[
            "GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            "GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        ],
    )?;

    assert!(
        responses[0].starts_with("HTTP/1.1 200 OK"),
        "{}",
        responses[0]
    );
    assert!(
        responses[1].starts_with("HTTP/1.1 429 Too Many Requests"),
        "{}",
        responses[1]
    );
    let response_body = response_body(&responses[1])?;
    let json: serde_json::Value = serde_json::from_str(response_body)?;
    assert_eq!(
        json.get("error").and_then(serde_json::Value::as_str),
        Some("rate limit exceeded")
    );
    Ok(())
}

#[test]
fn serve_verify_rejects_zero_rate_limit() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "serve",
            "verify",
            "--addr",
            "127.0.0.1:0",
            "--rate-limit-per-minute",
            "0",
        ])
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("rate-limit-per-minute must be greater than zero"),
        "{stderr}"
    );
    Ok(())
}

#[test]
fn serve_verify_requires_fresh_revocations_requires_revocation_store() -> Result<(), Box<dyn Error>>
{
    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "serve",
            "verify",
            "--addr",
            "127.0.0.1:0",
            "--require-fresh-revocations",
        ])
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("require-fresh-revocations requires revocation-store"),
        "{stderr}"
    );
    Ok(())
}

#[test]
fn serve_verify_requires_replay_store_requires_replay_store() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "serve",
            "verify",
            "--addr",
            "127.0.0.1:0",
            "--require-replay-store",
        ])
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("require-replay-store requires replay-store"),
        "{stderr}"
    );
    Ok(())
}

#[test]
fn serve_verify_requires_audit_log_requires_audit_log() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "serve",
            "verify",
            "--addr",
            "127.0.0.1:0",
            "--require-audit-log",
        ])
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("require-audit-log requires audit-log"),
        "{stderr}"
    );
    Ok(())
}

#[test]
fn serve_verify_requires_auth_token_env_requires_auth_token_env() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "serve",
            "verify",
            "--addr",
            "127.0.0.1:0",
            "--require-auth-token-env",
        ])
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("require-auth-token-env requires auth-token-env"),
        "{stderr}"
    );
    Ok(())
}

#[test]
fn serve_verify_requires_rate_limit_requires_rate_limit() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "serve",
            "verify",
            "--addr",
            "127.0.0.1:0",
            "--require-rate-limit-per-minute",
        ])
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("require-rate-limit-per-minute requires rate-limit-per-minute"),
        "{stderr}"
    );
    Ok(())
}

#[test]
fn serve_verify_requires_metrics_requires_metrics() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "serve",
            "verify",
            "--addr",
            "127.0.0.1:0",
            "--require-metrics",
        ])
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("require-metrics requires metrics"),
        "{stderr}"
    );
    Ok(())
}

#[test]
fn serve_verify_with_rate_limit_reports_caller_scope_when_caller_id_configured(
) -> Result<(), Box<dyn Error>> {
    let responses = run_server_raw_requests_with_env(
        &[
            "--auth-token-env",
            "RAVA_TEST_AUTH_TOKEN",
            "--caller-id",
            "tenant-a",
            "--rate-limit-per-minute",
            "1",
        ],
        &[("RAVA_TEST_AUTH_TOKEN", "secret-token")],
        &[
            "GET /healthz HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer secret-token\r\nConnection: close\r\n\r\n",
            "GET /healthz HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer secret-token\r\nConnection: close\r\n\r\n",
        ],
    )?;

    assert!(
        responses[0].starts_with("HTTP/1.1 200 OK"),
        "{}",
        responses[0]
    );
    let health_body = response_body(&responses[0])?;
    let health: serde_json::Value = serde_json::from_str(health_body)?;
    assert_eq!(
        health
            .get("rate_limit_scope")
            .and_then(serde_json::Value::as_str),
        Some("caller")
    );
    assert!(
        responses[1].starts_with("HTTP/1.1 429 Too Many Requests"),
        "{}",
        responses[1]
    );
    let response_body = response_body(&responses[1])?;
    let json: serde_json::Value = serde_json::from_str(response_body)?;
    assert_eq!(
        json.get("rate_limit_scope")
            .and_then(serde_json::Value::as_str),
        Some("caller")
    );
    assert_eq!(
        json.get("caller_id").and_then(serde_json::Value::as_str),
        Some("tenant-a")
    );
    Ok(())
}

#[test]
fn serve_verify_with_metrics_reports_metadata_counters() -> Result<(), Box<dyn Error>> {
    let accepted_body = accepted_request_body()?;
    let mut rejected_body = accepted_body.clone();
    rejected_body["action"]["constraints"]["amount_usd"]["integer"] = serde_json::json!(900);
    let action_id = string_field(&accepted_body["action"], "id")?.to_owned();

    let requests = [
        post_json_request("/verify/action", &accepted_body)?,
        post_json_request("/verify/action", &rejected_body)?,
        "GET /metrics HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n".to_owned(),
    ];
    let request_refs = requests.iter().map(String::as_str).collect::<Vec<_>>();

    let responses = run_server_raw_requests(&["--metrics"], &request_refs)?;

    assert_accepted_response(&responses[0])?;
    assert_rejection_code(&responses[1], "action_signature_invalid")?;
    assert!(
        responses[2].starts_with("HTTP/1.1 200 OK"),
        "{}",
        responses[2]
    );
    let metrics = response_body(&responses[2])?;
    assert!(
        metrics.contains(
            "rava_preview_http_requests_total{route=\"/verify/action\",status=\"200\"} 2"
        ),
        "{metrics}"
    );
    assert!(
        metrics.contains("rava_preview_verifier_decisions_total{decision=\"accepted\"} 1"),
        "{metrics}"
    );
    assert!(
        metrics.contains("rava_preview_verifier_decisions_total{decision=\"rejected\"} 1"),
        "{metrics}"
    );
    assert!(
        metrics.contains(
            "rava_preview_verifier_rejections_total{code=\"action_signature_invalid\"} 1"
        ),
        "{metrics}"
    );
    assert!(!metrics.contains(action_id.as_str()), "{metrics}");
    assert!(!metrics.contains("intent"), "{metrics}");
    assert!(!metrics.contains("resource"), "{metrics}");
    assert!(!metrics.contains("constraints"), "{metrics}");
    assert!(!metrics.contains("proof"), "{metrics}");
    Ok(())
}

#[test]
fn serve_verify_metrics_reports_replay_attempts() -> Result<(), Box<dyn Error>> {
    let replay_store = temp_file_path("rava-serve-metrics-replay");
    let replay_store_arg = replay_store.to_string_lossy().into_owned();
    let body = accepted_request_body()?;
    let requests = [
        post_json_request("/verify/action", &body)?,
        post_json_request("/verify/action", &body)?,
        "GET /metrics HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n".to_owned(),
    ];
    let request_refs = requests.iter().map(String::as_str).collect::<Vec<_>>();

    let responses = run_server_raw_requests(
        &["--metrics", "--replay-store", &replay_store_arg],
        &request_refs,
    )?;

    assert_accepted_response(&responses[0])?;
    assert_rejection_code(&responses[1], "action_replayed")?;
    let metrics = response_body(&responses[2])?;
    assert!(
        metrics.contains("rava_preview_replay_attempts_total 1"),
        "{metrics}"
    );
    assert!(
        !metrics.contains(string_field(&body["action"], "id")?),
        "{metrics}"
    );

    std::fs::remove_file(replay_store)?;
    Ok(())
}

#[test]
fn serve_verify_metrics_reports_replay_store_failures() -> Result<(), Box<dyn Error>> {
    let replay_store = temp_file_path("rava-serve-metrics-replay-corrupt");
    std::fs::write(&replay_store, b"not json")?;
    let replay_store_arg = replay_store.to_string_lossy().into_owned();
    let body = accepted_request_body()?;
    let verify_request = post_json_request("/verify/action", &body)?;

    let responses = run_server_raw_requests(
        &["--metrics", "--replay-store", &replay_store_arg],
        &[
            verify_request.as_str(),
            "GET /metrics HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        ],
    )?;

    assert!(
        responses[0].starts_with("HTTP/1.1 500 Internal Server Error"),
        "{}",
        responses[0]
    );
    let metrics = response_body(&responses[1])?;
    assert!(
        metrics.contains("rava_preview_replay_store_failures_total 1"),
        "{metrics}"
    );
    assert!(!metrics.contains(replay_store_arg.as_str()), "{metrics}");
    assert!(!metrics.contains("not json"), "{metrics}");

    std::fs::remove_file(replay_store)?;
    Ok(())
}

#[test]
fn serve_verify_metrics_reports_revocation_read_failures() -> Result<(), Box<dyn Error>> {
    let revocation_store = temp_file_path("rava-serve-metrics-revocations");
    std::fs::write(&revocation_store, b"not json")?;
    let revocation_store_arg = revocation_store.to_string_lossy().into_owned();
    let body = accepted_request_body()?;
    let verify_request = post_json_request("/verify/action", &body)?;

    let responses = run_server_raw_requests(
        &["--metrics", "--revocation-store", &revocation_store_arg],
        &[
            verify_request.as_str(),
            "GET /metrics HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        ],
    )?;

    assert!(
        responses[0].starts_with("HTTP/1.1 500 Internal Server Error"),
        "{}",
        responses[0]
    );
    let metrics = response_body(&responses[1])?;
    assert!(
        metrics.contains("rava_preview_revocation_read_failures_total 1"),
        "{metrics}"
    );
    assert!(
        !metrics.contains(revocation_store_arg.as_str()),
        "{metrics}"
    );
    assert!(!metrics.contains("not json"), "{metrics}");

    std::fs::remove_file(revocation_store)?;
    Ok(())
}

#[test]
fn serve_verify_metrics_reports_revocation_freshness_failures() -> Result<(), Box<dyn Error>> {
    let revocation_store = temp_file_path("rava-serve-metrics-stale-revocations");
    let revocation_store_arg = revocation_store.to_string_lossy().into_owned();
    std::fs::write(
        &revocation_store,
        serde_json::to_vec_pretty(&serde_json::json!({
            "revoked_ids": [],
            "fresh_until_unix": 1650000000
        }))?,
    )?;
    let body = accepted_request_body()?;
    let verify_request = post_json_request("/verify/action", &body)?;

    let responses = run_server_raw_requests(
        &[
            "--metrics",
            "--revocation-store",
            &revocation_store_arg,
            "--require-fresh-revocations",
        ],
        &[
            verify_request.as_str(),
            "GET /metrics HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        ],
    )?;

    assert!(
        responses[0].starts_with("HTTP/1.1 500 Internal Server Error"),
        "{}",
        responses[0]
    );
    let metrics = response_body(&responses[1])?;
    assert!(
        metrics.contains("rava_preview_revocation_freshness_failures_total 1"),
        "{metrics}"
    );
    assert!(
        !metrics.contains(revocation_store_arg.as_str()),
        "{metrics}"
    );
    assert!(!metrics.contains("fresh_until_unix"), "{metrics}");

    std::fs::remove_file(revocation_store)?;
    Ok(())
}

#[test]
fn serve_verify_metrics_reports_missing_public_keys() -> Result<(), Box<dyn Error>> {
    let mut body = accepted_request_body()?;
    let issuer = body["issuer_public_keys"]
        .as_object()
        .and_then(|keys| keys.keys().next())
        .ok_or("missing issuer key")?
        .to_owned();
    body["issuer_public_keys"]
        .as_object_mut()
        .ok_or("issuer_public_keys must be an object")?
        .remove(&issuer);
    let requests = [
        post_json_request("/verify/action", &body)?,
        "GET /metrics HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n".to_owned(),
    ];
    let request_refs = requests.iter().map(String::as_str).collect::<Vec<_>>();

    let responses = run_server_raw_requests(&["--metrics"], &request_refs)?;

    assert_rejection_code(&responses[0], "missing_issuer_public_key")?;
    let metrics = response_body(&responses[1])?;
    assert!(
        metrics.contains("rava_preview_missing_public_keys_total 1"),
        "{metrics}"
    );
    assert!(!metrics.contains(issuer.as_str()), "{metrics}");
    Ok(())
}

#[test]
fn serve_verify_metrics_reports_verifier_latency() -> Result<(), Box<dyn Error>> {
    let body = accepted_request_body()?;
    let requests = [
        post_json_request("/verify/action", &body)?,
        "GET /metrics HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n".to_owned(),
    ];
    let request_refs = requests.iter().map(String::as_str).collect::<Vec<_>>();

    let responses = run_server_raw_requests(&["--metrics"], &request_refs)?;

    assert_accepted_response(&responses[0])?;
    let metrics = response_body(&responses[1])?;
    assert!(
        metrics.contains("rava_preview_verifier_latency_ms_count 1"),
        "{metrics}"
    );
    assert!(
        metrics
            .lines()
            .any(|line| line.starts_with("rava_preview_verifier_latency_ms_total ")),
        "{metrics}"
    );
    assert!(
        !metrics.contains(string_field(&body["action"], "id")?),
        "{metrics}"
    );
    Ok(())
}

#[test]
fn serve_verify_metrics_requires_auth_when_configured() -> Result<(), Box<dyn Error>> {
    let responses = run_server_raw_requests_with_env(
        &["--metrics", "--auth-token-env", "RAVA_TEST_AUTH_TOKEN"],
        &[("RAVA_TEST_AUTH_TOKEN", "secret-token")],
        &[
            "GET /metrics HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            "GET /metrics HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer secret-token\r\nConnection: close\r\n\r\n",
        ],
    )?;

    assert!(
        responses[0].starts_with("HTTP/1.1 401 Unauthorized"),
        "{}",
        responses[0]
    );
    assert!(
        responses[1].starts_with("HTTP/1.1 200 OK"),
        "{}",
        responses[1]
    );
    let metrics = response_body(&responses[1])?;
    assert!(
        metrics.contains("rava_preview_http_requests_total{route=\"/metrics\",status=\"401\"} 1"),
        "{metrics}"
    );
    assert!(!metrics.contains("secret-token"), "{metrics}");
    Ok(())
}

#[test]
fn serve_verify_metrics_reports_request_body_limit_rejections() -> Result<(), Box<dyn Error>> {
    let oversized = format!(
        "POST /verify/action HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        4096
    );
    let responses = run_server_raw_requests(
        &["--metrics", "--max-request-bytes", "128"],
        &[
            oversized.as_str(),
            "GET /metrics HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        ],
    )?;

    assert!(
        responses[0].starts_with("HTTP/1.1 413 Payload Too Large"),
        "{}",
        responses[0]
    );
    assert!(
        responses[1].starts_with("HTTP/1.1 200 OK"),
        "{}",
        responses[1]
    );
    let metrics = response_body(&responses[1])?;
    assert!(
        metrics.contains(
            "rava_preview_http_requests_total{route=\"/verify/action\",status=\"413\"} 1"
        ),
        "{metrics}"
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

#[test]
fn serve_verify_requires_fresh_revocations_rejects_stale_snapshot() -> Result<(), Box<dyn Error>> {
    let revocation_store = temp_file_path("rava-serve-stale-revocations");
    let revocation_store_arg = revocation_store.to_string_lossy().into_owned();
    std::fs::write(
        &revocation_store,
        serde_json::to_vec_pretty(&serde_json::json!({
            "revoked_ids": [],
            "fresh_until_unix": 1650000000
        }))?,
    )?;
    let body = accepted_request_body()?;

    let response = run_server_request_with_args(
        &[
            "--revocation-store",
            &revocation_store_arg,
            "--require-fresh-revocations",
        ],
        &body,
    )?;

    assert!(
        response.starts_with("HTTP/1.1 500 Internal Server Error"),
        "{response}"
    );
    let response_body = response_body(&response)?;
    let json: serde_json::Value = serde_json::from_str(response_body)?;
    assert_eq!(
        json.get("error").and_then(serde_json::Value::as_str),
        Some("revocation store is stale")
    );

    std::fs::remove_file(revocation_store)?;
    Ok(())
}

#[test]
fn serve_verify_requires_fresh_revocations_accepts_fresh_snapshot() -> Result<(), Box<dyn Error>> {
    let revocation_store = temp_file_path("rava-serve-fresh-revocations");
    let revocation_store_arg = revocation_store.to_string_lossy().into_owned();
    std::fs::write(
        &revocation_store,
        serde_json::to_vec_pretty(&serde_json::json!({
            "revoked_ids": [],
            "fresh_until_unix": 1650000001
        }))?,
    )?;
    let body = accepted_request_body()?;

    let response = run_server_request_with_args(
        &[
            "--revocation-store",
            &revocation_store_arg,
            "--require-fresh-revocations",
        ],
        &body,
    )?;

    assert_accepted_response(&response)?;
    std::fs::remove_file(revocation_store)?;
    Ok(())
}

#[test]
fn serve_verify_with_audit_log_appends_decision_metadata_without_raw_payload(
) -> Result<(), Box<dyn Error>> {
    let audit_log = temp_file_path("rava-serve-audit");
    let audit_log_arg = audit_log.to_string_lossy().into_owned();
    let accepted_body = accepted_request_body()?;
    let mut rejected_body = accepted_body.clone();
    rejected_body["action"]["constraints"]["amount_usd"]["integer"] = serde_json::json!(900);
    let action_id = string_field(&accepted_body["action"], "id")?.to_owned();

    let responses = run_server_requests_with_args(
        &["--audit-log", &audit_log_arg],
        &[accepted_body, rejected_body],
    )?;

    assert_accepted_response(&responses[0])?;
    assert_rejection_code(&responses[1], "action_signature_invalid")?;
    let audit_lines = std::fs::read_to_string(&audit_log)?;
    let entries = audit_lines
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<Vec<serde_json::Value>, _>>()?;
    assert_eq!(entries.len(), 2);
    assert_eq!(
        entries[0]
            .get("action_id")
            .and_then(serde_json::Value::as_str),
        Some(action_id.as_str())
    );
    assert_eq!(
        entries[0]
            .get("accepted")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        entries[1]
            .pointer("/rejection/code")
            .and_then(serde_json::Value::as_str),
        Some("action_signature_invalid")
    );
    for entry in entries {
        assert!(entry.get("intent").is_none(), "{entry}");
        assert!(entry.get("resource").is_none(), "{entry}");
        assert!(entry.get("constraints").is_none(), "{entry}");
        assert!(entry.get("proof").is_none(), "{entry}");
    }

    std::fs::remove_file(audit_log)?;
    Ok(())
}

#[test]
fn serve_verify_with_caller_id_records_audit_correlation() -> Result<(), Box<dyn Error>> {
    let audit_log = temp_file_path("rava-serve-caller-audit");
    let audit_log_arg = audit_log.to_string_lossy().into_owned();
    let body = accepted_request_body()?;
    let actor_id = string_field(&body["action"], "actor")?.to_owned();
    let request =
        post_json_request_with_authorization("/verify/action", &body, "Bearer secret-token")?;

    let response = run_server_raw_request_with_env(
        &[
            "--auth-token-env",
            "RAVA_TEST_AUTH_TOKEN",
            "--caller-id",
            "tenant-a",
            "--audit-log",
            &audit_log_arg,
        ],
        &[("RAVA_TEST_AUTH_TOKEN", "secret-token")],
        &request,
    )?;

    assert_accepted_response(&response)?;
    let audit_lines = std::fs::read_to_string(&audit_log)?;
    let entries = audit_lines
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<Vec<serde_json::Value>, _>>()?;
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0]
            .get("caller_id")
            .and_then(serde_json::Value::as_str),
        Some("tenant-a")
    );
    assert_eq!(
        entries[0]
            .get("actor_id")
            .and_then(serde_json::Value::as_str),
        Some(actor_id.as_str())
    );
    assert_ne!(
        entries[0]
            .get("caller_id")
            .and_then(serde_json::Value::as_str),
        Some(actor_id.as_str())
    );

    std::fs::remove_file(audit_log)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn serve_verify_with_audit_log_creates_owner_only_file() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::PermissionsExt;

    let audit_log = temp_file_path("rava-serve-audit-permissions");
    let audit_log_arg = audit_log.to_string_lossy().into_owned();
    let body = accepted_request_body()?;

    let response = run_server_request_with_args(&["--audit-log", &audit_log_arg], &body)?;

    assert_accepted_response(&response)?;
    assert_eq!(
        std::fs::metadata(&audit_log)?.permissions().mode() & 0o777,
        0o600
    );
    std::fs::remove_file(audit_log)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn serve_verify_with_audit_log_rejects_insecure_file_permissions() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::PermissionsExt;

    let audit_log = temp_file_path("rava-serve-audit-insecure");
    std::fs::write(&audit_log, b"")?;
    let mut permissions = std::fs::metadata(&audit_log)?.permissions();
    permissions.set_mode(0o644);
    std::fs::set_permissions(&audit_log, permissions)?;
    let audit_log_arg = audit_log.to_string_lossy().into_owned();
    let body = accepted_request_body()?;

    let response = run_server_request_with_args(&["--audit-log", &audit_log_arg], &body)?;

    assert!(response.starts_with("HTTP/1.1 500 Internal Server Error"));
    assert_eq!(std::fs::read_to_string(&audit_log)?, "");
    std::fs::remove_file(audit_log)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn serve_verify_with_audit_log_rejects_symlink_path() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::PermissionsExt;

    let target = temp_file_path("rava-serve-audit-symlink-target");
    std::fs::write(&target, b"")?;
    let mut permissions = std::fs::metadata(&target)?.permissions();
    permissions.set_mode(0o600);
    std::fs::set_permissions(&target, permissions)?;

    let audit_log = temp_file_path("rava-serve-audit-symlink");
    std::os::unix::fs::symlink(&target, &audit_log)?;
    let audit_log_arg = audit_log.to_string_lossy().into_owned();
    let body = accepted_request_body()?;

    let response = run_server_request_with_args(&["--audit-log", &audit_log_arg], &body)?;

    assert!(response.starts_with("HTTP/1.1 500 Internal Server Error"));
    assert_eq!(std::fs::read_to_string(&target)?, "");
    std::fs::remove_file(audit_log)?;
    std::fs::remove_file(target)?;
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
    let (mut server, address) = spawn_server(extra_args, &[])?;

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
    run_server_raw_request_with_env(extra_args, &[], request)
}

fn run_server_raw_request_with_env(
    extra_args: &[&str],
    envs: &[(&str, &str)],
    request: &str,
) -> Result<String, Box<dyn Error>> {
    let responses = run_server_raw_requests_with_env(extra_args, envs, &[request])?;
    responses
        .into_iter()
        .next()
        .ok_or_else(|| "missing server response".into())
}

fn run_server_raw_requests(
    extra_args: &[&str],
    requests: &[&str],
) -> Result<Vec<String>, Box<dyn Error>> {
    run_server_raw_requests_with_env(extra_args, &[], requests)
}

fn run_server_raw_requests_with_env(
    extra_args: &[&str],
    envs: &[(&str, &str)],
    requests: &[&str],
) -> Result<Vec<String>, Box<dyn Error>> {
    let (mut server, address) = spawn_server(extra_args, envs)?;

    let mut responses = Vec::new();
    for request in requests {
        match raw_request_when_ready(&address, request) {
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

fn spawn_server(
    extra_args: &[&str],
    envs: &[(&str, &str)],
) -> Result<(Child, String), Box<dyn Error>> {
    let mut args = vec!["serve", "verify", "--addr", "127.0.0.1:0"];
    args.extend_from_slice(extra_args);
    let mut command = Command::new(env!("CARGO_BIN_EXE_rava"));
    command.args(args).stdout(Stdio::piped());
    for (name, value) in envs {
        command.env(name, value);
    }
    let mut server = command.spawn()?;
    let stdout = server.stdout.take().ok_or("server stdout was not piped")?;
    let mut lines = BufReader::new(stdout).lines();
    let Some(line) = lines.next() else {
        terminate(&mut server);
        return Err("server exited before reporting listening address".into());
    };
    let line = line?;
    let Some(address) = line.strip_prefix("Rava verifier service listening: ") else {
        terminate(&mut server);
        return Err(format!("unexpected server startup line: {line}").into());
    };

    Ok((server, address.to_owned()))
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
                            ErrorKind::BrokenPipe | ErrorKind::ConnectionReset
                        ) => {}
                    Err(error) => return Err(error.into()),
                }
                match read_response_allowing_reset(&mut stream) {
                    Ok(response) => return Ok(response),
                    Err(error)
                        if error.kind() == ErrorKind::ConnectionReset
                            && started.elapsed() < Duration::from_secs(5) =>
                    {
                        thread::sleep(Duration::from_millis(25));
                    }
                    Err(error) => return Err(error.into()),
                }
            }
            Err(error) if started.elapsed() < Duration::from_secs(5) => {
                let _ = error;
                thread::sleep(Duration::from_millis(25));
            }
            Err(error) => return Err(error.into()),
        }
    }
}

fn post_json_request(path: &str, body: &serde_json::Value) -> Result<String, Box<dyn Error>> {
    let body = serde_json::to_string(body)?;
    Ok(format!(
        "POST {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    ))
}

fn post_json_request_with_authorization(
    path: &str,
    body: &serde_json::Value,
    authorization: &str,
) -> Result<String, Box<dyn Error>> {
    let body = serde_json::to_string(body)?;
    Ok(format!(
        "POST {path} HTTP/1.1\r\nHost: localhost\r\nAuthorization: {authorization}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    ))
}

fn read_response_allowing_reset(stream: &mut TcpStream) -> Result<String, std::io::Error> {
    let mut response = Vec::new();
    let mut chunk = [0_u8; 512];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => response.extend_from_slice(&chunk[..count]),
            Err(error) if error.kind() == ErrorKind::ConnectionReset && !response.is_empty() => {
                break;
            }
            Err(error) => return Err(error),
        }
    }

    Ok(String::from_utf8_lossy(&response).into_owned())
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
