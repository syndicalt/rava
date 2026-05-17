use std::error::Error;
use std::path::{Path, PathBuf};

#[test]
fn readme_states_publication_posture_and_operator_path() -> Result<(), Box<dyn Error>> {
    let readme = std::fs::read_to_string(repository_root().join("README.md"))?;

    for required in [
        "Rava V0 is a draft reference implementation, not production-ready security software.",
        "## Table of Contents",
        "## Requirements",
        "## Quickstart",
        "## Repository Layout",
        "## Verification Gates",
        "docs/roadmap.md",
        "cargo fmt --check",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "cargo test --workspace",
        "cargo run -p rava -- demo flight-booking",
        "cargo package --workspace",
        "cargo check -p rava-wasm --target wasm32-unknown-unknown",
        "npm --prefix packages/rava-wasm-js test",
        "--deterministic-fixtures",
        "--max-request-bytes",
        "--replay-store",
        "--revocation-store",
        "--audit-log",
        "--auth-token-env",
        "--rate-limit-per-minute",
        "GET /healthz",
        "From this repository, prefix CLI commands with `cargo run -p rava --`.",
        "The preview service is not a production authorization service.",
    ] {
        assert!(readme.contains(required), "README missing: {required}");
    }

    Ok(())
}

#[test]
fn functional_roadmap_separates_current_state_from_future_work() -> Result<(), Box<dyn Error>> {
    let roadmap = std::fs::read_to_string(repository_root().join("docs/roadmap.md"))?;

    for required in [
        "# Rava Functional Roadmap",
        "## Current Baseline",
        "## Release Readiness",
        "## V0 Hardening",
        "## V1 Developer Preview",
        "## Interop",
        "## Production Trust and Operations",
        "## Non-Goals",
        "not implemented guarantees today",
    ] {
        assert!(roadmap.contains(required), "roadmap missing: {required}");
    }

    Ok(())
}

#[test]
fn release_audit_does_not_report_resolved_eventloom_state_as_current() -> Result<(), Box<dyn Error>>
{
    let audit =
        std::fs::read_to_string(repository_root().join("docs/security/release-audit-v0.md"))?;

    assert!(audit.contains("`.eventloom/` is ignored"));
    assert!(!audit.contains("currently contains `.eventloom/rava-default.jsonl`"));
    Ok(())
}

#[test]
fn operator_rejection_code_docs_cover_stable_verifier_codes() -> Result<(), Box<dyn Error>> {
    let docs =
        std::fs::read_to_string(repository_root().join("docs/operators/rejection-codes-v0.md"))?;

    for required in [
        "# Rava V0 Rejection Codes",
        "## Operator Contract",
        "## Stable Codes",
        "unsupported_action_version",
        "unsupported_capability_version",
        "action_nonce_invalid",
        "capability_nonce_invalid",
        "action_context_hash_invalid",
        "action_signature_invalid",
        "action_id_mismatch",
        "action_replayed",
        "capability_chain_empty",
        "action_capability_not_final",
        "root_issuer_not_controller",
        "capability_id_mismatch",
        "capability_operations_empty",
        "capability_operations_not_canonical",
        "capability_signature_invalid",
        "capability_revoked",
        "signer_revoked",
        "capability_expired",
        "capability_parent_mismatch",
        "capability_issuer_not_parent_subject",
        "capability_resource_mismatch",
        "capability_operation_not_granted",
        "capability_expiry_outlives_parent",
        "capability_constraint_removed",
        "capability_constraint_expanded",
        "parent_capability_not_delegable",
        "final_subject_not_actor",
        "resource_mismatch",
        "operation_not_allowed",
        "constraint_exceeded",
        "missing_issuer_public_key",
    ] {
        assert!(
            docs.contains(required),
            "missing rejection-code docs: {required}"
        );
    }

    Ok(())
}

