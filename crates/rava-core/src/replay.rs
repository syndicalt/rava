use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Replay registry contract for one-time action verification.
///
/// Implementations must report whether an accepted action ID has already been
/// consumed. `consume_action_id` must atomically record the action ID only if it
/// has not already been consumed, and the record must be durable for the
/// caller's one-time-use boundary before returning `Ok(true)`. If recording
/// fails, verification must not report successful one-time consumption.
pub trait ReplayRegistry {
    fn has_seen(&self, action_id: &str) -> bool;
    fn record(&mut self, action_id: String) -> Result<(), ReplayStoreError>;
    fn consume_action_id(&mut self, action_id: String) -> Result<bool, ReplayStoreError> {
        if self.has_seen(&action_id) {
            return Ok(false);
        }
        self.record(action_id)?;
        Ok(true)
    }
}

#[derive(Debug, Default, Clone)]
pub struct InMemoryReplayRegistry {
    action_ids: BTreeSet<String>,
}

impl ReplayRegistry for InMemoryReplayRegistry {
    fn has_seen(&self, action_id: &str) -> bool {
        self.action_ids.contains(action_id)
    }

    fn record(&mut self, action_id: String) -> Result<(), ReplayStoreError> {
        self.action_ids.insert(action_id);
        Ok(())
    }

    fn consume_action_id(&mut self, action_id: String) -> Result<bool, ReplayStoreError> {
        Ok(self.action_ids.insert(action_id))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReplayStoreError {
    #[error("replay store I/O failed: {0}")]
    Io(#[from] io::Error),

    #[error("replay store JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct FileReplayRegistry {
    path: PathBuf,
    action_ids: BTreeSet<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ReplayDocument {
    action_ids: BTreeSet<String>,
}

impl FileReplayRegistry {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ReplayStoreError> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Ok(Self {
                path,
                action_ids: BTreeSet::new(),
            });
        }

        let bytes = fs::read(&path)?;
        let document: ReplayDocument = serde_json::from_slice(&bytes)?;

        Ok(Self {
            path,
            action_ids: document.action_ids,
        })
    }

    fn persist(&self) -> Result<(), ReplayStoreError> {
        let document = ReplayDocument {
            action_ids: self.action_ids.clone(),
        };
        let bytes = serde_json::to_vec_pretty(&document)?;
        let temporary_path = self.path.with_extension(format!("{}.tmp", Uuid::new_v4()));

        fs::write(&temporary_path, bytes)?;
        fs::rename(&temporary_path, &self.path)?;
        Ok(())
    }
}

impl ReplayRegistry for FileReplayRegistry {
    fn has_seen(&self, action_id: &str) -> bool {
        self.action_ids.contains(action_id)
    }

    fn record(&mut self, action_id: String) -> Result<(), ReplayStoreError> {
        self.consume_action_id(action_id)?;
        Ok(())
    }

    fn consume_action_id(&mut self, action_id: String) -> Result<bool, ReplayStoreError> {
        if self.action_ids.contains(&action_id) {
            return Ok(false);
        }
        let previous_action_ids = self.action_ids.clone();
        self.action_ids.insert(action_id);
        if let Err(error) = self.persist() {
            self.action_ids = previous_action_ids;
            return Err(error);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::fs;
    use uuid::Uuid;

    #[test]
    fn file_registry_persists_seen_action_ids() -> Result<(), Box<dyn Error>> {
        let path = std::env::temp_dir().join(format!("rava-replay-{}.json", Uuid::new_v4()));
        let mut registry = FileReplayRegistry::open(&path)?;

        registry.record("act_demo".to_owned())?;
        let reloaded = FileReplayRegistry::open(&path)?;

        assert!(reloaded.has_seen("act_demo"));
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn file_registry_rejects_invalid_json() -> Result<(), Box<dyn Error>> {
        let path = std::env::temp_dir().join(format!("rava-replay-{}.json", Uuid::new_v4()));
        fs::write(&path, b"not-json")?;

        let result = FileReplayRegistry::open(&path);

        assert!(matches!(result, Err(ReplayStoreError::Json(_))));
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn file_registry_does_not_consume_action_when_persistence_fails() -> Result<(), Box<dyn Error>>
    {
        let path = std::env::temp_dir()
            .join(format!("rava-replay-missing-{}", Uuid::new_v4()))
            .join("replay.json");
        let mut registry = FileReplayRegistry::open(&path)?;

        let result = registry.record("act_not_durable".to_owned());

        assert!(matches!(result, Err(ReplayStoreError::Io(_))));
        assert!(!registry.has_seen("act_not_durable"));
        Ok(())
    }
}
