use std::error::Error;
use std::path::{Path, PathBuf};

#[test]
fn interop_plan_keeps_wrappers_subordinate_to_rust_verifier() -> Result<(), Box<dyn Error>> {
    let root = repository_root();
    let plan = std::fs::read_to_string(root.join("docs/interop/roadmap-v0.md"))?;
    let roadmap = std::fs::read_to_string(root.join("docs/roadmap.md"))?;

    assert!(plan.contains("Rust verifier remains the trusted implementation"));
    assert!(plan.contains("WASM and TypeScript wrappers must not reimplement verification logic"));
    assert!(plan.contains("DID/key resolution is a caller trust-policy layer"));
    assert!(plan.contains("MCP adapters pass signed action envelopes to the verifier"));
    assert!(plan
        .contains("OAuth exchange is integration glue, not a replacement for Rava capabilities"));
    assert!(plan.contains("No wrapper may weaken fail-closed verifier behavior"));
    for required in [
        "Interop evidence lives in `docs/interop/roadmap-v0.md`, `docs/interop/wasm-v0.md`, `docs/interop/typescript-v0.md`, `docs/interop/did-resolution-v0.md`, `docs/interop/mcp-adapter-v0.md`, `docs/interop/oauth-exchange-v0.md`, `crates/rava-cli/tests/interop_plan.rs`, `crates/rava-wasm/src/lib.rs`, `packages/rava-wasm-js/src/index.ts`, and `packages/rava-wasm-js/test/vectors.test.ts`.",
        "docs/interop/wasm-v0.md",
        "docs/interop/typescript-v0.md",
        "docs/interop/did-resolution-v0.md",
        "docs/interop/mcp-adapter-v0.md",
        "docs/interop/oauth-exchange-v0.md",
        "crates/rava-wasm/src/lib.rs",
        "packages/rava-wasm-js/test/vectors.test.ts",
    ] {
        assert!(
            roadmap.contains(required),
            "roadmap missing interop evidence: {required}"
        );
    }
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

#[test]
fn typescript_package_calls_wasm_and_runs_vectors() -> Result<(), Box<dyn Error>> {
    let root = repository_root();
    let manifest = std::fs::read_to_string(root.join("packages/rava-wasm-js/package.json"))?;
    let wrapper = std::fs::read_to_string(root.join("packages/rava-wasm-js/src/index.ts"))?;
    let tests = std::fs::read_to_string(root.join("packages/rava-wasm-js/test/vectors.test.ts"))?;
    let docs = std::fs::read_to_string(root.join("docs/interop/typescript-v0.md"))?;

    for required in [
        r#""name": "rava-wasm-js""#,
        "build:wasm",
        "wasm-bindgen",
        "node --test dist/test/vectors.test.js",
    ] {
        assert!(
            manifest.contains(required),
            "missing TS package manifest: {required}"
        );
    }

    for required in ["verifyAction", "verify_action_json", "../wasm/rava_wasm.js"] {
        assert!(wrapper.contains(required), "missing TS wrapper: {required}");
    }

    for required in [
        "test-vectors/v0/flight-booking",
        "verifyAction accepts the V0 flight-booking vector",
        "action_signature_invalid",
    ] {
        assert!(
            tests.contains(required),
            "missing TS vector test: {required}"
        );
    }

    for required in [
        "# Rava TypeScript V0 Package",
        "calls the generated WASM wrapper",
        "does not reimplement Rava verification",
        "npm --prefix packages/rava-wasm-js test",
        "npm pack --dry-run",
    ] {
        assert!(docs.contains(required), "missing TS docs: {required}");
    }

    Ok(())
}

#[test]
fn did_key_resolution_examples_stay_outside_core_trust() -> Result<(), Box<dyn Error>> {
    let docs =
        std::fs::read_to_string(repository_root().join("docs/interop/did-resolution-v0.md"))?;

    for required in [
        "# Rava DID and Key Resolution V0",
        "caller trust-policy layer",
        "Rava V0 verifies signatures against public keys supplied by the caller",
        "resolve before invoking",
        "fail closed",
        "resolver_freshness_unix",
        "actor_public_key_hex",
        "issuer_public_keys",
        "not implemented in `rava-core`",
    ] {
        assert!(docs.contains(required), "missing DID docs: {required}");
    }

    Ok(())
}

#[test]
fn mcp_adapter_poc_requires_verification_before_tool_execution() -> Result<(), Box<dyn Error>> {
    let docs = std::fs::read_to_string(repository_root().join("docs/interop/mcp-adapter-v0.md"))?;

    for required in [
        "# Rava MCP Adapter V0",
        "proof of concept",
        "verify before tool execution",
        "deny by default",
        "verifyAction",
        "accepted !== true",
        "capability_chain",
        "actor_public_key_hex",
        "issuer_public_keys",
        "must not replace capability attenuation with broad tool grants",
    ] {
        assert!(docs.contains(required), "missing MCP docs: {required}");
    }

    Ok(())
}

#[test]
fn oauth_exchange_examples_keep_rava_authorization_first() -> Result<(), Box<dyn Error>> {
    let docs =
        std::fs::read_to_string(repository_root().join("docs/interop/oauth-exchange-v0.md"))?;

    for required in [
        "# Rava OAuth Exchange V0",
        "integration glue",
        "Rava verification happens before token exchange",
        "OAuth scopes do not prove delegation-chain attenuation",
        "verified action context",
        "token custody",
        "fail closed",
        "not an OAuth replacement",
    ] {
        assert!(docs.contains(required), "missing OAuth docs: {required}");
    }

    Ok(())
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
