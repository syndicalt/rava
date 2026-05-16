use std::error::Error;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn examples_corpus_is_verifiable() -> Result<(), Box<dyn Error>> {
    let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples/flight-booking");
    let keys: serde_json::Value =
        serde_json::from_slice(&std::fs::read(corpus.join("keys.json"))?)?;
    assert!(keys.get("private_key_hex").is_none());
    let actor_key = keys
        .get("actor_public_key_hex")
        .and_then(serde_json::Value::as_str)
        .ok_or("missing actor_public_key_hex")?;
    let issuer_keys = keys
        .get("issuer_public_keys")
        .and_then(serde_json::Value::as_object)
        .ok_or("missing issuer_public_keys")?;
    let verifier_key = keys
        .get("verifier_public_key_hex")
        .and_then(serde_json::Value::as_str)
        .ok_or("missing verifier_public_key_hex")?;
    let evaluator_key = keys
        .get("evaluator_public_key_hex")
        .and_then(serde_json::Value::as_str)
        .ok_or("missing evaluator_public_key_hex")?;
    let temp = temp_directory()?;
    std::fs::create_dir_all(&temp)?;
    let replay_store = temp.join("replay.json");
    let revocation_store = temp.join("revocations.json");
    std::fs::copy(corpus.join("replay.json"), &replay_store)?;
    std::fs::copy(corpus.join("revocations.json"), &revocation_store)?;

    let mut verify_action = Command::new(env!("CARGO_BIN_EXE_rava"));
    verify_action.args([
        "verify",
        "action",
        "--action",
        path_str(&corpus.join("action.json"))?,
        "--capability-chain",
        path_str(&corpus.join("capability-chain.json"))?,
        "--actor-key",
        actor_key,
        "--now-unix",
        "1650000000",
        "--replay-store",
        path_str(&replay_store)?,
        "--revocation-store",
        path_str(&revocation_store)?,
    ]);
    for (issuer, key) in issuer_keys {
        verify_action.args([
            "--issuer-key",
            &format!(
                "{}={}",
                issuer,
                key.as_str().ok_or("issuer key is not a string")?
            ),
        ]);
    }
    let action_output = verify_action.output()?;
    assert!(
        action_output.status.success(),
        "example action verification failed with stderr: {}",
        String::from_utf8_lossy(&action_output.stderr)
    );
    assert!(String::from_utf8(action_output.stdout)?.contains("Rava verification accepted: true\n"));

    assert_cli_bool(
        Command::new(env!("CARGO_BIN_EXE_rava")).args([
            "verify",
            "receipt",
            "--receipt",
            path_str(&corpus.join("receipt.json"))?,
            "--verifier-key",
            verifier_key,
        ]),
        "Rava receipt verified: true\n",
    )?;
    assert_cli_bool(
        Command::new(env!("CARGO_BIN_EXE_rava")).args([
            "verify",
            "receipt",
            "--receipt",
            path_str(&corpus.join("tampered-receipt.json"))?,
            "--verifier-key",
            verifier_key,
        ]),
        "Rava receipt verified: false\n",
    )?;
    assert_cli_bool(
        Command::new(env!("CARGO_BIN_EXE_rava")).args([
            "verify",
            "attestation",
            "--attestation",
            path_str(&corpus.join("attestation.json"))?,
            "--evaluator-key",
            evaluator_key,
        ]),
        "Rava attestation verified: true\n",
    )?;
    assert_cli_bool(
        Command::new(env!("CARGO_BIN_EXE_rava")).args([
            "verify",
            "attestation",
            "--attestation",
            path_str(&corpus.join("tampered-attestation.json"))?,
            "--evaluator-key",
            evaluator_key,
        ]),
        "Rava attestation verified: false\n",
    )?;

    Ok(())
}

fn assert_cli_bool(command: &mut Command, expected_stdout: &str) -> Result<(), Box<dyn Error>> {
    let output = command.output()?;
    assert!(
        output.status.success(),
        "command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8(output.stdout)?.contains(expected_stdout));
    Ok(())
}

fn path_str(path: &std::path::Path) -> Result<&str, Box<dyn Error>> {
    path.to_str()
        .ok_or_else(|| "path is not valid UTF-8".into())
}

fn temp_directory() -> Result<std::path::PathBuf, Box<dyn Error>> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(std::env::temp_dir().join(format!(
        "rava-examples-corpus-{}-{nonce}",
        std::process::id()
    )))
}
