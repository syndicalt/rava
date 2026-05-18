use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::time::{Duration, Instant};

use rava_core::action::ActionEnvelope;
use rava_core::capability::Capability;
use rava_core::replay::FileReplayRegistry;
use rava_core::revocation::{
    FileRevocationRegistry, InMemoryRevocationRegistry, RevocationRegistry,
};
use rava_core::verifier::{
    verify_action, verify_action_once, VerificationResult, VerifyActionInput, VerifyActionOnceInput,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::cli::ServeVerifyArgs;

const SERVICE_NAME: &str = "rava-verifier-preview-v0";
const MAX_HEADER_BYTES: usize = 16 * 1024;

#[derive(Debug, Deserialize)]
struct VerifyActionRequest {
    action: ActionEnvelope,
    capability_chain: Vec<Capability>,
    actor_public_key_hex: String,
    issuer_public_keys: BTreeMap<String, String>,
    now_unix: i64,
}

#[derive(Debug, Serialize)]
struct VerifyActionResponse {
    service: &'static str,
    accepted: bool,
    rejection: Option<VerifyActionRejection>,
}

#[derive(Debug, Serialize)]
struct VerifyActionRejection {
    code: String,
    subject: Option<String>,
}

pub fn run_serve_verify(args: ServeVerifyArgs) -> Result<(), Box<dyn Error>> {
    if args.caller_id.is_some() && args.auth_token_env.is_none() {
        return Err("--caller-id requires --auth-token-env".into());
    }
    if args.require_auth_token_env && args.auth_token_env.is_none() {
        return Err("require-auth-token-env requires auth-token-env".into());
    }
    if args.require_replay_store && args.replay_store.is_none() {
        return Err("require-replay-store requires replay-store".into());
    }
    if args.require_fresh_revocations && args.revocation_store.is_none() {
        return Err("require-fresh-revocations requires revocation-store".into());
    }
    if args.require_audit_log && args.audit_log.is_none() {
        return Err("require-audit-log requires audit-log".into());
    }
    if let Some(caller_id) = &args.caller_id {
        validate_caller_id(caller_id)?;
    }
    if args.rate_limit_per_minute == Some(0) {
        return Err("rate-limit-per-minute must be greater than zero".into());
    }
    let listener = TcpListener::bind(&args.addr)?;
    let mut rate_limit = args.rate_limit_per_minute.map(RateLimitState::new);
    let mut metrics = MetricsState::default();
    println!(
        "Rava verifier service listening: {}",
        listener.local_addr()?
    );

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) =
                    handle_connection(&mut stream, &args, &mut rate_limit, &mut metrics)
                {
                    write_json_response(
                        &mut stream,
                        "500 Internal Server Error",
                        &serde_json::json!({
                            "service": SERVICE_NAME,
                            "error": error.to_string()
                        }),
                    )?;
                }
            }
            Err(error) => return Err(error.into()),
        }
    }

    Ok(())
}

fn validate_caller_id(caller_id: &str) -> Result<(), Box<dyn Error>> {
    if caller_id.is_empty() || caller_id.len() > 128 {
        return Err("invalid caller-id: must be 1..=128 characters".into());
    }
    if !caller_id.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'@')
    }) {
        return Err(
            "invalid caller-id: use ASCII letters, digits, '.', '_', '-', ':', or '@'".into(),
        );
    }
    Ok(())
}

