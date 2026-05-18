use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use rava_core::identity::{Signer, SignerKind};

pub fn write_signer_key_file(
    path: &Path,
    signer: &Signer,
    force: bool,
) -> Result<(), Box<dyn Error>> {
    let bytes = serde_json::to_vec_pretty(&serde_json::json!({
        "version": "rava-local-signer-key-v0",
        "id": signer.id,
        "kind": signer_kind_label(signer.kind),
        "public_key_hex": signer.public_key_hex,
        "private_key_hex": signer.signing_key_hex(),
    }))?;
    if force {
        write_forced_key_file(path, &bytes)?;
    } else {
        let mut file = create_new_key_file(path)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
    }
    Ok(())
}

pub fn read_signer_key_file(path: &PathBuf) -> Result<Signer, Box<dyn Error>> {
    validate_key_file_permissions(path)?;
    let value: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
    let version = value
        .get("version")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "key file missing version"))?;
    if version != "rava-local-signer-key-v0" {
        return Err(
            io::Error::new(io::ErrorKind::InvalidData, "unsupported key file version").into(),
        );
    }
    let kind = value
        .get("kind")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "key file missing kind"))?;
    let private_key_hex = value
        .get("private_key_hex")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "key file missing private_key_hex",
            )
        })?;
    let signer = Signer::from_signing_key_hex(parse_signer_kind(kind)?, private_key_hex)?;
    let expected_id = value
        .get("id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "key file missing id"))?;
    if signer.id != expected_id {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "key file id does not match private key",
        )
        .into());
    }
    Ok(signer)
}

pub fn parse_signer_kind(kind: &str) -> Result<SignerKind, Box<dyn Error>> {
    match kind {
        "human" => Ok(SignerKind::Human),
        "agent" => Ok(SignerKind::Agent),
        "service" => Ok(SignerKind::Service),
        "runtime" => Ok(SignerKind::Runtime),
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "unsupported signer kind").into()),
    }
}

fn signer_kind_label(kind: SignerKind) -> &'static str {
    match kind {
        SignerKind::Human => "human",
        SignerKind::Agent => "agent",
        SignerKind::Service => "service",
        SignerKind::Runtime => "runtime",
    }
}

#[cfg(unix)]
fn validate_key_file_permissions(path: &PathBuf) -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::PermissionsExt;

    let mode = fs::metadata(path)?.permissions().mode();
    if mode & 0o077 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "key file must not be readable, writable, or executable by group or others",
        )
        .into());
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_key_file_permissions(_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    Ok(())
}

#[cfg(unix)]
fn create_new_key_file(path: &Path) -> Result<File, Box<dyn Error>> {
    use std::os::unix::fs::OpenOptionsExt;

    Ok(OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?)
}

#[cfg(not(unix))]
fn create_new_key_file(path: &Path) -> Result<File, Box<dyn Error>> {
    Ok(OpenOptions::new().write(true).create_new(true).open(path)?)
}

fn write_forced_key_file(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let temp_path = forced_key_temp_path(path);
    let result = (|| -> Result<(), Box<dyn Error>> {
        let mut file = create_new_key_file(&temp_path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        rename_forced_key_file(&temp_path, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

fn forced_key_temp_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("rava-key");
    path.with_file_name(format!(".{file_name}.rava-new-{}", std::process::id()))
}

#[cfg(unix)]
fn rename_forced_key_file(temp_path: &Path, path: &Path) -> Result<(), Box<dyn Error>> {
    fs::rename(temp_path, path)?;
    Ok(())
}

#[cfg(not(unix))]
fn rename_forced_key_file(temp_path: &Path, path: &Path) -> Result<(), Box<dyn Error>> {
    match fs::rename(temp_path, path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            fs::remove_file(path)?;
            fs::rename(temp_path, path)?;
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}
