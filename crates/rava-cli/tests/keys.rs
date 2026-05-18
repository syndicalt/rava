use std::error::Error;
use std::fs;
use std::process::Command;

#[path = "support/scenario.rs"]
mod scenario;
#[path = "support/temp.rs"]
mod temp;

use scenario::Scenario;
use temp::temp_directory;

#[test]
fn key_generate_refuses_to_overwrite_without_force() -> Result<(), Box<dyn Error>> {
    let directory = temp_directory("key-overwrite")?;
    fs::create_dir_all(&directory)?;
    let key_path = directory.join("verifier-key.json");

    let first = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "key",
            "generate",
            "--kind",
            "service",
            "--out",
            key_path.to_str().ok_or("invalid key path")?,
        ])
        .output()?;
    assert!(first.status.success());

    let second = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "key",
            "generate",
            "--kind",
            "service",
            "--out",
            key_path.to_str().ok_or("invalid key path")?,
        ])
        .output()?;

    assert!(!second.status.success());
    assert!(String::from_utf8(second.stderr)?.contains("rava:"));
    Ok(())
}

#[test]
fn key_generate_overwrites_with_force_without_printing_private_key() -> Result<(), Box<dyn Error>> {
    let directory = temp_directory("key-force")?;
    fs::create_dir_all(&directory)?;
    let key_path = directory.join("verifier-key.json");

    let first = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "key",
            "generate",
            "--kind",
            "service",
            "--out",
            key_path.to_str().ok_or("invalid key path")?,
        ])
        .output()?;
    assert!(first.status.success());
    let first_key: serde_json::Value = serde_json::from_slice(&fs::read(&key_path)?)?;

    let second = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "key",
            "generate",
            "--kind",
            "service",
            "--out",
            key_path.to_str().ok_or("invalid key path")?,
            "--force",
        ])
        .output()?;

    assert!(
        second.status.success(),
        "forced key generation failed with stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    let second_stdout = String::from_utf8(second.stdout)?;
    assert!(!second_stdout.contains("private_key_hex"));
    let second_key: serde_json::Value = serde_json::from_slice(&fs::read(&key_path)?)?;
    assert_ne!(first_key.get("id"), second_key.get("id"));
    Ok(())
}

#[test]
fn key_revoke_records_signer_id_in_revocation_store() -> Result<(), Box<dyn Error>> {
    let directory = temp_directory("key-revoke")?;
    fs::create_dir_all(&directory)?;
    let revocation_store = directory.join("revocations.json");

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "key",
            "revoke",
            "--id",
            "signer_demo",
            "--revocation-store",
            revocation_store.to_str().ok_or("invalid revocation path")?,
        ])
        .output()?;

    assert!(
        output.status.success(),
        "key revoke failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let revocations: serde_json::Value = serde_json::from_slice(&fs::read(&revocation_store)?)?;
    assert_eq!(
        revocations
            .get("revoked_ids")
            .and_then(serde_json::Value::as_array)
            .map(|ids| ids.iter().any(|id| id.as_str() == Some("signer_demo"))),
        Some(true)
    );
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Rava key revoked: signer_demo"));
    assert!(!stdout.contains("private_key_hex"));
    Ok(())
}

#[test]
fn key_revoke_preserves_revocation_store_freshness() -> Result<(), Box<dyn Error>> {
    let directory = temp_directory("key-revoke-freshness")?;
    fs::create_dir_all(&directory)?;
    let revocation_store = directory.join("revocations.json");
    fs::write(
        &revocation_store,
        serde_json::to_vec_pretty(&serde_json::json!({
            "revoked_ids": ["old_signer"],
            "fresh_until_unix": 1_700_000_000_i64
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "key",
            "revoke",
            "--id",
            "new_signer",
            "--revocation-store",
            revocation_store.to_str().ok_or("invalid revocation path")?,
        ])
        .output()?;

    assert!(
        output.status.success(),
        "key revoke failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let revocations: serde_json::Value = serde_json::from_slice(&fs::read(&revocation_store)?)?;
    assert_eq!(
        revocations
            .get("fresh_until_unix")
            .and_then(serde_json::Value::as_i64),
        Some(1_700_000_000)
    );
    assert_eq!(
        revocations
            .get("revoked_ids")
            .and_then(serde_json::Value::as_array)
            .map(|ids| ids.iter().any(|id| id.as_str() == Some("old_signer"))),
        Some(true)
    );
    assert_eq!(
        revocations
            .get("revoked_ids")
            .and_then(serde_json::Value::as_array)
            .map(|ids| ids.iter().any(|id| id.as_str() == Some("new_signer"))),
        Some(true)
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn key_generate_force_replaces_symlink_instead_of_writing_through_it() -> Result<(), Box<dyn Error>>
{
    use std::os::unix::fs::{symlink, PermissionsExt};

    let directory = temp_directory("key-force-symlink")?;
    fs::create_dir_all(&directory)?;
    let key_path = directory.join("verifier-key.json");
    let symlink_target = directory.join("unexpected-target.json");
    fs::write(&symlink_target, b"sentinel")?;
    symlink(&symlink_target, &key_path)?;

    let output = Command::new(env!("CARGO_BIN_EXE_rava"))
        .args([
            "key",
            "generate",
            "--kind",
            "service",
            "--out",
            key_path.to_str().ok_or("invalid key path")?,
            "--force",
        ])
        .output()?;

    assert!(
        output.status.success(),
        "forced key generation failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read(&symlink_target)?, b"sentinel");
    assert!(!fs::symlink_metadata(&key_path)?.file_type().is_symlink());

    let key_file: serde_json::Value = serde_json::from_slice(&fs::read(&key_path)?)?;
    assert!(key_file.get("private_key_hex").is_some());
    assert_eq!(fs::metadata(&key_path)?.permissions().mode() & 0o777, 0o600);
    Ok(())
}

#[cfg(unix)]
#[test]
fn private_key_load_rejects_group_or_world_readable_file() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::PermissionsExt;

    let scenario = Scenario::new(750)?;
    let files = scenario.write_files("key-permissions")?;
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
    assert!(keygen.status.success());
    let mut permissions = fs::metadata(&key_path)?.permissions();
    permissions.set_mode(0o644);
    fs::set_permissions(&key_path, permissions)?;

    let output = scenario.run_verify_action(
        &files,
        &[
            ("--receipt-out", &receipt_path),
            ("--receipt-key", &key_path),
        ],
    )?;

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)?.contains("rava:"));
    Ok(())
}
