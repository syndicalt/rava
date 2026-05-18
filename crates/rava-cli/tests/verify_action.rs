use std::error::Error;
use std::fs;
use std::process::Command;

#[path = "support/scenario.rs"]
mod scenario;
#[path = "support/temp.rs"]
mod temp;

use scenario::Scenario;

#[test]
fn verify_action_files_accepts_valid_signed_action() -> Result<(), Box<dyn Error>> {
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("valid")?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "verify",
            "action",
            "--action",
            files.action_path.to_str().ok_or("invalid action path")?,
            "--capability-chain",
            files.chain_path.to_str().ok_or("invalid chain path")?,
            "--actor-key",
            &scenario.booking_agent.public_key_hex,
            "--issuer-key",
            &format!("{}={}", scenario.human.id, scenario.human.public_key_hex),
            "--issuer-key",
            &format!(
                "{}={}",
                scenario.personal_agent.id, scenario.personal_agent.public_key_hex
            ),
            "--now-unix",
            "1650000000",
        ])
        .output()?;

    assert!(
        output.status.success(),
        "verify failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Rava verification accepted: true\n"));
    Ok(())
}

#[test]
fn verify_action_files_accepts_static_trust_bundle() -> Result<(), Box<dyn Error>> {
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("trust-bundle")?;
    let trust_bundle_path = files.directory.join("trust-bundle.json");
    fs::write(
        &trust_bundle_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": "rava-static-trust-bundle-v0",
            "keys": {
                scenario.booking_agent.id: scenario.booking_agent.public_key_hex,
                scenario.human.id: scenario.human.public_key_hex,
                scenario.personal_agent.id: scenario.personal_agent.public_key_hex,
            }
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "verify",
            "action",
            "--action",
            files.action_path.to_str().ok_or("invalid action path")?,
            "--capability-chain",
            files.chain_path.to_str().ok_or("invalid chain path")?,
            "--trust-bundle",
            trust_bundle_path
                .to_str()
                .ok_or("invalid trust bundle path")?,
            "--now-unix",
            "1650000000",
        ])
        .output()?;

    assert!(
        output.status.success(),
        "verify failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8(output.stdout)?.contains("Rava verification accepted: true\n"));
    Ok(())
}

#[test]
fn verify_action_requires_fresh_static_trust_bundle_when_configured() -> Result<(), Box<dyn Error>>
{
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("trust-bundle-fresh")?;
    let trust_bundle_path = files.directory.join("trust-bundle.json");
    fs::write(
        &trust_bundle_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": "rava-static-trust-bundle-v0",
            "fresh_until_unix": 1650000001,
            "keys": {
                scenario.booking_agent.id: scenario.booking_agent.public_key_hex,
                scenario.human.id: scenario.human.public_key_hex,
                scenario.personal_agent.id: scenario.personal_agent.public_key_hex,
            }
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "verify",
            "action",
            "--action",
            files.action_path.to_str().ok_or("invalid action path")?,
            "--capability-chain",
            files.chain_path.to_str().ok_or("invalid chain path")?,
            "--trust-bundle",
            trust_bundle_path
                .to_str()
                .ok_or("invalid trust bundle path")?,
            "--now-unix",
            "1650000000",
            "--require-fresh-trust-bundle",
        ])
        .output()?;

    assert!(
        output.status.success(),
        "fresh trust bundle should allow verification: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8(output.stdout)?.contains("Rava verification accepted: true\n"));
    Ok(())
}

#[test]
fn verify_action_rejects_stale_static_trust_bundle_when_freshness_is_required(
) -> Result<(), Box<dyn Error>> {
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("trust-bundle-stale")?;
    let trust_bundle_path = files.directory.join("trust-bundle.json");
    fs::write(
        &trust_bundle_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": "rava-static-trust-bundle-v0",
            "fresh_until_unix": 1650000000,
            "keys": {
                scenario.booking_agent.id: scenario.booking_agent.public_key_hex,
                scenario.human.id: scenario.human.public_key_hex,
                scenario.personal_agent.id: scenario.personal_agent.public_key_hex,
            }
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "verify",
            "action",
            "--action",
            files.action_path.to_str().ok_or("invalid action path")?,
            "--capability-chain",
            files.chain_path.to_str().ok_or("invalid chain path")?,
            "--trust-bundle",
            trust_bundle_path
                .to_str()
                .ok_or("invalid trust bundle path")?,
            "--now-unix",
            "1650000000",
            "--require-fresh-trust-bundle",
        ])
        .output()?;

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)?.contains("trust bundle is stale"));
    Ok(())
}

#[test]
fn verify_action_rejects_missing_static_trust_bundle_freshness_when_required(
) -> Result<(), Box<dyn Error>> {
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("trust-bundle-missing-freshness")?;
    let trust_bundle_path = files.directory.join("trust-bundle.json");
    fs::write(
        &trust_bundle_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": "rava-static-trust-bundle-v0",
            "keys": {
                scenario.booking_agent.id: scenario.booking_agent.public_key_hex,
                scenario.human.id: scenario.human.public_key_hex,
                scenario.personal_agent.id: scenario.personal_agent.public_key_hex,
            }
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "verify",
            "action",
            "--action",
            files.action_path.to_str().ok_or("invalid action path")?,
            "--capability-chain",
            files.chain_path.to_str().ok_or("invalid chain path")?,
            "--trust-bundle",
            trust_bundle_path
                .to_str()
                .ok_or("invalid trust bundle path")?,
            "--now-unix",
            "1650000000",
            "--require-fresh-trust-bundle",
        ])
        .output()?;

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)?.contains("trust bundle missing fresh_until_unix"));
    Ok(())
}

