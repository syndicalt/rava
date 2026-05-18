use std::error::Error;
use std::io::BufRead;
use std::path::Path;

use serde_json::Value;

use crate::cli::ExportAuditArgs;

const RAW_PAYLOAD_FIELDS: &[&str] = &[
    "action",
    "capability_chain",
    "constraints",
    "intent",
    "proof",
    "resource",
];

pub fn run_audit_export(args: ExportAuditArgs) -> Result<(), Box<dyn Error>> {
    if let (Some(since), Some(until)) = (args.since_unix, args.until_unix) {
        if since > until {
            return Err("since-unix must be less than or equal to until-unix".into());
        }
    }
    let filter_by_time = args.since_unix.is_some() || args.until_unix.is_some();
    let file = std::fs::File::open(&args.audit_log)?;
    let reader = std::io::BufReader::new(file);
    let mut entries = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let entry: Value = serde_json::from_str(&line)?;
        reject_raw_payload_fields(&entry, index + 1)?;
        if filter_by_time && !entry_is_in_time_window(&entry, index + 1, &args)? {
            continue;
        }
        entries.push(entry);
    }

    if let Some(output) = &args.output {
        let mut file = create_export_file(output)?;
        serde_json::to_writer_pretty(&mut file, &entries)?;
        use std::io::Write;
        writeln!(file)?;
        file.sync_data()?;
    } else {
        serde_json::to_writer_pretty(std::io::stdout(), &entries)?;
        println!();
    }
    Ok(())
}

#[cfg(unix)]
fn create_export_file(path: &Path) -> Result<std::fs::File, Box<dyn Error>> {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    let mode = file.metadata()?.permissions().mode();
    if mode & 0o177 != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "audit export file must be owner-only on Unix",
        )
        .into());
    }
    file.set_len(0)?;
    Ok(file)
}

#[cfg(not(unix))]
fn create_export_file(path: &Path) -> Result<std::fs::File, Box<dyn Error>> {
    Ok(std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?)
}

fn entry_is_in_time_window(
    entry: &Value,
    line_number: usize,
    args: &ExportAuditArgs,
) -> Result<bool, Box<dyn Error>> {
    let verified_at_unix = entry
        .get("verified_at_unix")
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("audit entry missing verified_at_unix on line {line_number}"))?;

    if let Some(since) = args.since_unix {
        if verified_at_unix < since {
            return Ok(false);
        }
    }
    if let Some(until) = args.until_unix {
        if verified_at_unix > until {
            return Ok(false);
        }
    }

    Ok(true)
}

fn reject_raw_payload_fields(entry: &Value, line_number: usize) -> Result<(), Box<dyn Error>> {
    let Some(object) = entry.as_object() else {
        return Err(format!("audit entry on line {line_number} is not a JSON object").into());
    };

    for field in RAW_PAYLOAD_FIELDS {
        if object.contains_key(*field) {
            return Err(format!(
                "audit entry contains raw payload field {field:?} on line {line_number}"
            )
            .into());
        }
    }

    Ok(())
}
