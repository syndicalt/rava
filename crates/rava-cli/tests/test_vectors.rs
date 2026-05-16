use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn v0_test_vectors_are_language_neutral_and_verifiable() -> Result<(), Box<dyn Error>> {
    let root = repository_root().join("test-vectors/v0");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json"))?)?;

    assert_eq!(
        manifest
            .get("schema_version")
            .and_then(serde_json::Value::as_str),
        Some("rava-test-vectors-v0")
    );
    let vectors = manifest
        .get("vectors")
        .and_then(serde_json::Value::as_array)
        .ok_or("manifest vectors must be an array")?;
    assert!(!vectors.is_empty());

    for vector in vectors {
        assert_eq!(
            vector.get("kind").and_then(serde_json::Value::as_str),
            Some("flight-booking-accepted")
        );
        verify_flight_booking_vector(&root, vector)?;
    }

    Ok(())
}

fn verify_flight_booking_vector(
    root: &Path,
    vector: &serde_json::Value,
) -> Result<(), Box<dyn Error>> {
    let files = vector
        .get("files")
        .and_then(serde_json::Value::as_object)
        .ok_or("vector files must be an object")?;
    let action_path = root.join(file(files, "action")?);
    let chain_path = root.join(file(files, "capability_chain")?);
    let keys_path = root.join(file(files, "keys")?);
    let replay_path = root.join(file(files, "replay_store")?);
    let revocation_path = root.join(file(files, "revocation_store")?);
    let receipt_path = root.join(file(files, "receipt")?);
    let attestation_path = root.join(file(files, "attestation")?);
    let tampered_receipt_path = root.join(file(files, "tampered_receipt")?);
    let tampered_attestation_path = root.join(file(files, "tampered_attestation")?);

    for path in [
        &action_path,
        &chain_path,
        &keys_path,
        &replay_path,
        &revocation_path,
        &receipt_path,
        &attestation_path,
        &tampered_receipt_path,
        &tampered_attestation_path,
    ] {
        assert!(path.exists(), "missing vector file: {}", path.display());
    }

    let keys: serde_json::Value = serde_json::from_slice(&std::fs::read(keys_path)?)?;
    assert!(keys.get("private_key_hex").is_none());
    assert!(keys.get("signing_key").is_none());
    let actor_key = string_field(&keys, "actor_public_key_hex")?;
    let verifier_key = string_field(&keys, "verifier_public_key_hex")?;
    let evaluator_key = string_field(&keys, "evaluator_public_key_hex")?;
    let issuer_keys = keys
        .get("issuer_public_keys")
        .and_then(serde_json::Value::as_object)
        .ok_or("issuer_public_keys must be an object")?;
    let temp = temp_directory()?;
    std::fs::create_dir_all(&temp)?;
    let replay_copy = temp.join("replay.json");
    let revocation_copy = temp.join("revocations.json");
    std::fs::copy(replay_path, &replay_copy)?;
    std::fs::copy(revocation_path, &revocation_copy)?;

    let mut verify_action = Command::new(env!("CARGO_BIN_EXE_rava"));
    verify_action.args([
        "verify",
        "action",
        "--action",
        path_str(&action_path)?,
        "--capability-chain",
        path_str(&chain_path)?,
        "--actor-key",
        actor_key,
        "--now-unix",
        "1650000000",
        "--replay-store",
        path_str(&replay_copy)?,
        "--revocation-store",
        path_str(&revocation_copy)?,
    ]);
    for (issuer, key) in issuer_keys {
        verify_action.args([
            "--issuer-key",
            &format!(
                "{}={}",
                issuer,
                key.as_str().ok_or("issuer key must be a string")?
            ),
        ]);
    }
    assert_cli_contains(&mut verify_action, "Rava verification accepted: true\n")?;

    assert_cli_contains(
        Command::new(env!("CARGO_BIN_EXE_rava")).args([
            "verify",
            "receipt",
            "--receipt",
            path_str(&receipt_path)?,
            "--verifier-key",
            verifier_key,
        ]),
        "Rava receipt verified: true\n",
    )?;
    assert_cli_contains(
        Command::new(env!("CARGO_BIN_EXE_rava")).args([
            "verify",
            "receipt",
            "--receipt",
            path_str(&tampered_receipt_path)?,
            "--verifier-key",
            verifier_key,
        ]),
        "Rava receipt verified: false\n",
    )?;
    assert_cli_contains(
        Command::new(env!("CARGO_BIN_EXE_rava")).args([
            "verify",
            "attestation",
            "--attestation",
            path_str(&attestation_path)?,
            "--evaluator-key",
            evaluator_key,
        ]),
        "Rava attestation verified: true\n",
    )?;
    assert_cli_contains(
        Command::new(env!("CARGO_BIN_EXE_rava")).args([
            "verify",
            "attestation",
            "--attestation",
            path_str(&tampered_attestation_path)?,
            "--evaluator-key",
            evaluator_key,
        ]),
        "Rava attestation verified: false\n",
    )?;

    Ok(())
}

fn file<'a>(
    files: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<&'a str, Box<dyn Error>> {
    files
        .get(key)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("missing vector file entry {key:?}").into())
}

fn string_field<'a>(value: &'a serde_json::Value, key: &str) -> Result<&'a str, Box<dyn Error>> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("missing string field {key:?}").into())
}

fn assert_cli_contains(command: &mut Command, expected_stdout: &str) -> Result<(), Box<dyn Error>> {
    let output = command.output()?;
    assert!(
        output.status.success(),
        "command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8(output.stdout)?.contains(expected_stdout));
    Ok(())
}

fn path_str(path: &Path) -> Result<&str, Box<dyn Error>> {
    path.to_str()
        .ok_or_else(|| "path is not valid UTF-8".into())
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn temp_directory() -> Result<PathBuf, Box<dyn Error>> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(std::env::temp_dir().join(format!("rava-test-vectors-{}-{nonce}", std::process::id())))
}