fn handle_connection(
    stream: &mut TcpStream,
    args: &ServeVerifyArgs,
    rate_limit: &mut Option<RateLimitState>,
    metrics: &mut MetricsState,
) -> Result<(), Box<dyn Error>> {
    let Some(request) = read_http_request(stream, args.max_request_bytes, metrics)? else {
        return Ok(());
    };
    let route = route_label(&request.method, &request.path);
    if !request_is_authorized(&request, args)? {
        metrics.record_http(route, "401");
        write_json_response(
            stream,
            "401 Unauthorized",
            &serde_json::json!({
                "service": SERVICE_NAME,
                "error": "authorization required"
            }),
        )?;
        return Ok(());
    }
    if let Some(rate_limit) = rate_limit {
        if !rate_limit.allow_request() {
            let rate_limit_scope = rate_limit_scope(args);
            metrics.record_http(route, "429");
            write_json_response(
                stream,
                "429 Too Many Requests",
                &serde_json::json!({
                    "service": SERVICE_NAME,
                    "error": "rate limit exceeded",
                    "rate_limit_per_minute": rate_limit.limit,
                    "rate_limit_scope": rate_limit_scope,
                    "caller_id": args.caller_id,
                }),
            )?;
            return Ok(());
        }
    }
    if request.method == "GET" && request.path == "/metrics" && args.metrics {
        metrics.record_http(route, "200");
        write_text_response(stream, "200 OK", &metrics.render())?;
        return Ok(());
    }
    if request.method == "GET" && request.path == "/healthz" {
        metrics.record_http(route, "200");
        write_json_response(
            stream,
            "200 OK",
            &serde_json::json!({
                "service": SERVICE_NAME,
                "status": "ok",
                "max_request_bytes": args.max_request_bytes,
                "replay_store_configured": args.replay_store.is_some(),
                "require_replay_store": args.require_replay_store,
                "revocation_store_configured": args.revocation_store.is_some(),
                "require_fresh_revocations": args.require_fresh_revocations,
                "audit_log_configured": args.audit_log.is_some(),
                "require_audit_log": args.require_audit_log,
                "auth_required": args.auth_token_env.is_some(),
                "require_auth_token_env": args.require_auth_token_env,
                "caller_id_configured": args.caller_id.is_some(),
                "rate_limit_per_minute": args.rate_limit_per_minute,
                "rate_limit_scope": rate_limit_scope(args),
                "metrics_configured": args.metrics,
            }),
        )?;
        return Ok(());
    }
    if request.method != "POST" || request.path != "/verify/action" {
        metrics.record_http(route, "404");
        write_json_response(
            stream,
            "404 Not Found",
            &serde_json::json!({
                "service": SERVICE_NAME,
                "error": "only POST /verify/action is supported"
            }),
        )?;
        return Ok(());
    }

    let request: VerifyActionRequest = serde_json::from_slice(&request.body)?;
    let now = OffsetDateTime::from_unix_timestamp(request.now_unix)?;
    let verifier_started = Instant::now();
    let result = if let Some(revocation_store) = &args.revocation_store {
        let revocations = match FileRevocationRegistry::open(revocation_store) {
            Ok(revocations) => revocations,
            Err(error) => {
                metrics.revocation_read_failures += 1;
                metrics.record_http(route, "500");
                return Err(error.into());
            }
        };
        if args.require_fresh_revocations {
            require_fresh_revocation_snapshot(&revocations, request.now_unix)?;
        }
        verify_action_with_optional_replay(&request, &revocations, args, now)
    } else {
        let revocations = InMemoryRevocationRegistry::default();
        verify_action_with_optional_replay(&request, &revocations, args, now)
    };
    let result = match result {
        Ok(result) => result,
        Err(error) if args.replay_store.is_some() => {
            metrics.replay_store_failures += 1;
            metrics.record_http(route, "500");
            return Err(error);
        }
        Err(error) => return Err(error),
    };
    metrics.record_verifier_latency(verifier_started.elapsed());
    if let Some(audit_log) = &args.audit_log {
        if let Err(error) =
            append_audit_log(audit_log, &request, &result, now, args.caller_id.as_deref())
        {
            metrics.audit_write_failures += 1;
            metrics.record_http(route, "500");
            return Err(error);
        }
    }
    metrics.record_verifier_decision(&result);
    metrics.record_http(route, "200");
    let response = match result {
        VerificationResult::Accepted => VerifyActionResponse {
            service: SERVICE_NAME,
            accepted: true,
            rejection: None,
        },
        VerificationResult::Rejected(error) => VerifyActionResponse {
            service: SERVICE_NAME,
            accepted: false,
            rejection: Some(VerifyActionRejection {
                code: error.code().to_owned(),
                subject: error.subject(),
            }),
        },
    };

    write_json_response(stream, "200 OK", &response)?;
    Ok(())
}

#[derive(Debug)]
struct RateLimitState {
    limit: usize,
    window_started: Instant,
    used: usize,
}

impl RateLimitState {
    fn new(limit: usize) -> Self {
        Self {
            limit,
            window_started: Instant::now(),
            used: 0,
        }
    }

    fn allow_request(&mut self) -> bool {
        if self.window_started.elapsed() >= Duration::from_secs(60) {
            self.window_started = Instant::now();
            self.used = 0;
        }
        if self.used >= self.limit {
            return false;
        }
        self.used += 1;
        true
    }
}

