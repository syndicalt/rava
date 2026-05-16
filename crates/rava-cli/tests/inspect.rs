use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

#[path = "support/temp.rs"]
mod temp;

use rava_core::action::ActionEnvelope;
use rava_core::capability::Capability;

#[test]
fn inspect_action_summarizes_signed_action_without_verifying() -> Result<(), Box<dyn Error>> {
    let action_path = example_path("action.json");
    let action: ActionEnvelope = serde_json::from_slice(&std::fs::read(&action_path)?)?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "inspect",
            "action",
            "--action",
            action_path.to_str().ok_or("invalid action path")?,
        ])
        .output()?;

    assert!(
        output.status.success(),
        "inspect action failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Rava inspection only: true\n"));
    assert!(stdout.contains("Rava object: action\n"));
    assert!(stdout.contains(&format!("Rava action id: {}\n", action.id)));
    assert!(stdout.contains(&format!("Rava actor: {}\n", action.actor)));
    assert!(stdout.contains(&format!("Rava controller: {}\n", action.controller)));
    assert!(stdout.contains("Rava intent: book_flight\n"));
    assert!(stdout.contains("Rava resource: travel.booking\n"));
    assert!(stdout.contains("Rava operation: purchase\n"));
    assert!(stdout.contains(&format!("Rava capability id: {}\n", action.capability_id)));
    assert!(!stdout.contains("Rava verification accepted:"));
    Ok(())
}

#[test]
fn inspect_capability_chain_summarizes_chain_without_verifying() -> Result<(), Box<dyn Error>> {
    let chain_path = example_path("capability-chain.json");
    let chain: Vec<Capability> = serde_json::from_slice(&std::fs::read(&chain_path)?)?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "inspect",
            "capability-chain",
            "--capability-chain",
            chain_path.to_str().ok_or("invalid chain path")?,
        ])
        .output()?;

    assert!(
        output.status.success(),
        "inspect capability chain failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    let root = chain.first().ok_or("missing root capability")?;
    let final_capability = chain.last().ok_or("missing final capability")?;
    assert!(stdout.contains("Rava inspection only: true\n"));
    assert!(stdout.contains("Rava object: capability-chain\n"));
    assert!(stdout.contains("Rava capability count: 2\n"));
    assert!(stdout.contains(&format!("Rava root issuer: {}\n", root.issuer)));
    assert!(stdout.contains(&format!(
        "Rava final subject: {}\n",
        final_capability.subject
    )));
    assert!(stdout.contains("Rava final resource: travel.booking\n"));
    assert!(stdout.contains("Rava final operations: purchase\n"));
    assert!(!stdout.contains("Rava verification accepted:"));
    Ok(())
}

#[test]
fn inspect_action_rejects_malformed_json_as_cli_error() -> Result<(), Box<dyn Error>> {
    let directory = temp::temp_directory("inspect-malformed")?;
    std::fs::create_dir_all(&directory)?;
    let action_path = directory.join("action.json");
    std::fs::write(&action_path, b"not-json")?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "inspect",
            "action",
            "--action",
            action_path.to_str().ok_or("invalid action path")?,
        ])
        .output()?;

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)?.contains("rava:"));
    Ok(())
}

fn example_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples")
        .join("flight-booking")
        .join(file_name)
}
