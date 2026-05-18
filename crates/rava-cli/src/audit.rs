use std::error::Error;
use std::io::BufRead;

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
        entries.push(entry);
    }

    serde_json::to_writer_pretty(std::io::stdout(), &entries)?;
    println!();
    Ok(())
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
