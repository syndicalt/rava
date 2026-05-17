use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{Read, Write};
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
    let listener = TcpListener::bind(&args.addr)?;
    let mut rate_limit = args.rate_limit_per_minute.map(RateLimitState::new);
    println!(
        "Rava verifier service listening: {}",
        listener.local_addr()?
    );

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) = handle_connection(&mut stream, &args, &mut rate_limit) {
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

fn handle_connection(
    stream: &mut TcpStream,
    args: &ServeVerifyArgs,
    rate_limit: &mut Option<RateLimitState>,
) -> Result<(), Box<dyn Error>> {
    let Some(request) = read_http_request(stream, args.max_request_bytes)? else {
        return Ok(());
    };
    if !request_is_authorized(&request, args)? {
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
            write_json_response(
                stream,
                "429 Too Many Requests",
                &serde_json::json!({
                    "service": SERVICE_NAME,
                    "error": "rate limit exceeded",
                    "rate_limit_per_minute": rate_limit.limit,
                }),
            )?;
            return Ok(());
        }
    }
    if request.method == "GET" && request.path == "/healthz" {
        write_json_response(
            stream,
            "200 OK",
            &serde_json::json!({
                "service": SERVICE_NAME,
                "status": "ok",
                "max_request_bytes": args.max_request_bytes,
                "replay_store_configured": args.replay_store.is_some(),
                "revocation_store_configured": args.revocation_store.is_some(),
                "audit_log_configured": args.audit_log.is_some(),
                "auth_required": args.auth_token_env.is_some(),
                "rate_limit_per_minute": args.rate_limit_per_minute,
            }),
        )?;
        return Ok(());
    }
    if request.method != "POST" || request.path != "/verify/action" {
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
    let result = if let Some(revocation_store) = &args.revocation_store {
        let revocations = FileRevocationRegistry::open(revocation_store)?;
        verify_action_with_optional_replay(&request, &revocations, args, now)?
    } else {
        let revocations = InMemoryRevocationRegistry::default();
        verify_action_with_optional_replay(&request, &revocations, args, now)?
    };
    if let Some(audit_log) = &args.audit_log {
        append_audit_log(audit_log, &request, &result, now)?;
    }
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

#[derive(Debug, Serialize)]
struct AuditLogEntry<'a> {
    service: &'static str,
    action_id: &'a str,
    actor_id: &'a str,
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
        controller_id: &request.action.controller,
        capability_id: &request.action.capability_id,
        accepted: result == &VerificationResult::Accepted,
        rejection,
        verified_at_unix: now.unix_timestamp(),
    };
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, &entry)?;
    writeln!(file)?;
    Ok(())
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
