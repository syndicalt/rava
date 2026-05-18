use std::error::Error;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn audit_export_writes_metadata_entries_as_json_array() -> Result<(), Box<dyn Error>> {
    let audit_log = temp_file_path("rava-audit-export");
    std::fs::write(
        &audit_log,
        concat!(
            r#"{"service":"rava-verifier-preview-v0","action_id":"action-1","actor_id":"agent-a","caller_id":"tenant-a","controller_id":"human-a","capability_id":"cap-1","accepted":true,"rejection":null,"verified_at_unix":1650000000}"#,
            "\n",
            r#"{"service":"rava-verifier-preview-v0","action_id":"action-2","actor_id":"agent-a","caller_id":"tenant-a","controller_id":"human-a","capability_id":"cap-1","accepted":false,"rejection":{"code":"action_replayed","subject":"action-2"},"verified_at_unix":1650000001}"#,
            "\n"
        ),
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args(["audit", "export", "--audit-log"])
        .arg(&audit_log)
        .output()?;

    assert!(output.status.success(), "{}", stderr(&output));
    let exported: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let entries = exported.as_array().ok_or("export output is not an array")?;
    assert_eq!(entries.len(), 2);
    assert_eq!(
        entries[0]
            .get("caller_id")
            .and_then(serde_json::Value::as_str),
        Some("tenant-a")
    );
    assert_eq!(
        entries[1]
            .pointer("/rejection/code")
            .and_then(serde_json::Value::as_str),
        Some("action_replayed")
    );

    std::fs::remove_file(audit_log)?;
    Ok(())
}

#[test]
fn audit_export_rejects_entries_with_raw_payload_fields() -> Result<(), Box<dyn Error>> {
    let audit_log = temp_file_path("rava-audit-export-raw");
    std::fs::write(
        &audit_log,
        r#"{"service":"rava-verifier-preview-v0","action_id":"action-1","actor_id":"agent-a","controller_id":"human-a","capability_id":"cap-1","accepted":true,"rejection":null,"verified_at_unix":1650000000,"intent":"book_flight"}"#,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args(["audit", "export", "--audit-log"])
        .arg(&audit_log)
        .output()?;

    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("audit entry contains raw payload field"),
        "{}",
        stderr(&output)
    );

    std::fs::remove_file(audit_log)?;
    Ok(())
}

#[test]
fn audit_export_filters_entries_by_verified_at_unix_bounds() -> Result<(), Box<dyn Error>> {
    let audit_log = temp_file_path("rava-audit-export-window");
    std::fs::write(
        &audit_log,
        concat!(
            r#"{"service":"rava-verifier-preview-v0","action_id":"before","actor_id":"agent-a","caller_id":"tenant-a","controller_id":"human-a","capability_id":"cap-1","accepted":true,"rejection":null,"verified_at_unix":1650000000}"#,
            "\n",
            r#"{"service":"rava-verifier-preview-v0","action_id":"inside","actor_id":"agent-a","caller_id":"tenant-a","controller_id":"human-a","capability_id":"cap-1","accepted":false,"rejection":{"code":"action_replayed","subject":"inside"},"verified_at_unix":1650000001}"#,
            "\n",
            r#"{"service":"rava-verifier-preview-v0","action_id":"after","actor_id":"agent-a","caller_id":"tenant-a","controller_id":"human-a","capability_id":"cap-1","accepted":true,"rejection":null,"verified_at_unix":1650000002}"#,
            "\n"
        ),
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "audit",
            "export",
            "--audit-log",
            audit_log.to_str().ok_or("invalid audit path")?,
            "--since-unix",
            "1650000001",
            "--until-unix",
            "1650000001",
        ])
        .output()?;

    assert!(output.status.success(), "{}", stderr(&output));
    let exported: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let entries = exported.as_array().ok_or("export output is not an array")?;
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0]
            .get("action_id")
            .and_then(serde_json::Value::as_str),
        Some("inside")
    );

    std::fs::remove_file(audit_log)?;
    Ok(())
}

#[test]
fn audit_export_rejects_entries_without_timestamp_when_filtering() -> Result<(), Box<dyn Error>> {
    let audit_log = temp_file_path("rava-audit-export-missing-time");
    std::fs::write(
        &audit_log,
        r#"{"service":"rava-verifier-preview-v0","action_id":"action-1","actor_id":"agent-a","controller_id":"human-a","capability_id":"cap-1","accepted":true,"rejection":null}"#,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "audit",
            "export",
            "--audit-log",
            audit_log.to_str().ok_or("invalid audit path")?,
            "--since-unix",
            "1650000000",
        ])
        .output()?;

    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("missing verified_at_unix"),
        "{}",
        stderr(&output)
    );

    std::fs::remove_file(audit_log)?;
    Ok(())
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn temp_file_path(prefix: &str) -> PathBuf {
    let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{counter}.ndjson", std::process::id()))
}