#[test]
fn time_semantics_docs_state_expiry_and_freshness_assumptions() -> Result<(), Box<dyn Error>> {
    let docs =
        std::fs::read_to_string(repository_root().join("docs/protocol/time-semantics-v0.md"))?;

    for required in [
        "# Rava V0 Time Semantics",
        "`now_unix`",
        "A capability is expired when `expires_at <= now`.",
        "Rava V0 does not apply implicit clock skew.",
        "Revocation and replay freshness are caller responsibilities.",
        "Use one verifier time source for all checks in a verification decision.",
    ] {
        assert!(
            docs.contains(required),
            "missing time-semantics docs: {required}"
        );
    }

    Ok(())
}

#[test]
fn security_review_guide_maps_review_scope_to_evidence() -> Result<(), Box<dyn Error>> {
    let guide =
        std::fs::read_to_string(repository_root().join("docs/security/review-guide-v0.md"))?;

    for required in [
        "# Rava V0 Security Review Guide",
        "## Review Scope",
        "## High-Value Review Questions",
        "canonicalization",
        "signature binding",
        "nonce validation",
        "replay semantics",
        "revocation semantics",
        "## Evidence Map",
        "crates/rava-core/src/verifier.rs",
        "docs/security/threat-model-v0.md",
        "cargo test --workspace",
        "not production-ready security software",
    ] {
        assert!(
            guide.contains(required),
            "missing review-guide docs: {required}"
        );
    }

    Ok(())
}

#[test]
fn security_review_register_tracks_external_findings_without_claiming_review(
) -> Result<(), Box<dyn Error>> {
    let register =
        std::fs::read_to_string(repository_root().join("docs/security/review-register-v0.md"))?;
    let guide =
        std::fs::read_to_string(repository_root().join("docs/security/review-guide-v0.md"))?;
    let audit =
        std::fs::read_to_string(repository_root().join("docs/security/release-audit-v0.md"))?;
    let production =
        std::fs::read_to_string(repository_root().join("docs/operations/production-trust-v0.md"))?;

    for required in [
        "# Rava V0 Security Review Register",
        "No external security review has been completed yet.",
        "## Finding States",
        "reported",
        "accepted-risk",
        "remediated",
        "verified",
        "## Register",
        "No external findings are recorded yet.",
    ] {
        assert!(
            register.contains(required),
            "missing security review register docs: {required}"
        );
    }

    for docs in [guide, audit, production] {
        assert!(
            docs.contains("docs/security/review-register-v0.md")
                || docs.contains("review-register-v0.md")
                || docs.contains("../security/review-register-v0.md"),
            "security review docs must link to the review register"
        );
    }

    Ok(())
}

#[test]
fn compatibility_policy_defines_v0_change_boundaries() -> Result<(), Box<dyn Error>> {
    let policy = std::fs::read_to_string(
        repository_root().join("docs/protocol/compatibility-policy-v0.md"),
    )?;

    for required in [
        "# Rava V0 Compatibility Policy",
        "## Stable During V0 Draft",
        "## Changes That Require New Test Vectors",
        "## Changes That Require a New Protocol Version",
        "wire object versions",
        "rejection code",
        "test-vectors/v0",
        "schemas",
        "No compatibility change may weaken fail-closed verification.",
    ] {
        assert!(
            policy.contains(required),
            "missing compatibility policy docs: {required}"
        );
    }

    Ok(())
}

#[test]
fn v1_preview_surface_docs_pin_cli_and_json_shapes() -> Result<(), Box<dyn Error>> {
    let docs =
        std::fs::read_to_string(repository_root().join("docs/protocol/v1-preview-surface.md"))?;

    for required in [
        "# Rava V1 Preview Surface",
        "## Stable CLI Commands",
        "rava verify action",
        "rava serve verify",
        "--max-request-bytes",
        "--replay-store",
        "--revocation-store",
        "--audit-log",
        "## HTTP Request Shape",
        "POST /verify/action",
        "actor_public_key_hex",
        "issuer_public_keys",
        "## HTTP Response Shape",
        "accepted",
        "rejection.code",
        "rejection.subject",
        "## Audit Log Shape",
        "action_id",
        "verified_at_unix",
        "## Rejection-Code Subjects",
        "docs/operators/rejection-codes-v0.md",
        "not a production authorization boundary",
    ] {
        assert!(
            docs.contains(required),
            "missing v1-preview surface docs: {required}"
        );
    }

    Ok(())
}

