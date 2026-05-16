use std::error::Error;
use std::path::{Path, PathBuf};

#[test]
fn interop_plan_keeps_wrappers_subordinate_to_rust_verifier() -> Result<(), Box<dyn Error>> {
    let plan = std::fs::read_to_string(repository_root().join("docs/interop/roadmap-v0.md"))?;

    assert!(plan.contains("Rust verifier remains the trusted implementation"));
    assert!(plan.contains("WASM and TypeScript wrappers must not reimplement verification logic"));
    assert!(plan.contains("DID/key resolution is a caller trust-policy layer"));
    assert!(plan.contains("MCP adapters pass signed action envelopes to the verifier"));
    assert!(plan
        .contains("OAuth exchange is integration glue, not a replacement for Rava capabilities"));
    assert!(plan.contains("No wrapper may weaken fail-closed verifier behavior"));
    Ok(())
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