fn request_is_authorized(
    request: &HttpRequest,
    args: &ServeVerifyArgs,
) -> Result<bool, Box<dyn Error>> {
    let Some(token_env) = &args.auth_token_env else {
        return Ok(true);
    };
    let token = env::var(token_env)?;
    let expected = format!("Bearer {token}");
    Ok(request
        .headers
        .iter()
        .rev()
        .find(|(name, _)| name.eq_ignore_ascii_case("authorization"))
        .map(|(_, value)| value == &expected)
        .unwrap_or(false))
}

fn verify_action_with_optional_replay<R: RevocationRegistry>(
    request: &VerifyActionRequest,
    revocations: &R,
    args: &ServeVerifyArgs,
    now: OffsetDateTime,
) -> Result<VerificationResult, Box<dyn Error>> {
    let result = if let Some(replay_store) = &args.replay_store {
        let mut replay = FileReplayRegistry::open(replay_store)?;
        verify_action_once(VerifyActionOnceInput {
            action: &request.action,
            capability_chain: &request.capability_chain,
            actor_public_key_hex: &request.actor_public_key_hex,
            capability_issuer_public_keys: &request.issuer_public_keys,
            revocations,
            replay: &mut replay,
            now,
        })?
    } else {
        verify_action(VerifyActionInput {
            action: &request.action,
            capability_chain: &request.capability_chain,
            actor_public_key_hex: &request.actor_public_key_hex,
            capability_issuer_public_keys: &request.issuer_public_keys,
            revocations,
            now,
        })?
    };

    Ok(result)
}

fn require_fresh_revocation_snapshot(
    revocations: &FileRevocationRegistry,
    now_unix: i64,
) -> Result<(), Box<dyn Error>> {
    let fresh_until_unix = revocations.fresh_until_unix().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "revocation store missing fresh_until_unix",
        )
    })?;
    if fresh_until_unix <= now_unix {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "revocation store is stale").into());
    }
    Ok(())
}

fn rate_limit_scope(args: &ServeVerifyArgs) -> &'static str {
    if args.rate_limit_per_minute.is_none() {
        "none"
    } else if args.caller_id.is_some() {
        "caller"
    } else {
        "process"
    }
}

#[derive(Default)]
struct MetricsState {
    http_get_healthz_200: u64,
    http_get_metrics_200: u64,
    http_get_metrics_401: u64,
    http_post_verify_200: u64,
    http_post_verify_429: u64,
    http_post_verify_413: u64,
    http_post_verify_500: u64,
    http_other_401: u64,
    http_other_404: u64,
    http_other_431: u64,
    verifier_accepted: u64,
    verifier_rejected: u64,
    verifier_rejections: BTreeMap<String, u64>,
    verifier_latency_ms_count: u64,
    verifier_latency_ms_total: u64,
    replay_attempts: u64,
    replay_store_failures: u64,
    missing_public_keys: u64,
    revocation_read_failures: u64,
    audit_write_failures: u64,
}

impl MetricsState {
    fn record_http(&mut self, route: RouteLabel, status: &'static str) {
        match (route, status) {
            (RouteLabel::Healthz, "200") => self.http_get_healthz_200 += 1,
            (RouteLabel::Metrics, "200") => self.http_get_metrics_200 += 1,
            (RouteLabel::Metrics, "401") => self.http_get_metrics_401 += 1,
            (RouteLabel::VerifyAction, "200") => self.http_post_verify_200 += 1,
            (RouteLabel::VerifyAction, "429") => self.http_post_verify_429 += 1,
            (RouteLabel::VerifyAction, "413") => self.http_post_verify_413 += 1,
            (RouteLabel::VerifyAction, "500") => self.http_post_verify_500 += 1,
            (_, "401") => self.http_other_401 += 1,
            (_, "404") => self.http_other_404 += 1,
            (_, "431") => self.http_other_431 += 1,
            _ => {}
        }
    }

    fn record_verifier_decision(&mut self, result: &VerificationResult) {
        match result {
            VerificationResult::Accepted => self.verifier_accepted += 1,
            VerificationResult::Rejected(error) => {
                self.verifier_rejected += 1;
                *self
                    .verifier_rejections
                    .entry(error.code().to_owned())
                    .or_insert(0) += 1;
                if error.code() == "action_replayed" {
                    self.replay_attempts += 1;
                }
                if error.code() == "missing_issuer_public_key" {
                    self.missing_public_keys += 1;
                }
            }
        }
    }

    fn record_verifier_latency(&mut self, duration: Duration) {
        self.verifier_latency_ms_count += 1;
        self.verifier_latency_ms_total = self
            .verifier_latency_ms_total
            .saturating_add(duration.as_millis().try_into().unwrap_or(u64::MAX));
    }

