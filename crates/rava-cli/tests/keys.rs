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