#[test]
fn v1_preview_migration_notes_cover_preview_surface_changes() -> Result<(), Box<dyn Error>> {
    let docs =
        std::fs::read_to_string(repository_root().join("docs/protocol/v1-preview-migration.md"))?;

    for required in [
        "# Rava V1 Preview Migration Notes",
        "## Compatibility Summary",
        "No V0 signed wire object version is changed",
        "rava serve verify",
        "--auth-token-env",
        "--rate-limit-per-minute",
        "--replay-store",
        "--revocation-store",
        "--audit-log",
        "rava-wasm",
        "rava-wasm-js",
        "docs/protocol/v1-preview-surface.md",
        "not production-ready security software",
    ] {
        assert!(
            docs.contains(required),
            "missing v1-preview migration docs: {required}"
        );
    }

    Ok(())
}

#[test]
fn production_trust_docs_define_external_operational_requirements() -> Result<(), Box<dyn Error>> {
    let docs =
        std::fs::read_to_string(repository_root().join("docs/operations/production-trust-v0.md"))?;

    for required in [
        "# Rava Production Trust and Operations V0",
        "not implemented guarantees",
        "## Key Custody",
        "rotation",
        "compromise response",
        "## Public-Key Discovery",
        "resolver freshness",
        "## Distributed Replay",
        "## Distributed Revocation",
        "## Audit Storage",
        "retention",
        "privacy",
        "export",
        "## Monitoring",
        "rejection patterns",
        "## External Security Review",
        "required before Rava is represented as a production authorization system",
    ] {
        assert!(
            docs.contains(required),
            "missing production trust docs: {required}"
        );
    }

    Ok(())
}

#[test]
fn fuzz_targets_cover_parser_canonicalization_and_verifier_entrypoints(
) -> Result<(), Box<dyn Error>> {
    let root = repository_root();
    let manifest = std::fs::read_to_string(root.join("fuzz/Cargo.toml"))?;
    let target = std::fs::read_to_string(root.join("fuzz/fuzz_targets/v0_wire_entrypoints.rs"))?;

    for required in [
        "[[bin]]",
        "v0_wire_entrypoints",
        "libfuzzer_sys",
        "rava-core",
    ] {
        assert!(
            manifest.contains(required),
            "missing fuzz manifest entry: {required}"
        );
    }

    for required in [
        "serde_json::from_slice",
        "canonical_json",
        "verify_action",
        "verify_verification_receipt",
        "verify_attestation",
        "no_main",
    ] {
        assert!(
            target.contains(required),
            "missing fuzz target coverage: {required}"
        );
    }

    Ok(())
}

#[test]
fn workspace_publish_metadata_uses_real_repository_url() -> Result<(), Box<dyn Error>> {
    let manifest = std::fs::read_to_string(repository_root().join("Cargo.toml"))?;
    let core_manifest =
        std::fs::read_to_string(repository_root().join("crates/rava-core/Cargo.toml"))?;
    let cli_manifest =
        std::fs::read_to_string(repository_root().join("crates/rava-cli/Cargo.toml"))?;
    let wasm_manifest =
        std::fs::read_to_string(repository_root().join("crates/rava-wasm/Cargo.toml"))?;

    assert!(manifest.contains(r#"repository = "https://github.com/syndicalt/rava""#));
    assert!(!manifest.contains("https://example.invalid/rava"));
    for required in [
        "description = ",
        r#"repository.workspace = true"#,
        r#"license.workspace = true"#,
    ] {
        assert!(
            core_manifest.contains(required),
            "missing rava-core package metadata: {required}"
        );
        assert!(
            cli_manifest.contains(required),
            "missing rava package metadata: {required}"
        );
        assert!(
            wasm_manifest.contains(required),
            "missing rava-wasm package metadata: {required}"
        );
    }
    assert!(cli_manifest.contains(r#"rava-core = { version = "0.1.0", path = "../rava-core" }"#));
    assert!(wasm_manifest.contains(r#"rava-core = { version = "0.1.0", path = "../rava-core" }"#));
    Ok(())
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