    fn render(&self) -> String {
        let mut output = String::new();
        push_metric(
            &mut output,
            "rava_preview_http_requests_total",
            &[("route", "/healthz"), ("status", "200")],
            self.http_get_healthz_200,
        );
        push_metric(
            &mut output,
            "rava_preview_http_requests_total",
            &[("route", "/metrics"), ("status", "200")],
            self.http_get_metrics_200,
        );
        push_metric(
            &mut output,
            "rava_preview_http_requests_total",
            &[("route", "/metrics"), ("status", "401")],
            self.http_get_metrics_401,
        );
        push_metric(
            &mut output,
            "rava_preview_http_requests_total",
            &[("route", "/verify/action"), ("status", "200")],
            self.http_post_verify_200,
        );
        push_metric(
            &mut output,
            "rava_preview_http_requests_total",
            &[("route", "/verify/action"), ("status", "429")],
            self.http_post_verify_429,
        );
        push_metric(
            &mut output,
            "rava_preview_http_requests_total",
            &[("route", "/verify/action"), ("status", "413")],
            self.http_post_verify_413,
        );
        push_metric(
            &mut output,
            "rava_preview_http_requests_total",
            &[("route", "/verify/action"), ("status", "500")],
            self.http_post_verify_500,
        );
        push_metric(
            &mut output,
            "rava_preview_http_requests_total",
            &[("route", "other"), ("status", "401")],
            self.http_other_401,
        );
        push_metric(
            &mut output,
            "rava_preview_http_requests_total",
            &[("route", "other"), ("status", "404")],
            self.http_other_404,
        );
        push_metric(
            &mut output,
            "rava_preview_http_requests_total",
            &[("route", "other"), ("status", "431")],
            self.http_other_431,
        );
        push_metric(
            &mut output,
            "rava_preview_verifier_decisions_total",
            &[("decision", "accepted")],
            self.verifier_accepted,
        );
        push_metric(
            &mut output,
            "rava_preview_verifier_decisions_total",
            &[("decision", "rejected")],
            self.verifier_rejected,
        );
        for (code, count) in &self.verifier_rejections {
            push_metric(
                &mut output,
                "rava_preview_verifier_rejections_total",
                &[("code", code.as_str())],
                *count,
            );
        }
        push_metric(
            &mut output,
            "rava_preview_verifier_latency_ms_count",
            &[],
            self.verifier_latency_ms_count,
        );
        push_metric(
            &mut output,
            "rava_preview_verifier_latency_ms_total",
            &[],
            self.verifier_latency_ms_total,
        );
        push_metric(
            &mut output,
            "rava_preview_replay_attempts_total",
            &[],
            self.replay_attempts,
        );
        push_metric(
            &mut output,
            "rava_preview_replay_store_failures_total",
            &[],
            self.replay_store_failures,
        );
        push_metric(
            &mut output,
            "rava_preview_missing_public_keys_total",
            &[],
            self.missing_public_keys,
        );
        push_metric(
            &mut output,
            "rava_preview_revocation_read_failures_total",
            &[],
            self.revocation_read_failures,
        );
        push_metric(
            &mut output,
            "rava_preview_audit_write_failures_total",
            &[],
            self.audit_write_failures,
        );
        output
    }
}

#[derive(Clone, Copy)]
enum RouteLabel {
    Healthz,
    Metrics,
    VerifyAction,
    Other,
}

fn route_label(method: &str, path: &str) -> RouteLabel {
    match (method, path) {
        ("GET", "/healthz") => RouteLabel::Healthz,
        ("GET", "/metrics") => RouteLabel::Metrics,
        ("POST", "/verify/action") => RouteLabel::VerifyAction,
        _ => RouteLabel::Other,
    }
}

fn push_metric(output: &mut String, name: &str, labels: &[(&str, &str)], value: u64) {
    output.push_str(name);
    if !labels.is_empty() {
        output.push('{');
        for (index, (key, value)) in labels.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            output.push_str(key);
            output.push_str("=\"");
            output.push_str(value);
            output.push('"');
        }
        output.push('}');
    }
    output.push(' ');
    output.push_str(&value.to_string());
    output.push('\n');
}

#[derive(Debug, Serialize)]
struct AuditLogEntry<'a> {
    service: &'static str,
    action_id: &'a str,
    actor_id: &'a str,
    caller_id: Option<&'a str>,
    controller_id: &'a str,
    capability_id: &'a str,
    accepted: bool,
    rejection: Option<AuditLogRejection>,
    verified_at_unix: i64,
}

