use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn temp_directory(label: &str) -> Result<std::path::PathBuf, Box<dyn Error>> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(std::env::temp_dir().join(format!("rava-cli-{}-{label}-{nonce}", std::process::id())))
}
