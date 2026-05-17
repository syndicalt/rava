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

#[test]
fn wasm_wrapper_is_documented_as_rust_verifier_boundary() -> Result<(), Box<dyn Error>> {
    let root = repository_root();
    let manifest = std::fs::read_to_string(root.join("crates/rava-wasm/Cargo.toml"))?;
    let wrapper = std::fs::read_to_string(root.join("crates/rava-wasm/src/lib.rs"))?;
    let docs = std::fs::read_to_string(root.join("docs/interop/wasm-v0.md"))?;

    for required in [
        r#"name = "rava-wasm""#,
        "wasm-bindgen",
        r#"crate-type = ["cdylib", "rlib"]"#,
    ] {
        assert!(
            manifest.contains(required),
            "missing wasm manifest: {required}"
        );
    }

    for required in [
        "verify_action(VerifyActionInput",
        "InMemoryRevocationRegistry",
        "verify_action_json",
    ] {
        assert!(
            wrapper.contains(required),
            "missing wasm wrapper code: {required}"
        );
    }

    for required in [
        "# Rava WASM V0 Wrapper",
        "calls the Rust verifier",
        "does not reimplement signatures, canonicalization, or attenuation",
        "verify_action_json",
        "revoked_ids",
        "cargo check -p rava-wasm --target wasm32-unknown-unknown",
    ] {
        assert!(docs.contains(required), "missing wasm docs: {required}");
    }

    Ok(())
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