#[derive(Debug, Serialize)]
struct AuditLogRejection {
    code: String,
    subject: Option<String>,
}

fn append_audit_log(
    path: &Path,
    request: &VerifyActionRequest,
    result: &VerificationResult,
    now: OffsetDateTime,
    caller_id: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let rejection = match result {
        VerificationResult::Accepted => None,
        VerificationResult::Rejected(error) => Some(AuditLogRejection {
            code: error.code().to_owned(),
            subject: error.subject(),
        }),
    };
    let entry = AuditLogEntry {
        service: SERVICE_NAME,
        action_id: &request.action.id,
        actor_id: &request.action.actor,
        caller_id,
        controller_id: &request.action.controller,
        capability_id: &request.action.capability_id,
        accepted: result == &VerificationResult::Accepted,
        rejection,
        verified_at_unix: now.unix_timestamp(),
    };
    let mut file = open_audit_log(path)?;
    serde_json::to_writer(&mut file, &entry)?;
    writeln!(file)?;
    file.flush()?;
    file.sync_data()?;
    Ok(())
}

#[cfg(unix)]
fn open_audit_log(path: &Path) -> Result<std::fs::File, Box<dyn Error>> {
    use std::io;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    let mode = file.metadata()?.permissions().mode();
    if mode & 0o177 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "audit log file must be owner-only on Unix",
        )
        .into());
    }
    Ok(file)
}

#[cfg(not(unix))]
fn open_audit_log(path: &Path) -> Result<std::fs::File, Box<dyn Error>> {
    Ok(OpenOptions::new().create(true).append(true).open(path)?)
}

struct HttpRequest {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

fn read_http_request(
    stream: &mut TcpStream,
    max_request_bytes: usize,
    metrics: &mut MetricsState,
) -> Result<Option<HttpRequest>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    let header_end = loop {
        let mut chunk = [0_u8; 512];
        let count = stream.read(&mut chunk)?;
        if count == 0 {
            return Err("connection closed before headers completed".into());
        }
        bytes.extend_from_slice(&chunk[..count]);
        if bytes.len() > MAX_HEADER_BYTES {
            metrics.record_http(RouteLabel::Other, "431");
            write_json_response(
                stream,
                "431 Request Header Fields Too Large",
                &serde_json::json!({
                    "service": SERVICE_NAME,
                    "error": "request headers exceed max header bytes"
                }),
            )?;
            return Ok(None);
        }
        if let Some(position) = find_header_end(&bytes) {
            break position;
        }
    };

    let header_bytes = &bytes[..header_end];
    let headers = std::str::from_utf8(header_bytes)?;
    let mut lines = headers.split("\r\n");
    let request_line = lines.next().ok_or("missing HTTP request line")?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts
        .next()
        .ok_or("missing HTTP method")?
        .to_owned();
    let path = request_parts.next().ok_or("missing HTTP path")?.to_owned();
    let parsed_headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_owned(), value.trim().to_owned()))
        .collect::<Vec<_>>();
    let content_length = parsed_headers
        .iter()
        .find_map(|(name, value)| {
            if name.eq_ignore_ascii_case("content-length") {
                Some(value.parse::<usize>())
            } else {
                None
            }
        })
        .transpose()?
        .unwrap_or(0);

    if content_length > max_request_bytes {
        metrics.record_http(route_label(&method, &path), "413");
        write_json_response(
            stream,
            "413 Payload Too Large",
            &serde_json::json!({
                "service": SERVICE_NAME,
                "error": "request body exceeds max_request_bytes",
                "max_request_bytes": max_request_bytes,
            }),
        )?;
        return Ok(None);
    }

    let body_start = header_end + 4;
    let mut body = bytes[body_start..].to_vec();
    while body.len() < content_length {
        let mut chunk = vec![0_u8; content_length - body.len()];
        stream.read_exact(&mut chunk)?;
        body.extend_from_slice(&chunk);
    }
    body.truncate(content_length);

    Ok(Some(HttpRequest {
        method,
        path,
        headers: parsed_headers,
        body,
    }))
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn write_json_response<T: Serialize>(
    stream: &mut TcpStream,
    status: &str,
    body: &T,
) -> Result<(), Box<dyn Error>> {
    let body = serde_json::to_vec(body)?;
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.write_all(&body)?;
    Ok(())
}

fn write_text_response(
    stream: &mut TcpStream,
    status: &str,
    body: &str,
) -> Result<(), Box<dyn Error>> {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.write_all(body.as_bytes())?;
    Ok(())
}
