use std::error::Error;
use std::fs;
use std::process::Command;

#[path = "support/temp.rs"]
mod temp;

use temp::temp_directory;

#[test]
fn flight_booking_demo_accepts_delegated_action() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args(["demo", "flight-booking"])
        .output()?;

    assert!(
        output.status.success(),
        "demo failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Rava verification accepted: true\n"));
    assert!(stdout.contains("Rava replay rejected: true\n"));
    assert!(stdout.contains("Rava receipt verified: true\n"));
    Ok(())
}

#[test]
fn flight_booking_demo_writes_fixtures_that_file_verifier_accepts() -> Result<(), Box<dyn Error>> {
    let directory = temp_directory("fixtures")?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "demo",
            "flight-booking",
            "--write-fixtures",
            directory.to_str().ok_or("invalid fixture directory")?,
        ])
        .output()?;

    assert!(
        output.status.success(),
        "fixture demo failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Rava fixtures written: "));

    let action_path = directory.join("action.json");
    let chain_path = directory.join("capability-chain.json");
    let keys_path = directory.join("keys.json");
    let replay_path = directory.join("replay.json");
    let revocation_path = directory.join("revocations.json");
    let receipt_path = directory.join("receipt.json");

    let keys: serde_json::Value = serde_json::from_slice(&fs::read(&keys_path)?)?;
    assert!(keys.get("private_key_hex").is_none());
    assert!(keys.get("signing_key").is_none());
    let actor_key = keys
        .get("actor_public_key_hex")
        .and_then(serde_json::Value::as_str)
        .ok_or("missing actor_public_key_hex")?;
    let issuer_keys = keys
        .get("issuer_public_keys")
        .and_then(serde_json::Value::as_object)
        .ok_or("missing issuer_public_keys")?;
    let issuer_args: Vec<String> = issuer_keys
        .iter()
        .map(|(issuer, key)| {
            key.as_str()
                .map(|public_key| format!("{issuer}={public_key}"))
                .ok_or_else(|| format!("issuer key for {issuer} is not a string"))
        })
        .collect::<Result<_, _>>()?;

    let mut command = Command::new(env!("CARGO_BIN_EXE_rava"));
    command.args([
        "verify",
        "action",
        "--action",
        action_path.to_str().ok_or("invalid action path")?,
        "--capability-chain",
        chain_path.to_str().ok_or("invalid chain path")?,
        "--actor-key",
        actor_key,
        "--now-unix",
        "1650000000",
        "--replay-store",
        replay_path.to_str().ok_or("invalid replay path")?,
        "--revocation-store",
        revocation_path.to_str().ok_or("invalid revocation path")?,
        "--receipt-out",
        receipt_path.to_str().ok_or("invalid receipt path")?,
    ]);
    for issuer_arg in &issuer_args {
        command.args(["--issuer-key", issuer_arg]);
    }
    let verify_output = command.output()?;

    assert!(
        verify_output.status.success(),
        "fixture verify failed with stderr: {}",
        String::from_utf8_lossy(&verify_output.stderr)
    );
    assert!(String::from_utf8(verify_output.stdout)?.contains("Rava verification accepted: true\n"));
    assert!(receipt_path.exists());
    Ok(())
}

#[test]
fn flight_booking_demo_deterministically_regenerates_committed_corpus() -> Result<(), Box<dyn Error>>
{
    let directory = temp_directory("deterministic-fixtures")?;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let corpus = root.join("examples/flight-booking");

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "demo",
            "flight-booking",
            "--write-fixtures",
            directory.to_str().ok_or("invalid fixture directory")?,
            "--deterministic-fixtures",
        ])
        .output()?;

    assert!(
        output.status.success(),
        "deterministic fixture demo failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    for file_name in [
        "action.json",
        "attestation.json",
        "capability-chain.json",
        "keys.json",
        "receipt.json",
        "replay.json",
        "revocations.json",
        "tampered-attestation.json",
        "tampered-receipt.json",
    ] {
        assert_eq!(
            fs::read(directory.join(file_name))?,
            fs::read(corpus.join(file_name))?,
            "{file_name} was not regenerated deterministically"
        );
    }

    Ok(())
}

#[test]
fn deterministic_fixtures_require_output_directory() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args(["demo", "flight-booking", "--deterministic-fixtures"])
        .output()?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("--deterministic-fixtures requires --write-fixtures"));
    Ok(())
}
