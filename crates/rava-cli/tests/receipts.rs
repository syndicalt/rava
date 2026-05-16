use std::error::Error;
use std::fs;
use std::process::Command;

use rava_core::audit::{verify_verification_receipt, VerificationDecision, VerificationReceipt};

#[path = "support/scenario.rs"]
mod scenario;
#[path = "support/stdout.rs"]
mod stdout;
#[path = "support/temp.rs"]
mod temp;

use scenario::Scenario;
use stdout::stdout_line_value;

#[test]
fn verify_action_files_writes_verifiable_accepted_receipt() -> Result<(), Box<dyn Error>> {
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("accepted-receipt")?;
    let receipt_path = files.directory.join("receipt.json");

    let output = scenario.run_verify_action(&files, &[("--receipt-out", &receipt_path)])?;

    assert!(
        output.status.success(),
        "receipt verify failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Rava verification accepted: true\n"));
    let verifier_public_key = stdout_line_value(&stdout, "Rava receipt verifier public key: ")?;
    let receipt: VerificationReceipt = serde_json::from_slice(&fs::read(&receipt_path)?)?;

    assert_eq!(receipt.decision, VerificationDecision::Accepted);
    assert!(verify_verification_receipt(&receipt, verifier_public_key)?);
    Ok(())
}

#[test]
fn verify_action_files_writes_verifiable_rejected_receipt() -> Result<(), Box<dyn Error>> {
    let scenario = Scenario::new(900)?;
    let files = scenario.write_files("rejected-receipt")?;
    let receipt_path = files.directory.join("receipt.json");

    let output = scenario.run_verify_action(&files, &[("--receipt-out", &receipt_path)])?;

    assert!(
        output.status.success(),
        "receipt verify failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Rava verification accepted: false\n"));
    let verifier_public_key = stdout_line_value(&stdout, "Rava receipt verifier public key: ")?;
    let receipt: VerificationReceipt = serde_json::from_slice(&fs::read(&receipt_path)?)?;

    assert_eq!(receipt.decision, VerificationDecision::Rejected);
    assert!(verify_verification_receipt(&receipt, verifier_public_key)?);
    Ok(())
}

#[test]
fn key_file_can_sign_durable_verification_receipt() -> Result<(), Box<dyn Error>> {
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("durable-receipt")?;
    let key_path = files.directory.join("verifier-key.json");
    let receipt_path = files.directory.join("receipt.json");

    let keygen = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "key",
            "generate",
            "--kind",
            "service",
            "--out",
            key_path.to_str().ok_or("invalid key path")?,
        ])
        .output()?;
    assert!(
        keygen.status.success(),
        "key generation failed with stderr: {}",
        String::from_utf8_lossy(&keygen.stderr)
    );
    let keygen_stdout = String::from_utf8(keygen.stdout)?;
    assert!(keygen_stdout.contains("Rava key written: "));
    assert!(!keygen_stdout.contains("private_key_hex"));
    let key_file: serde_json::Value = serde_json::from_slice(&fs::read(&key_path)?)?;
    let verifier_id = key_file
        .get("id")
        .and_then(serde_json::Value::as_str)
        .ok_or("missing key id")?;
    let verifier_public_key = key_file
        .get("public_key_hex")
        .and_then(serde_json::Value::as_str)
        .ok_or("missing public key")?;
    assert!(key_file.get("private_key_hex").is_some());

    let output = scenario.run_verify_action(
        &files,
        &[
            ("--receipt-out", &receipt_path),
            ("--receipt-key", &key_path),
        ],
    )?;
    assert!(
        output.status.success(),
        "receipt verify failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains(&format!("Rava receipt verifier: {verifier_id}\n")));
    assert!(stdout.contains(&format!(
        "Rava receipt verifier public key: {verifier_public_key}\n"
    )));
    let receipt: VerificationReceipt = serde_json::from_slice(&fs::read(&receipt_path)?)?;
    assert_eq!(receipt.verifier, verifier_id);
    assert!(verify_verification_receipt(&receipt, verifier_public_key)?);
    Ok(())
}

#[test]
fn verify_receipt_file_accepts_valid_receipt() -> Result<(), Box<dyn Error>> {
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("verify-receipt")?;
    let receipt_path = files.directory.join("receipt.json");

    let output = scenario.run_verify_action(&files, &[("--receipt-out", &receipt_path)])?;
    assert!(
        output.status.success(),
        "receipt verify setup failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    let verifier_public_key = stdout_line_value(&stdout, "Rava receipt verifier public key: ")?;

    let receipt_output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "verify",
            "receipt",
            "--receipt",
            receipt_path.to_str().ok_or("invalid receipt path")?,
            "--verifier-key",
            verifier_public_key,
        ])
        .output()?;

    assert!(
        receipt_output.status.success(),
        "receipt verification command failed with stderr: {}",
        String::from_utf8_lossy(&receipt_output.stderr)
    );
    assert!(String::from_utf8(receipt_output.stdout)?.contains("Rava receipt verified: true\n"));
    Ok(())
}

#[test]
fn verify_receipt_file_reports_invalid_receipt_signature() -> Result<(), Box<dyn Error>> {
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("verify-tampered-receipt")?;
    let receipt_path = files.directory.join("receipt.json");

    let output = scenario.run_verify_action(&files, &[("--receipt-out", &receipt_path)])?;
    assert!(
        output.status.success(),
        "receipt verify setup failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    let verifier_public_key = stdout_line_value(&stdout, "Rava receipt verifier public key: ")?;
    let mut receipt: serde_json::Value = serde_json::from_slice(&fs::read(&receipt_path)?)?;
    receipt["decision"] = serde_json::json!("rejected");
    fs::write(&receipt_path, serde_json::to_vec_pretty(&receipt)?)?;

    let receipt_output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "verify",
            "receipt",
            "--receipt",
            receipt_path.to_str().ok_or("invalid receipt path")?,
            "--verifier-key",
            verifier_public_key,
        ])
        .output()?;

    assert!(
        receipt_output.status.success(),
        "invalid receipt should not be a CLI failure: {}",
        String::from_utf8_lossy(&receipt_output.stderr)
    );
    assert!(String::from_utf8(receipt_output.stdout)?.contains("Rava receipt verified: false\n"));
    Ok(())
}
