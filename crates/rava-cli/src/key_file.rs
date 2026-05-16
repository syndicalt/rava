use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;

use rava_core::identity::{Signer, SignerKind};

pub fn write_signer_key_file(path: &PathBuf, signer: &Signer) -> Result<(), Box<dyn Error>> {
    let bytes = serde_json::to_vec_pretty(&serde_json::json!({
        "version": "rava-local-signer-key-v0",
        "id": signer.id,
        "kind": signer_kind_label(signer.kind),
        "public_key_hex": signer.public_key_hex,
        "private_key_hex": signer.signing_key_hex(),
    }))?;
    fs::write(path, bytes)?;
    restrict_key_file_permissions(path)?;
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
fn restrict_key_file_permissions(path: &PathBuf) -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_key_file_permissions(_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    Ok(())
}
