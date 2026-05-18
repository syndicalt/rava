use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Revocation registry contract for verifier-supplied revocation state.
///
/// Implementations answer whether a signer or capability ID is revoked in the
/// caller-provided snapshot. V0 does not guarantee distributed freshness;
/// callers must provide a sufficiently fresh registry for their risk boundary.
pub trait RevocationRegistry {
    fn is_revoked(&self, id: &str) -> bool;
}

#[derive(Debug, Default, Clone)]
pub struct InMemoryRevocationRegistry {
    revoked_ids: BTreeSet<String>,
}

impl InMemoryRevocationRegistry {
    pub fn revoke(&mut self, id: String) {
        self.revoked_ids.insert(id);
    }
}

impl RevocationRegistry for InMemoryRevocationRegistry {
    fn is_revoked(&self, id: &str) -> bool {
        self.revoked_ids.contains(id)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RevocationStoreError {
    #[error("revocation store I/O failed: {0}")]
    Io(#[from] io::Error),

    #[error("revocation store JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct FileRevocationRegistry {
    path: PathBuf,
    revoked_ids: BTreeSet<String>,
    fresh_until_unix: Option<i64>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct RevocationDocument {
    revoked_ids: BTreeSet<String>,
    fresh_until_unix: Option<i64>,
}

impl FileRevocationRegistry {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, RevocationStoreError> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Ok(Self {
                path,
                revoked_ids: BTreeSet::new(),
                fresh_until_unix: None,
            });
        }

        let bytes = fs::read(&path)?;
        let document: RevocationDocument = serde_json::from_slice(&bytes)?;

        Ok(Self {
            path,
            revoked_ids: document.revoked_ids,
            fresh_until_unix: document.fresh_until_unix,
        })
    }

    pub fn fresh_until_unix(&self) -> Option<i64> {
        self.fresh_until_unix
    }

    pub fn revoke_and_persist(&mut self, id: String) -> Result<(), RevocationStoreError> {
        self.revoked_ids.insert(id);
        self.persist()
    }

    fn persist(&self) -> Result<(), RevocationStoreError> {
        let document = RevocationDocument {
            revoked_ids: self.revoked_ids.clone(),
            fresh_until_unix: self.fresh_until_unix,
        };
        let bytes = serde_json::to_vec_pretty(&document)?;
        let temporary_path = self.path.with_extension(format!("{}.tmp", Uuid::new_v4()));

        fs::write(&temporary_path, bytes)?;
        fs::rename(&temporary_path, &self.path)?;
        Ok(())
    }
}

impl RevocationRegistry for FileRevocationRegistry {
    fn is_revoked(&self, id: &str) -> bool {
        self.revoked_ids.contains(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::fs;
    use uuid::Uuid;

    #[test]
    fn file_registry_persists_revoked_ids() -> Result<(), Box<dyn Error>> {
        let path = std::env::temp_dir().join(format!("rava-revocations-{}.json", Uuid::new_v4()));
        let mut registry = FileRevocationRegistry::open(&path)?;

        registry.revoke_and_persist("cap_demo".to_owned())?;
        let reloaded = FileRevocationRegistry::open(&path)?;

        assert!(reloaded.is_revoked("cap_demo"));
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn file_registry_rejects_invalid_json() -> Result<(), Box<dyn Error>> {
        let path = std::env::temp_dir().join(format!("rava-revocations-{}.json", Uuid::new_v4()));
        fs::write(&path, b"not-json")?;

        let result = FileRevocationRegistry::open(&path);

        assert!(matches!(result, Err(RevocationStoreError::Json(_))));
        fs::remove_file(path)?;
        Ok(())
    }
}
