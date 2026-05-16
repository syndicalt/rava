use std::error::Error;
use std::fs;
use std::process::Command;

#[path = "support/temp.rs"]
mod temp;

use temp::temp_directory;

#[test]
fn attestation_file_can_be_signed_and_verified() -> Result<(), Box<dyn Error>> {
    let directory = temp_directory("attestation")?;
    fs::create_dir_all(&directory)?;
    let key_path = directory.join("evaluator-key.json");
    let attestation_path = directory.join("attestation.json");

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
    let key_file: serde_json::Value = serde_json::from_slice(&fs::read(&key_path)?)?;
    let evaluator_public_key = key_file
        .get("public_key_hex")
        .and_then(serde_json::Value::as_str)
        .ok_or("missing public key")?;

    let sign = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "attest",
            "sign",
            "--key",
            key_path.to_str().ok_or("invalid key path")?,
            "--out",
            attestation_path
                .to_str()
                .ok_or("invalid attestation path")?,
            "--action-id",
            "act_demo",
            "--outcome",
            "accepted",
            "--subject",
            "travel.booking",
            "--occurred-at-unix",
            "1650000000",
            "--evidence-hash",
            "sha256:3333333333333333333333333333333333333333333333333333333333333333",
        ])
        .output()?;
    assert!(
        sign.status.success(),
        "attestation signing failed with stderr: {}",
        String::from_utf8_lossy(&sign.stderr)
    );
    assert!(String::from_utf8(sign.stdout)?.contains("Rava attestation written: "));

    let verify = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "verify",
            "attestation",
            "--attestation",
            attestation_path
                .to_str()
                .ok_or("invalid attestation path")?,
            "--evaluator-key",
            evaluator_public_key,
        ])
        .output()?;
    assert!(
        verify.status.success(),
        "attestation verification failed with stderr: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
    assert!(String::from_utf8(verify.stdout)?.contains("Rava attestation verified: true\n"));
    Ok(())
}

#[test]
fn verify_attestation_file_reports_invalid_signature() -> Result<(), Box<dyn Error>> {
    let directory = temp_directory("attestation-tamper")?;
    fs::create_dir_all(&directory)?;
    let key_path = directory.join("evaluator-key.json");
    let attestation_path = directory.join("attestation.json");

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
    assert!(keygen.status.success());
    let key_file: serde_json::Value = serde_json::from_slice(&fs::read(&key_path)?)?;
    let evaluator_public_key = key_file
        .get("public_key_hex")
        .and_then(serde_json::Value::as_str)
        .ok_or("missing public key")?;

    let sign = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "attest",
            "sign",
            "--key",
            key_path.to_str().ok_or("invalid key path")?,
            "--out",
            attestation_path
                .to_str()
                .ok_or("invalid attestation path")?,
            "--action-id",
            "act_demo",
            "--outcome",
            "accepted",
            "--subject",
            "travel.booking",
            "--occurred-at-unix",
            "1650000000",
            "--evidence-hash",
            "sha256:3333333333333333333333333333333333333333333333333333333333333333",
        ])
        .output()?;
    assert!(sign.status.success());
    let mut attestation: serde_json::Value = serde_json::from_slice(&fs::read(&attestation_path)?)?;
    attestation["outcome"] = serde_json::json!("rejected");
    fs::write(&attestation_path, serde_json::to_vec_pretty(&attestation)?)?;

    let verify = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "verify",
            "attestation",
            "--attestation",
            attestation_path
                .to_str()
                .ok_or("invalid attestation path")?,
            "--evaluator-key",
            evaluator_public_key,
        ])
        .output()?;

    assert!(
        verify.status.success(),
        "invalid attestation should not be a CLI failure: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
    assert!(String::from_utf8(verify.stdout)?.contains("Rava attestation verified: false\n"));
    Ok(())
}
