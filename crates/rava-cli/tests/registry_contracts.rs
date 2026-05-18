use std::error::Error;
use std::path::{Path, PathBuf};

#[test]
fn protocol_documents_replay_and_revocation_registry_contracts() -> Result<(), Box<dyn Error>> {
    let protocol = std::fs::read_to_string(repository_root().join("docs/protocol/rava-v0.md"))?;

    assert!(protocol.contains("Replay registry contract"));
    assert!(protocol.contains(
        "one-time verification asks the replay registry to atomically consume an accepted action ID"
    ));
    assert!(protocol.contains("recording an accepted action ID must be durable before the verifier reports one-time verification success"));
    assert!(protocol.contains(
        "if atomic consumption reports that the action ID was already consumed, one-time verification reports `action_replayed`"
    ));
    assert!(protocol.contains("serializes consume operations with a lock file"));
    assert!(protocol.contains("not distributed replay coordination across nodes or regions"));
    assert!(protocol.contains("Rejected actions must not be recorded"));
    assert!(protocol.contains("Revocation registry contract"));
    assert!(protocol.contains(
        "Registry lookup failures must fail closed before verification claims acceptance"
    ));
    assert!(protocol.contains("Freshness and distribution are caller responsibilities in V0"));
    Ok(())
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