#[test]
fn verify_action_files_rejects_conflicting_static_trust_bundle_key() -> Result<(), Box<dyn Error>> {
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("trust-bundle-conflict")?;
    let trust_bundle_path = files.directory.join("trust-bundle.json");
    fs::write(
        &trust_bundle_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": "rava-static-trust-bundle-v0",
            "keys": {
                scenario.booking_agent.id: scenario.human.public_key_hex,
                scenario.human.id: scenario.human.public_key_hex,
                scenario.personal_agent.id: scenario.personal_agent.public_key_hex,
            }
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "verify",
            "action",
            "--action",
            files.action_path.to_str().ok_or("invalid action path")?,
            "--capability-chain",
            files.chain_path.to_str().ok_or("invalid chain path")?,
            "--actor-key",
            &scenario.booking_agent.public_key_hex,
            "--trust-bundle",
            trust_bundle_path
                .to_str()
                .ok_or("invalid trust bundle path")?,
            "--now-unix",
            "1650000000",
        ])
        .output()?;

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)?.contains("rava:"));
    Ok(())
}

#[test]
fn verify_action_files_reports_rejected_signed_action() -> Result<(), Box<dyn Error>> {
    let scenario = Scenario::new(900)?;
    let files = scenario.write_files("over-budget")?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "verify",
            "action",
            "--action",
            files.action_path.to_str().ok_or("invalid action path")?,
            "--capability-chain",
            files.chain_path.to_str().ok_or("invalid chain path")?,
            "--actor-key",
            &scenario.booking_agent.public_key_hex,
            "--issuer-key",
            &format!("{}={}", scenario.human.id, scenario.human.public_key_hex),
            "--issuer-key",
            &format!(
                "{}={}",
                scenario.personal_agent.id, scenario.personal_agent.public_key_hex
            ),
            "--now-unix",
            "1650000000",
        ])
        .output()?;

    assert!(
        output.status.success(),
        "authorization rejection should not be a CLI failure: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Rava verification accepted: false\n"));
    assert!(stdout.contains("Rava rejection: ConstraintExceeded"));
    Ok(())
}

#[test]
fn verify_action_files_rejects_missing_issuer_key() -> Result<(), Box<dyn Error>> {
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("missing-issuer")?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "verify",
            "action",
            "--action",
            files.action_path.to_str().ok_or("invalid action path")?,
            "--capability-chain",
            files.chain_path.to_str().ok_or("invalid chain path")?,
            "--actor-key",
            &scenario.booking_agent.public_key_hex,
            "--issuer-key",
            &format!("{}={}", scenario.human.id, scenario.human.public_key_hex),
            "--now-unix",
            "1650000000",
        ])
        .output()?;

    assert!(
        output.status.success(),
        "missing issuer public key is a verification rejection, not a CLI failure: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Rava verification accepted: false\n"));
    assert!(stdout.contains("Rava rejection: MissingIssuerPublicKey"));
    Ok(())
}

#[test]
fn verify_action_files_rejects_malformed_json_as_cli_error() -> Result<(), Box<dyn Error>> {
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("malformed-json")?;
    fs::write(&files.action_path, b"not-json")?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "verify",
            "action",
            "--action",
            files.action_path.to_str().ok_or("invalid action path")?,
            "--capability-chain",
            files.chain_path.to_str().ok_or("invalid chain path")?,
            "--actor-key",
            &scenario.booking_agent.public_key_hex,
            "--issuer-key",
            &format!("{}={}", scenario.human.id, scenario.human.public_key_hex),
            "--issuer-key",
            &format!(
                "{}={}",
                scenario.personal_agent.id, scenario.personal_agent.public_key_hex
            ),
            "--now-unix",
            "1650000000",
        ])
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("rava:"));
    Ok(())
}

