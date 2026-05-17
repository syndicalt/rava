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
        "docs/release/v0-draft-checklist.md",
        "docs/release/notes-template-v0.md",
        "docs/operations/production-trust-v0.md",
        "docs/security/review-register-v0.md",
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
        "release/v0-draft-checklist.md",
        "release/notes-template-v0.md",
        "operations/key-custody-v0.md",
        "operations/key-discovery-v0.md",
        "operations/distributed-replay-v0.md",
        "operations/distributed-revocation-v0.md",
        "operations/audit-storage-v0.md",
        "operations/caller-identity-v0.md",
        "operations/distributed-rate-limits-v0.md",
        "operations/monitoring-v0.md",
        "security/review-register-v0.md",
        "cargo package --workspace",
        "cargo check --manifest-path fuzz/Cargo.toml",
        "cargo check -p rava-wasm --target wasm32-unknown-unknown",
        "npm --prefix packages/rava-wasm-js test",
    ] {
        assert!(roadmap.contains(required), "roadmap missing: {required}");
    }

    Ok(())
}

#[test]
fn hardening_property_regressions_are_visible_in_roadmap() -> Result<(), Box<dyn Error>> {
    let root = repository_root();
    let roadmap = std::fs::read_to_string(root.join("docs/roadmap.md"))?;
    let canonical = std::fs::read_to_string(root.join("crates/rava-core/src/canonical.rs"))?;
    let capability = std::fs::read_to_string(root.join("crates/rava-core/src/capability.rs"))?;
    let verifier = std::fs::read_to_string(root.join("crates/rava-core/src/verifier.rs"))?;

    for required in [
        "Property-style regression coverage currently guards canonical JSON stability, capability operation canonicalization, attenuation monotonicity, replay consumption, and malformed signed-object rejection.",
        "Additional V0 hardening evidence lives in `fuzz/fuzz_targets/v0_wire_entrypoints.rs`, `crates/rava-cli/tests/flight_booking.rs`, `crates/rava-cli/tests/test_vectors.rs`, `crates/rava-cli/tests/publication_docs.rs`, `docs/operators/rejection-codes-v0.md`, `docs/protocol/time-semantics-v0.md`, and `docs/security/review-guide-v0.md`.",
        "Keep property-style regression tests current for canonicalization, attenuation monotonicity, replay consumption, and malformed signed objects.",
        "crates/rava-core/src/canonical.rs",
        "crates/rava-core/src/capability.rs",
        "crates/rava-core/src/verifier.rs",
        "fuzz/fuzz_targets/v0_wire_entrypoints.rs",
        "crates/rava-cli/tests/flight_booking.rs",
        "crates/rava-cli/tests/test_vectors.rs",
        "docs/operators/rejection-codes-v0.md",
        "docs/protocol/time-semantics-v0.md",
        "docs/security/review-guide-v0.md",
    ] {
        assert!(
            roadmap.contains(required),
            "roadmap missing hardening coverage note: {required}"
        );
    }
    assert!(!roadmap.contains(
        "Add property tests for canonicalization, attenuation monotonicity, replay consumption, and malformed signed objects."
    ));

    for (source, required) in [
        (
            canonical.as_str(),
            "canonical_json_is_stable_after_parse_round_trip_for_permuted_objects",
        ),
        (
            capability.as_str(),
            "mint_and_delegation_canonicalize_operation_sets_for_permuted_inputs",
        ),
        (
            verifier.as_str(),
            "delegated_amount_constraints_are_monotonic_at_boundary",
        ),
        (
            verifier.as_str(),
            "malformed_signed_action_variants_fail_closed_after_resigning",
        ),
        (
            verifier.as_str(),
            "accepted_action_replay_is_rejected_but_rejected_action_is_not_consumed",
        ),
    ] {
        assert!(
            source.contains(required),
            "missing hardening regression test: {required}"
        );
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
fn release_audit_tracks_current_release_and_operations_artifacts() -> Result<(), Box<dyn Error>> {
    let audit =
        std::fs::read_to_string(repository_root().join("docs/security/release-audit-v0.md"))?;

    for required in [
        "../release/v0-draft-checklist.md",
        "../release/notes-template-v0.md",
        "../operations/key-custody-v0.md",
        "../operations/key-discovery-v0.md",
        "../operations/distributed-replay-v0.md",
        "../operations/distributed-revocation-v0.md",
        "../operations/caller-identity-v0.md",
        "../operations/distributed-rate-limits-v0.md",
        "../operations/monitoring-v0.md",
        "documented external requirements",
        "not implemented production systems",
        "local preview controls",
        "production responsibilities",
        "canonical JSON insertion-order and parse round-trip stability",
        "capability operation canonicalization",
        "exact capability expiry boundary",
        "The functional roadmap now includes evidence maps for release readiness, V0 hardening, V1 developer preview, interop, and production trust work.",
    ] {
        assert!(
            audit.contains(required),
            "release audit missing current artifact: {required}"
        );
    }
    assert!(!audit.contains("does not implement production service responsibilities such as request authentication, replay consumption, distributed revocation freshness, rate limiting, or persistent audit storage"));

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
    let root = repository_root();
    let docs = std::fs::read_to_string(root.join("docs/protocol/time-semantics-v0.md"))?;
    let verifier = std::fs::read_to_string(root.join("crates/rava-core/src/verifier.rs"))?;

    for required in [
        "# Rava V0 Time Semantics",
        "`now_unix`",
        "A capability is expired when `expires_at <= now`.",
        "Rava V0 does not apply implicit clock skew.",
        "Revocation and replay freshness are caller responsibilities.",
        "Use one verifier time source for all checks in a verification decision.",
        "Boundary regression coverage verifies that a capability is accepted immediately before `expires_at` and rejected exactly at `expires_at`.",
    ] {
        assert!(
            docs.contains(required),
            "missing time-semantics docs: {required}"
        );
    }
    assert!(verifier.contains("capability_expiry_boundary_rejects_at_exact_expiry_without_skew"));

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
    let root = repository_root();
    let roadmap = std::fs::read_to_string(root.join("docs/roadmap.md"))?;
    let docs = std::fs::read_to_string(root.join("docs/protocol/v1-preview-surface.md"))?;

    for required in [
        "# Rava V1 Preview Surface",
        "## Stable CLI Commands",
        "rava verify action",
        "rava serve verify",
        "--max-request-bytes",
        "--replay-store",
        "--revocation-store",
        "--audit-log",
        "--auth-token-env",
        "--rate-limit-per-minute",
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

    for required in [
        "V1 developer preview evidence lives in `docs/protocol/v1-preview-surface.md`, `docs/protocol/v1-preview-migration.md`, `docs/protocol/compatibility-policy-v0.md`, `docs/release/v0-draft-checklist.md`, `docs/release/notes-template-v0.md`, `crates/rava-cli/tests/publication_docs.rs`, `crates/rava-cli/tests/serve_verify.rs`, `crates/rava-cli/tests/test_vectors.rs`, and `crates/rava-cli/tests/wire_schemas.rs`.",
        "docs/protocol/v1-preview-surface.md",
        "docs/protocol/v1-preview-migration.md",
        "docs/protocol/compatibility-policy-v0.md",
        "docs/release/v0-draft-checklist.md",
        "docs/release/notes-template-v0.md",
        "crates/rava-cli/tests/serve_verify.rs",
        "crates/rava-cli/tests/wire_schemas.rs",
    ] {
        assert!(
            roadmap.contains(required),
            "roadmap missing v1-preview evidence: {required}"
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
        "../operations/caller-identity-v0.md",
        "../operations/distributed-replay-v0.md",
        "../operations/distributed-revocation-v0.md",
        "../operations/distributed-rate-limits-v0.md",
        "../operations/key-custody-v0.md",
        "../operations/key-discovery-v0.md",
        "../operations/audit-storage-v0.md",
        "../operations/monitoring-v0.md",
        "../security/review-register-v0.md",
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
    let root = repository_root();
    let docs = std::fs::read_to_string(root.join("docs/operations/production-trust-v0.md"))?;
    let roadmap = std::fs::read_to_string(root.join("docs/roadmap.md"))?;

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

    for required in [
        "Production trust evidence lives in `docs/operations/production-trust-v0.md`, `docs/operations/key-custody-v0.md`, `docs/operations/key-discovery-v0.md`, `docs/operations/distributed-replay-v0.md`, `docs/operations/distributed-revocation-v0.md`, `docs/operations/audit-storage-v0.md`, `docs/operations/caller-identity-v0.md`, `docs/operations/distributed-rate-limits-v0.md`, `docs/operations/monitoring-v0.md`, `docs/security/review-register-v0.md`, `docs/security/release-audit-v0.md`, and `crates/rava-cli/tests/publication_docs.rs`.",
        "docs/operations/production-trust-v0.md",
        "docs/operations/key-custody-v0.md",
        "docs/operations/key-discovery-v0.md",
        "docs/operations/distributed-replay-v0.md",
        "docs/operations/distributed-revocation-v0.md",
        "docs/operations/audit-storage-v0.md",
        "docs/operations/caller-identity-v0.md",
        "docs/operations/distributed-rate-limits-v0.md",
        "docs/operations/monitoring-v0.md",
        "docs/security/review-register-v0.md",
        "docs/security/release-audit-v0.md",
    ] {
        assert!(
            roadmap.contains(required),
            "roadmap missing production evidence: {required}"
        );
    }

    Ok(())
}

#[test]
fn audit_storage_runbook_defines_managed_audit_requirements_without_preview_claims(
) -> Result<(), Box<dyn Error>> {
    let runbook =
        std::fs::read_to_string(repository_root().join("docs/operations/audit-storage-v0.md"))?;
    let production =
        std::fs::read_to_string(repository_root().join("docs/operations/production-trust-v0.md"))?;
    let audit =
        std::fs::read_to_string(repository_root().join("docs/security/release-audit-v0.md"))?;

    for required in [
        "# Rava Production Audit Storage V0",
        "not implemented by the V0 preview service",
        "## Required Properties",
        "retention",
        "privacy",
        "export",
        "tamper evidence",
        "access control",
        "## Failure Policy",
        "fail closed",
        "## Correlation",
        "verification receipt",
        "attestation",
        "resolver evidence",
        "downstream tool or API call",
        "## Data Minimization",
        "raw action payloads",
    ] {
        assert!(
            runbook.contains(required),
            "missing audit storage runbook docs: {required}"
        );
    }

    for docs in [production, audit] {
        assert!(
            docs.contains("audit-storage-v0.md")
                || docs.contains("../operations/audit-storage-v0.md"),
            "production and release audit docs must link to the audit storage runbook"
        );
    }

    Ok(())
}

#[test]
fn production_operations_runbooks_define_external_systems_without_core_claims(
) -> Result<(), Box<dyn Error>> {
    let root = repository_root();
    let production = std::fs::read_to_string(root.join("docs/operations/production-trust-v0.md"))?;

    let runbooks = [
        (
            "key-custody-v0.md",
            [
                "# Rava Production Key Custody V0",
                "not implemented by the Rava V0 core",
                "generation ceremony",
                "rotation",
                "emergency rotation",
                "compromise response",
                "private keys",
            ],
        ),
        (
            "key-discovery-v0.md",
            [
                "# Rava Production Public-Key Discovery V0",
                "caller trust-policy layer",
                "trust roots",
                "resolver freshness",
                "cache lifetime",
                "fail closed",
                "rollback",
            ],
        ),
        (
            "distributed-replay-v0.md",
            [
                "# Rava Production Distributed Replay V0",
                "not implemented by the V0 preview service",
                "atomic action-ID consumption",
                "durability before acceptance",
                "partial failure",
                "cross-region",
                "fail closed",
            ],
        ),
        (
            "distributed-revocation-v0.md",
            [
                "# Rava Production Distributed Revocation V0",
                "not implemented by the V0 core",
                "freshness target",
                "maximum tolerated staleness",
                "emergency revocation",
                "outage behavior",
                "fail closed",
            ],
        ),
        (
            "monitoring-v0.md",
            [
                "# Rava Production Monitoring V0",
                "not implemented by the V0 preview service",
                "accepted and rejected decision rates",
                "rejection code distribution",
                "replay attempts",
                "audit-write failures",
                "must not leak private keys",
            ],
        ),
    ];

    for (file, required) in runbooks {
        let docs = std::fs::read_to_string(root.join("docs/operations").join(file))?;
        for phrase in required {
            assert!(docs.contains(phrase), "{file} missing: {phrase}");
        }
        assert!(
            production.contains(file),
            "production trust docs must link to {file}"
        );
    }

    Ok(())
}

#[test]
fn service_boundary_runbooks_define_caller_identity_and_distributed_rate_limits(
) -> Result<(), Box<dyn Error>> {
    let root = repository_root();
    let production = std::fs::read_to_string(root.join("docs/operations/production-trust-v0.md"))?;
    let migration = std::fs::read_to_string(root.join("docs/protocol/v1-preview-migration.md"))?;

    let runbooks: [(&str, &[&str]); 2] = [
        (
            "caller-identity-v0.md",
            &[
                "# Rava Production Caller Identity V0",
                "not implemented by the V0 preview service",
                "ingress authentication",
                "caller-to-policy mapping",
                "must not be inferred from the action actor",
                "fail closed",
            ],
        ),
        (
            "distributed-rate-limits-v0.md",
            &[
                "# Rava Production Distributed Rate Limits V0",
                "not implemented by the V0 preview service",
                "shared quota state",
                "caller identity",
                "cross-node",
                "fail closed",
                "accepted and rejected requests",
            ],
        ),
    ];

    for (file, required) in runbooks {
        let docs = std::fs::read_to_string(root.join("docs/operations").join(file))?;
        for phrase in required {
            assert!(docs.contains(phrase), "{file} missing: {phrase}");
        }
        assert!(
            production.contains(file) && migration.contains(file),
            "production trust and migration docs must link to {file}"
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
fn draft_release_checklist_guards_publication_without_production_claims(
) -> Result<(), Box<dyn Error>> {
    let checklist =
        std::fs::read_to_string(repository_root().join("docs/release/v0-draft-checklist.md"))?;
    let readme = std::fs::read_to_string(repository_root().join("README.md"))?;
    let audit =
        std::fs::read_to_string(repository_root().join("docs/security/release-audit-v0.md"))?;
    let compatibility = std::fs::read_to_string(
        repository_root().join("docs/protocol/compatibility-policy-v0.md"),
    )?;

    for required in [
        "# Rava V0 Draft Release Checklist",
        "not a production readiness checklist",
        "## Pre-Tag Gate",
        "cargo fmt --check",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "cargo test --workspace",
        "cargo run -p rava -- demo flight-booking",
        "cargo package --workspace",
        "cargo check --manifest-path fuzz/Cargo.toml",
        "npm --prefix packages/rava-wasm-js test",
        "## Review Artifacts",
        "docs/roadmap.md",
        "../roadmap.md",
        "docs/security/release-audit-v0.md",
        "docs/security/review-register-v0.md",
        "No external security review has been completed",
        "## Publication Guardrails",
        "must not claim production readiness",
        "## Post-Tag Verification",
    ] {
        assert!(
            checklist.contains(required),
            "missing draft release checklist docs: {required}"
        );
    }

    for docs in [readme, audit, compatibility] {
        assert!(
            docs.contains("docs/release/v0-draft-checklist.md")
                || docs.contains("../release/v0-draft-checklist.md")
                || docs.contains("../../docs/release/v0-draft-checklist.md"),
            "release docs must link to the draft release checklist"
        );
    }

    Ok(())
}

#[test]
fn release_notes_template_preserves_draft_status_and_compatibility_tracking(
) -> Result<(), Box<dyn Error>> {
    let template =
        std::fs::read_to_string(repository_root().join("docs/release/notes-template-v0.md"))?;
    let checklist =
        std::fs::read_to_string(repository_root().join("docs/release/v0-draft-checklist.md"))?;
    let compatibility = std::fs::read_to_string(
        repository_root().join("docs/protocol/compatibility-policy-v0.md"),
    )?;

    for required in [
        "# Rava V0 Draft Release Notes Template",
        "not production-ready security software",
        "## Status",
        "No external security review has been completed",
        "## Verification",
        "cargo fmt --check",
        "cargo test --workspace",
        "cargo package --workspace",
        "## Compatibility",
        "protocol version",
        "test vectors",
        "schemas",
        "rejection codes",
        "## Roadmap Status",
        "docs/roadmap.md",
        "evidence maps",
        "remaining external blockers",
        "## Known Non-Goals and External Requirements",
        "no production monitoring guarantee",
        "## Review Register",
        "docs/security/review-register-v0.md",
    ] {
        assert!(
            template.contains(required),
            "missing release notes template docs: {required}"
        );
    }

    for docs in [checklist, compatibility] {
        assert!(
            docs.contains("docs/release/notes-template-v0.md")
                || docs.contains("../release/notes-template-v0.md"),
            "release checklist and compatibility policy must link to notes template"
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