#[test]
fn verify_action_files_with_replay_store_rejects_second_accepted_action(
) -> Result<(), Box<dyn Error>> {
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("replay")?;
    let replay_path = files.directory.join("replay.json");

    let first = scenario.run_verify_action(&files, &[("--replay-store", &replay_path)])?;
    assert!(
        first.status.success(),
        "first verify failed with stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(String::from_utf8(first.stdout)?.contains("Rava verification accepted: true\n"));

    let second = scenario.run_verify_action(&files, &[("--replay-store", &replay_path)])?;
    assert!(
        second.status.success(),
        "replay rejection should not be a CLI failure: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    let stdout = String::from_utf8(second.stdout)?;
    assert!(stdout.contains("Rava verification accepted: false\n"));
    assert!(stdout.contains("Rava rejection: ActionReplayed"));
    Ok(())
}

#[test]
fn verify_action_files_with_replay_store_does_not_record_rejected_action(
) -> Result<(), Box<dyn Error>> {
    let rejected_scenario = Scenario::new(900)?;
    let rejected_files = rejected_scenario.write_files("replay-rejected")?;
    let replay_path = rejected_files.directory.join("replay.json");

    let rejected = rejected_scenario
        .run_verify_action(&rejected_files, &[("--replay-store", &replay_path)])?;
    assert!(
        rejected.status.success(),
        "authorization rejection should not be a CLI failure: {}",
        String::from_utf8_lossy(&rejected.stderr)
    );
    assert!(String::from_utf8(rejected.stdout)?.contains("Rava verification accepted: false\n"));

    let accepted_scenario = Scenario::new(750)?;
    let accepted_files = accepted_scenario.write_files("replay-accepted-after-rejected")?;
    let accepted = accepted_scenario
        .run_verify_action(&accepted_files, &[("--replay-store", &replay_path)])?;
    assert!(
        accepted.status.success(),
        "accepted verify failed with stderr: {}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    assert!(String::from_utf8(accepted.stdout)?.contains("Rava verification accepted: true\n"));
    Ok(())
}

#[test]
fn verify_action_files_with_revocation_store_rejects_revoked_capability(
) -> Result<(), Box<dyn Error>> {
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("revoked")?;
    let revocation_path = files.directory.join("revocations.json");
    fs::write(
        &revocation_path,
        format!(
            "{{\"revoked_ids\":[\"{}\"]}}",
            scenario.chain.last().ok_or("missing final capability")?.id
        ),
    )?;

    let output = scenario.run_verify_action(&files, &[("--revocation-store", &revocation_path)])?;

    assert!(
        output.status.success(),
        "revocation rejection should not be a CLI failure: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Rava verification accepted: false\n"));
    assert!(stdout.contains("Rava rejection: CapabilityRevoked"));
    Ok(())
}

#[test]
fn verify_action_requires_fresh_revocation_snapshot_when_configured() -> Result<(), Box<dyn Error>>
{
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("revocation-fresh")?;
    let revocation_path = files.directory.join("revocations.json");
    fs::write(
        &revocation_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "revoked_ids": [],
            "fresh_until_unix": 1650000001
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "verify",
            "action",
            "--action",
            files.action_path.to_str().ok_or("invalid action path")?,
            "--capability-chain",
            files.chain_path.to_str().ok_or("invalid chain path")?,
            "--actor-key",
            &scenario.booking_agent.public_key_hex,
            "--issuer-key",
            &format!("{}={}", scenario.human.id, scenario.human.public_key_hex),
            "--issuer-key",
            &format!(
                "{}={}",
                scenario.personal_agent.id, scenario.personal_agent.public_key_hex
            ),
            "--now-unix",
            "1650000000",
            "--revocation-store",
            revocation_path.to_str().ok_or("invalid revocation path")?,
            "--require-fresh-revocations",
        ])
        .output()?;

    assert!(
        output.status.success(),
        "fresh revocation snapshot should allow verification: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8(output.stdout)?.contains("Rava verification accepted: true\n"));
    Ok(())
}

#[test]
fn verify_action_fails_closed_for_stale_required_revocation_snapshot() -> Result<(), Box<dyn Error>>
{
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("revocation-stale")?;
    let revocation_path = files.directory.join("revocations.json");
    fs::write(
        &revocation_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "revoked_ids": [],
            "fresh_until_unix": 1650000000
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "verify",
            "action",
            "--action",
            files.action_path.to_str().ok_or("invalid action path")?,
            "--capability-chain",
            files.chain_path.to_str().ok_or("invalid chain path")?,
            "--actor-key",
            &scenario.booking_agent.public_key_hex,
            "--issuer-key",
            &format!("{}={}", scenario.human.id, scenario.human.public_key_hex),
            "--issuer-key",
            &format!(
                "{}={}",
                scenario.personal_agent.id, scenario.personal_agent.public_key_hex
            ),
            "--now-unix",
            "1650000000",
            "--revocation-store",
            revocation_path.to_str().ok_or("invalid revocation path")?,
            "--require-fresh-revocations",
        ])
        .output()?;

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)?.contains("rava:"));
    Ok(())
}

#[test]
fn verify_action_files_with_corrupt_replay_store_fails_closed() -> Result<(), Box<dyn Error>> {
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("corrupt-replay")?;
    let replay_path = files.directory.join("replay.json");
    fs::write(&replay_path, b"not-json")?;

    let output = scenario.run_verify_action(&files, &[("--replay-store", &replay_path)])?;

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)?.contains("rava:"));
    Ok(())
}

#[test]
fn verify_action_files_with_corrupt_revocation_store_fails_closed() -> Result<(), Box<dyn Error>> {
    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("corrupt-revocation")?;
    let revocation_path = files.directory.join("revocations.json");
    fs::write(&revocation_path, b"not-json")?;

    let output = scenario.run_verify_action(&files, &[("--revocation-store", &revocation_path)])?;

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)?.contains("rava:"));
    Ok(())
}
