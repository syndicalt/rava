use std::error::Error;
use std::path::{Path, PathBuf};

#[test]
fn readme_states_publication_posture_and_operator_path() -> Result<(), Box<dyn Error>> {
    let readme = std::fs::read_to_string(repository_root().join("README.md"))?;

    for required in [
        "Rava V0 is a draft reference implementation, not production-ready security software.",
        "Rava V0 is complete as a draft reference implementation for review, examples, interop work, and integration design.",
        "## Table of Contents",
        "## Requirements",
        "## Quickstart",
        "## Repository Layout",
        "## Verification Gates",
        "docs/roadmap.md",
        "docs/release/v0-draft-checklist.md",
        "docs/release/notes-template-v0.md",
        "docs/protocol/v1-preview-surface.md",
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
        "SECURITY.md",
        "cargo fmt --check",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "cargo test --workspace",
        "cargo run -p rava -- demo flight-booking",
        "cargo package --workspace",
        "cargo check --manifest-path fuzz/Cargo.toml",
        "cargo check -p rava-wasm --target wasm32-unknown-unknown",
        "npm --prefix packages/rava-wasm-js test",
        "npm pack --dry-run",
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
    assert!(!readme.contains(
        "This repository should be treated as a draft protocol package until the roadmap release-readiness items are complete."
    ));

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
        "npm pack --dry-run",
    ] {
        assert!(roadmap.contains(required), "roadmap missing: {required}");
    }

    Ok(())
}

#[test]
fn v0_draft_completion_audit_maps_completion_to_evidence() -> Result<(), Box<dyn Error>> {
    let root = repository_root();
    let audit = std::fs::read_to_string(root.join("docs/security/v0-draft-completion-audit.md"))?;
    let readme = std::fs::read_to_string(root.join("README.md"))?;
    let roadmap = std::fs::read_to_string(root.join("docs/roadmap.md"))?;

    for required in [
        "# Rava V0 Draft Completion Audit",
        "100% complete for the V0 draft reference implementation",
        "not production-ready security software",
        "No external security review has been completed",
        "Production systems remain external requirements",
        "## Completion Definition",
        "## Prompt-to-Artifact Checklist",
        "## Verification Gate",
        "## Remaining Non-Draft Work",
        "signed actions",
        "delegated capabilities",
        "attenuated delegation",
        "replay",
        "revocation",
        "receipts",
        "attestations",
        "CLI flows",
        "preview HTTP verifier",
        "WASM wrapper",
        "TypeScript package",
        "test-vectors/v0",
        "examples/flight-booking",
        "docs/security/threat-model-v0.md",
        "docs/security/release-audit-v0.md",
        "docs/release/v0-draft-checklist.md",
        "docs/roadmap.md",
        "cargo fmt --check",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "cargo test --workspace",
        "cargo run -p rava -- demo flight-booking",
        "cargo package --workspace",
        "cargo check --manifest-path fuzz/Cargo.toml",
        "cargo check -p rava-wasm --target wasm32-unknown-unknown",
        "npm --prefix packages/rava-wasm-js test",
        "npm pack --dry-run",
    ] {
        assert!(
            audit.contains(required),
            "completion audit missing: {required}"
        );
    }

    assert!(
        readme.contains("docs/security/v0-draft-completion-audit.md"),
        "README must link to the V0 draft completion audit"
    );
    assert!(
        roadmap.contains("security/v0-draft-completion-audit.md"),
        "roadmap must link to the V0 draft completion audit"
    );
    assert!(!audit.contains("Rava is production-ready"));

    Ok(())
}

#[test]
fn github_pages_site_explains_lifecycle_and_links_protocol_docs() -> Result<(), Box<dyn Error>> {
    let root = repository_root();
    let site = std::fs::read_to_string(root.join("site/index.html"))?;
    let styles = std::fs::read_to_string(root.join("site/styles.css"))?;
    let script = std::fs::read_to_string(root.join("site/script.js"))?;
    let workflow = std::fs::read_to_string(root.join(".github/workflows/pages.yml"))?;

    for required in [
        "<title>Rava",
        "Action-native authorization for autonomous agents",
        "Most auth asks who logged in",
        "Rava asks whether this exact signed action is allowed",
        "not production-ready security software",
        "Problem",
        "Proposed Solution",
        "Full Lifecycle",
        "Documentation",
        "Roadmap",
        "https://github.com/syndicalt/rava",
        "docs/protocol/rava-v0.md",
        "docs/security/threat-model-v0.md",
        "docs/security/v0-draft-completion-audit.md",
        "docs/roadmap.md",
        "docs/operations/production-trust-v0.md",
        "cargo run -p rava -- demo flight-booking",
        "rava verify action",
        "rava verify receipt",
        "rava attest sign",
        "cargo fmt --check",
        "cargo test --workspace",
        "cargo package --workspace",
        "signed action",
        "capability chain",
        "verification receipt",
        "post-action attestation",
    ] {
        assert!(site.contains(required), "site missing: {required}");
    }

    for required in [
        "--ink",
        "--paper",
        "font-family",
        ".lifecycle",
        ".protocol-card",
        ".doc-grid",
        "@media",
    ] {
        assert!(styles.contains(required), "site CSS missing: {required}");
    }

    for required in [
        "data-copy-command",
        "navigator.clipboard.writeText",
        "aria-label",
    ] {
        assert!(script.contains(required), "site script missing: {required}");
    }

    for required in [
        "Deploy Rava Pages",
        "github-pages",
        "actions/configure-pages",
        "enablement: true",
        "actions/upload-pages-artifact",
        "actions/deploy-pages",
        "path: site",
    ] {
        assert!(
            workflow.contains(required),
            "Pages workflow missing: {required}"
        );
    }

    assert!(!site.contains("Rava is production-ready"));
    assert!(!site.contains("blockchain-backed"));

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
        "SECURITY.md",
        "Vulnerability reporting and external review intake are documented in [../../SECURITY.md](../../SECURITY.md).",
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
        "npm pack --dry-run",
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
        "caller identity",
        "distributed rate limiting",
        "managed audit storage",
        "production monitoring",
        "../operations/production-trust-v0.md",
        "../operations/key-custody-v0.md",
        "../operations/key-discovery-v0.md",
        "../operations/distributed-replay-v0.md",
        "../operations/distributed-revocation-v0.md",
        "../operations/audit-storage-v0.md",
        "../operations/caller-identity-v0.md",
        "../operations/distributed-rate-limits-v0.md",
        "../operations/monitoring-v0.md",
        "## Evidence Map",
        "review-plan-v0.md",
        "fuzz-campaigns/template-v0.md",
        "crates/rava-core/src/verifier.rs",
        "docs/security/threat-model-v0.md",
        "cargo package --workspace",
        "cargo check --manifest-path fuzz/Cargo.toml",
        "cargo check -p rava-wasm --target wasm32-unknown-unknown",
        "npm --prefix packages/rava-wasm-js test",
        "npm pack --dry-run",
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
fn security_review_plan_and_fuzz_template_define_external_review_process(
) -> Result<(), Box<dyn Error>> {
    let root = repository_root();
    let plan = std::fs::read_to_string(root.join("docs/security/review-plan-v0.md"))?;
    let template =
        std::fs::read_to_string(root.join("docs/security/fuzz-campaigns/template-v0.md"))?;
    let checklist = std::fs::read_to_string(root.join("docs/release/v0-draft-checklist.md"))?;
    let register = std::fs::read_to_string(root.join("docs/security/review-register-v0.md"))?;
    let roadmap = std::fs::read_to_string(root.join("docs/roadmap.md"))?;

    for required in [
        "# Rava V0 External Security Review Plan",
        "not evidence that Rava has been externally audited",
        "## Review Target",
        "Record the exact commit SHA",
        "## Reviewer Packet",
        "docs/security/threat-model-v0.md",
        "docs/protocol/rava-v0.md",
        "docs/security/v0-draft-completion-audit.md",
        "docs/security/review-guide-v0.md",
        "docs/security/review-register-v0.md",
        "## In Scope",
        "canonicalization",
        "signature binding",
        "delegation attenuation",
        "replay semantics",
        "revocation semantics",
        "receipt and attestation verification",
        "fail-closed behavior",
        "## Out of Scope",
        "production key custody",
        "distributed replay",
        "distributed revocation",
        "managed audit storage",
        "## Finding Intake",
        "RAVA-REVIEW-001",
        "reported",
        "accepted",
        "remediated",
        "verified",
        "accepted-risk",
        "out-of-scope",
        "## Release Rule",
        "No finding that weakens fail-closed verification may remain unresolved",
        "## Fuzz Evidence",
        "docs/security/fuzz-campaigns/template-v0.md",
    ] {
        assert!(plan.contains(required), "review plan missing: {required}");
    }

    for required in [
        "# Rava V0 Fuzz Campaign Log Template",
        "not a proof of security",
        "## Campaign Metadata",
        "Commit SHA",
        "Command",
        "cargo fuzz run v0_wire_entrypoints -- -max_total_time=86400",
        "Duration",
        "Host",
        "Corpus path",
        "## Coverage Intent",
        "JSON parsing",
        "canonicalization",
        "action verification",
        "receipt verification",
        "attestation verification",
        "## Results",
        "Crash count",
        "Minimized crashing inputs",
        "## Remediation",
        "Regression tests",
        "Pull requests",
        "## Final Rerun",
    ] {
        assert!(
            template.contains(required),
            "fuzz template missing: {required}"
        );
    }

    for required in [
        "docs/security/review-plan-v0.md",
        "docs/security/fuzz-campaigns/template-v0.md",
    ] {
        assert!(
            checklist.contains(required)
                && roadmap.contains(required)
                && register.contains(required),
            "checklist, roadmap, and review register must link review process artifact: {required}"
        );
    }

    Ok(())
}

#[test]
fn bounded_fuzz_campaign_log_records_v0_wire_entrypoints_evidence() -> Result<(), Box<dyn Error>> {
    let root = repository_root();
    let campaign_path = "docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints.md";
    let campaign = std::fs::read_to_string(root.join(campaign_path))?;
    let audit = std::fs::read_to_string(root.join("docs/security/release-audit-v0.md"))?;

    for required in [
        "# Rava V0 Fuzz Campaign: v0_wire_entrypoints 2026-05-18",
        "not a proof of security, external audit, or production readiness certification",
        "83bf11da3078c643acabb35ccec409725b4f95a2",
        "cargo +nightly fuzz run v0_wire_entrypoints -- -max_total_time=600",
        "cargo-fuzz version: `0.13.1`",
        "Seed: `3852110329`",
        "JSON parsing",
        "canonicalization",
        "action verification",
        "receipt verification",
        "attestation verification",
        "Total executions: 20,742,375",
        "Final coverage: `cov: 2343`",
        "Final feature count: `ft: 9732`",
        "Final corpus: `2315/827Kb`",
        "Crash count: 0",
        "Timeout count: 0",
        "OOM count: 0",
        "Done 20742375 runs in 601 second(s)",
        "No crash or bug was found",
    ] {
        assert!(
            campaign.contains(required),
            "campaign log missing: {required}"
        );
    }

    assert!(
        audit.contains(campaign_path),
        "release audit must link bounded fuzz campaign evidence"
    );
    assert!(
        audit.contains("recurring or overnight fuzz campaigns are not part of the default gate"),
        "release audit must preserve the distinction between bounded fuzz evidence and ongoing campaigns"
    );

    Ok(())
}

#[test]
fn longer_fuzz_campaign_log_records_v0_wire_entrypoints_evidence() -> Result<(), Box<dyn Error>> {
    let root = repository_root();
    let campaign_path = "docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints-1800s.md";
    let campaign = std::fs::read_to_string(root.join(campaign_path))?;
    let audit = std::fs::read_to_string(root.join("docs/security/release-audit-v0.md"))?;

    for required in [
        "# Rava V0 Fuzz Campaign: v0_wire_entrypoints 2026-05-18 1800s",
        "not a proof of security, external audit, or production readiness certification",
        "does not change the frozen external-review target",
        "becbff9e2326f5304822decf636aadcd0e37bb48",
        "cargo +nightly fuzz run v0_wire_entrypoints -- -max_total_time=1800",
        "Duration: 1801 seconds",
        "Start time: `2026-05-18T03:25:30Z`",
        "cargo-fuzz version: `0.13.1`",
        "Corpus path: `fuzz/corpus/v0_wire_entrypoints` (generated locally; not committed)",
        "Artifact path: `fuzz/artifacts/v0_wire_entrypoints`",
        "Seed: `2752387297`",
        "JSON parsing",
        "canonicalization",
        "action verification",
        "receipt verification",
        "attestation verification",
        "Total executions: 58,358,035",
        "Final coverage: `cov: 2657`",
        "Final feature count: `ft: 10540`",
        "Final corpus: `2548/1089Kb`",
        "Input limit: `4096`",
        "Final exec/s: `32403`",
        "Final RSS: `736Mb`",
        "Crash count: 0",
        "Timeout count: 0",
        "OOM count: 0",
        "Minimized crashing inputs: none",
        "Sanitizer or panic output: none observed",
        "Final line: `Done 58358035 runs in 1801 second(s)`",
        "No crash or bug was found",
    ] {
        assert!(
            campaign.contains(required),
            "longer campaign log missing: {required}"
        );
    }

    assert!(
        audit.contains(campaign_path),
        "release audit must link longer fuzz campaign evidence"
    );
    assert!(
        audit.contains("not part of the default gate"),
        "release audit must preserve the distinction between campaign evidence and the default gate"
    );

    Ok(())
}

#[test]
fn external_review_packet_defines_frozen_handoff_manifest() -> Result<(), Box<dyn Error>> {
    let root = repository_root();
    let packet_path = "docs/security/external-review-packet-v0.md";
    let packet = std::fs::read_to_string(root.join(packet_path))?;
    let plan = std::fs::read_to_string(root.join("docs/security/review-plan-v0.md"))?;
    let guide = std::fs::read_to_string(root.join("docs/security/review-guide-v0.md"))?;
    let checklist = std::fs::read_to_string(root.join("docs/release/v0-draft-checklist.md"))?;

    for required in [
        "# Rava V0 External Review Packet",
        "not evidence that Rava has been externally reviewed",
        "## Freeze Rule",
        "immutable commit SHA or signed tag",
        "Do not change the target during review",
        "## Packet Manifest",
        "README.md",
        "docs/security/threat-model-v0.md",
        "docs/protocol/rava-v0.md",
        "docs/security/review-guide-v0.md",
        "docs/security/review-register-v0.md",
        "docs/security/release-audit-v0.md",
        "docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints.md",
        "test-vectors/v0",
        "examples/flight-booking",
        "## Verification Baseline",
        "cargo fmt --check",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "cargo test --workspace",
        "cargo run -p rava -- demo flight-booking",
        "cargo package --workspace",
        "cargo check --manifest-path fuzz/Cargo.toml",
        "cargo check -p rava-wasm --target wasm32-unknown-unknown",
        "npm --prefix packages/rava-wasm-js test",
        "(cd packages/rava-wasm-js && npm pack --dry-run)",
        "## Finding Handling",
        "RAVA-REVIEW-001",
        "reported",
        "accepted",
        "remediated",
        "verified",
        "accepted-risk",
        "out-of-scope",
        "No finding that weakens fail-closed verification may remain unresolved",
    ] {
        assert!(
            packet.contains(required),
            "external review packet missing: {required}"
        );
    }

    for docs in [plan, guide, checklist] {
        assert!(
            docs.contains(packet_path),
            "review docs must link external packet manifest: {packet_path}"
        );
    }

    Ok(())
}

#[test]
fn external_review_packet_links_post_candidate_fuzz_evidence_without_moving_target(
) -> Result<(), Box<dyn Error>> {
    let packet = std::fs::read_to_string(
        repository_root().join("docs/security/external-review-packet-v0.md"),
    )?;

    for required in [
        "## Additional Post-Candidate Evidence",
        "docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints-1800s.md",
        "becbff9e2326f5304822decf636aadcd0e37bb48",
        "does not change the frozen review target",
        "should be treated as supplemental review evidence",
        "not a proof of security",
        "not evidence that Rava has been externally reviewed",
    ] {
        assert!(
            packet.contains(required),
            "external review packet missing post-candidate evidence boundary: {required}"
        );
    }

    Ok(())
}

#[test]
fn external_review_cover_note_preserves_scope_and_non_production_boundary(
) -> Result<(), Box<dyn Error>> {
    let root = repository_root();
    let cover_path = "docs/security/external-review-cover-note-v0.md";
    let cover = std::fs::read_to_string(root.join(cover_path))?;
    let packet = std::fs::read_to_string(root.join("docs/security/external-review-packet-v0.md"))?;
    let plan = std::fs::read_to_string(root.join("docs/security/review-plan-v0.md"))?;

    for required in [
        "# Rava V0 External Review Cover Note",
        "not production-ready security software",
        "not evidence that Rava has been externally reviewed",
        "v0-review-candidate-2026-05-18",
        "0672e61fcf46b472aee4e32d1915a0c975a0bbda",
        "https://github.com/syndicalt/rava/issues/87",
        "docs/security/external-review-packet-v0.md",
        "docs/security/threat-model-v0.md",
        "docs/protocol/rava-v0.md",
        "docs/security/review-guide-v0.md",
        "docs/security/review-register-v0.md",
        ".github/ISSUE_TEMPLATE/security-review-finding.yml",
        "canonicalization",
        "signature binding",
        "delegation attenuation",
        "replay semantics",
        "revocation semantics",
        "receipt and attestation verification",
        "fail-closed behavior",
        "production key custody",
        "distributed replay",
        "distributed revocation",
        "caller identity",
        "cargo fmt --check",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "cargo test --workspace",
        "cargo run -p rava -- demo flight-booking",
        "cargo package --workspace",
        "cargo check --manifest-path fuzz/Cargo.toml",
        "cargo check -p rava-wasm --target wasm32-unknown-unknown",
        "npm --prefix packages/rava-wasm-js test",
        "(cd packages/rava-wasm-js && npm pack --dry-run)",
    ] {
        assert!(
            cover.contains(required),
            "external review cover note missing: {required}"
        );
    }

    for docs in [packet, plan] {
        assert!(
            docs.contains(cover_path),
            "review handoff docs must link cover note: {cover_path}"
        );
    }

    Ok(())
}

#[test]
fn security_review_finding_template_requires_remediation_evidence() -> Result<(), Box<dyn Error>> {
    let root = repository_root();
    let template_path = "docs/security/review-findings/template-v0.md";
    let template = std::fs::read_to_string(root.join(template_path))?;
    let register = std::fs::read_to_string(root.join("docs/security/review-register-v0.md"))?;
    let packet = std::fs::read_to_string(root.join("docs/security/external-review-packet-v0.md"))?;

    for required in [
        "# Rava V0 Security Review Finding Template",
        "not evidence that Rava has been externally reviewed",
        "RAVA-REVIEW-001",
        "## Target",
        "Immutable commit SHA or signed tag",
        "## Classification",
        "protocol correctness issue",
        "implementation bug",
        "documentation ambiguity",
        "test coverage gap",
        "V0 non-goal or future production requirement",
        "## Triage",
        "reported",
        "accepted",
        "remediated",
        "verified",
        "accepted-risk",
        "out-of-scope",
        "## Impact",
        "fail-closed verification",
        "## Remediation Plan",
        "## Required Regression Evidence",
        "Regression tests",
        "Documentation changes",
        "## Verification Evidence",
        "cargo fmt --check",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "cargo test --workspace",
        "cargo run -p rava -- demo flight-booking",
        "cargo package --workspace",
        "cargo check --manifest-path fuzz/Cargo.toml",
        "cargo check -p rava-wasm --target wasm32-unknown-unknown",
        "npm --prefix packages/rava-wasm-js test",
        "(cd packages/rava-wasm-js && npm pack --dry-run)",
        "## Register Update",
        "docs/security/review-register-v0.md",
    ] {
        assert!(
            template.contains(required),
            "finding template missing: {required}"
        );
    }

    for docs in [register, packet] {
        assert!(
            docs.contains(template_path),
            "review tracking docs must link finding template: {template_path}"
        );
    }

    Ok(())
}

#[test]
fn github_issue_template_routes_security_review_findings_to_remediation_tracking(
) -> Result<(), Box<dyn Error>> {
    let root = repository_root();
    let issue_template =
        std::fs::read_to_string(root.join(".github/ISSUE_TEMPLATE/security-review-finding.yml"))?;
    let register = std::fs::read_to_string(root.join("docs/security/review-register-v0.md"))?;

    for required in [
        "name: Rava security review finding",
        "description: Record an external review finding or remediation item for Rava V0",
        "labels: [security-review]",
        "not evidence that Rava has been externally reviewed",
        "docs/security/review-register-v0.md",
        "docs/security/review-findings/template-v0.md",
        "RAVA-REVIEW-",
        "Immutable review target",
        "Finding state",
        "reported",
        "accepted",
        "remediated",
        "verified",
        "accepted-risk",
        "out-of-scope",
        "Affected area",
        "Impact on fail-closed verification",
        "Regression evidence",
        "Verification commands",
        "cargo fmt --check",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "cargo test --workspace",
        "cargo run -p rava -- demo flight-booking",
        "cargo package --workspace",
        "cargo check --manifest-path fuzz/Cargo.toml",
        "cargo check -p rava-wasm --target wasm32-unknown-unknown",
        "npm --prefix packages/rava-wasm-js test",
        "(cd packages/rava-wasm-js && npm pack --dry-run)",
    ] {
        assert!(
            issue_template.contains(required),
            "security review issue template missing: {required}"
        );
    }

    assert!(
        register.contains("reported")
            && register.contains("accepted")
            && register.contains("remediated")
            && register.contains("verified")
            && register.contains("accepted-risk")
            && register.contains("out-of-scope"),
        "issue template states must stay aligned with the review register"
    );

    Ok(())
}

#[test]
fn v0_review_candidate_notes_record_frozen_target_without_release_claims(
) -> Result<(), Box<dyn Error>> {
    let root = repository_root();
    let notes_path = "docs/release/v0-review-candidate-2026-05-18.md";
    let notes = std::fs::read_to_string(root.join(notes_path))?;
    let packet = std::fs::read_to_string(root.join("docs/security/external-review-packet-v0.md"))?;
    let checklist = std::fs::read_to_string(root.join("docs/release/v0-draft-checklist.md"))?;

    for required in [
        "# Rava V0 Review Candidate Notes: 2026-05-18",
        "not a production release",
        "not evidence that an external security review has been completed",
        "v0-review-candidate-2026-05-18",
        "0672e61fcf46b472aee4e32d1915a0c975a0bbda",
        "External security review: V0 review candidate",
        "https://github.com/syndicalt/rava/issues/87",
        "docs/security/external-review-packet-v0.md",
        "docs/security/review-findings/template-v0.md",
        "docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints.md",
        "cargo fmt --check",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "cargo test --workspace",
        "cargo run -p rava -- demo flight-booking",
        "cargo package --workspace",
        "cargo check --manifest-path fuzz/Cargo.toml",
        "cargo check -p rava-wasm --target wasm32-unknown-unknown",
        "npm --prefix packages/rava-wasm-js test",
        "(cd packages/rava-wasm-js && npm pack --dry-run)",
        "master CI run `26011396149`",
        "Pages run `26011396147`",
        "No external security review has been completed",
    ] {
        assert!(
            notes.contains(required),
            "review candidate notes missing: {required}"
        );
    }

    for docs in [packet, checklist] {
        assert!(
            docs.contains(notes_path),
            "review packet and checklist must link review candidate notes: {notes_path}"
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
    let security_policy = std::fs::read_to_string(repository_root().join("SECURITY.md"))?;
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
        "caller identity",
        "distributed rate limiting",
        "../operations/key-custody-v0.md",
        "../operations/key-discovery-v0.md",
        "../operations/distributed-replay-v0.md",
        "../operations/distributed-revocation-v0.md",
        "../operations/audit-storage-v0.md",
        "../operations/caller-identity-v0.md",
        "../operations/distributed-rate-limits-v0.md",
        "../operations/monitoring-v0.md",
        ".github/ISSUE_TEMPLATE/security-review-finding.yml",
    ] {
        assert!(
            register.contains(required),
            "missing security review register docs: {required}"
        );
    }

    for required in [
        "# Security Policy",
        "Rava V0 is a draft reference implementation",
        "No external security review has been completed",
        "Do not include private keys, credentials, access tokens, or raw sensitive action payloads",
        "docs/security/review-register-v0.md",
        "docs/security/review-guide-v0.md",
        ".github/ISSUE_TEMPLATE/security-review-finding.yml",
        "docs/operations/key-custody-v0.md",
        "docs/operations/key-discovery-v0.md",
        "docs/operations/distributed-replay-v0.md",
        "docs/operations/distributed-revocation-v0.md",
        "docs/operations/audit-storage-v0.md",
        "docs/operations/caller-identity-v0.md",
        "docs/operations/distributed-rate-limits-v0.md",
        "docs/operations/monitoring-v0.md",
    ] {
        assert!(
            security_policy.contains(required),
            "missing security policy docs: {required}"
        );
    }

    for docs in [guide, audit, production, security_policy] {
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
        "## HTTP Error Shape",
        "401 Unauthorized",
        "429 Too Many Requests",
        "413 Payload Too Large",
        "431 Request Header Fields Too Large",
        "404 Not Found",
        "`error`",
        "## Audit Log Shape",
        "action_id",
        "verified_at_unix",
        "## Rejection-Code Subjects",
        "`auth_required`",
        "`rate_limit_per_minute`",
        "docs/operators/rejection-codes-v0.md",
        "not a production authorization boundary",
        "../operations/key-custody-v0.md",
        "../operations/key-discovery-v0.md",
        "../operations/distributed-replay-v0.md",
        "../operations/distributed-revocation-v0.md",
        "../operations/audit-storage-v0.md",
        "../operations/caller-identity-v0.md",
        "../operations/distributed-rate-limits-v0.md",
        "../operations/monitoring-v0.md",
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
        "caller identity and distributed rate limiting",
    ] {
        assert!(
            docs.contains(required),
            "missing production trust docs: {required}"
        );
    }

    for required in [
        "Production trust evidence lives in `SECURITY.md`, `docs/operations/production-trust-v0.md`, `docs/operations/key-custody-v0.md`, `docs/operations/key-discovery-v0.md`, `docs/operations/distributed-replay-v0.md`, `docs/operations/distributed-revocation-v0.md`, `docs/operations/audit-storage-v0.md`, `docs/operations/caller-identity-v0.md`, `docs/operations/distributed-rate-limits-v0.md`, `docs/operations/monitoring-v0.md`, `docs/security/review-register-v0.md`, `docs/security/release-audit-v0.md`, and `crates/rava-cli/tests/publication_docs.rs`.",
        "SECURITY.md",
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
        "cargo check -p rava-wasm --target wasm32-unknown-unknown",
        "npm --prefix packages/rava-wasm-js test",
        "(cd packages/rava-wasm-js && npm pack --dry-run)",
        "## Optional Fuzz Campaign",
        "cargo fuzz run v0_wire_entrypoints",
        "corpus, duration, and any minimized crash input",
        "Long-running fuzz campaigns are not part of the default gate",
        "## Review Artifacts",
        "docs/roadmap.md",
        "SECURITY.md",
        "../roadmap.md",
        "docs/security/release-audit-v0.md",
        "docs/security/review-register-v0.md",
        "docs/operations/production-trust-v0.md",
        "docs/operations/key-custody-v0.md",
        "docs/operations/key-discovery-v0.md",
        "docs/operations/distributed-replay-v0.md",
        "docs/operations/distributed-revocation-v0.md",
        "docs/operations/audit-storage-v0.md",
        "docs/operations/caller-identity-v0.md",
        "docs/operations/distributed-rate-limits-v0.md",
        "docs/operations/monitoring-v0.md",
        "../protocol/v1-preview-surface.md",
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
        "npm pack --dry-run",
        "## Fuzz Campaign",
        "cargo fuzz run v0_wire_entrypoints",
        "corpus, duration, and any minimized crash input",
        "not run",
        "## Compatibility",
        "protocol version",
        "test vectors",
        "schemas",
        "rejection codes",
        "preview service request, response, health, audit, or error shapes",
        "## Roadmap Status",
        "docs/roadmap.md",
        "docs/operations/production-trust-v0.md",
        "docs/operations/key-custody-v0.md",
        "docs/operations/key-discovery-v0.md",
        "docs/operations/distributed-replay-v0.md",
        "docs/operations/distributed-revocation-v0.md",
        "docs/operations/audit-storage-v0.md",
        "docs/operations/caller-identity-v0.md",
        "docs/operations/distributed-rate-limits-v0.md",
        "docs/operations/monitoring-v0.md",
        "evidence maps",
        "remaining external blockers",
        "## Known Non-Goals and External Requirements",
        "no production monitoring guarantee",
        "## Review Register",
        "SECURITY.md",
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
    let ts_package =
        std::fs::read_to_string(repository_root().join("packages/rava-wasm-js/package.json"))?;

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
    assert!(ts_package.contains(r#""license": "Apache-2.0""#));
    assert!(ts_package.contains(r#""url": "https://github.com/syndicalt/rava.git""#));
    assert!(ts_package.contains(r#""files""#));
    assert!(ts_package.contains(r#""dist""#));
    Ok(())
}

#[test]
fn release_artifact_ignores_exclude_generated_outputs() -> Result<(), Box<dyn Error>> {
    let gitignore = std::fs::read_to_string(repository_root().join(".gitignore"))?;

    for required in ["target/", "node_modules/", "dist/", "*.tgz"] {
        assert!(
            gitignore.contains(required),
            ".gitignore missing generated artifact ignore: {required}"
        );
    }

    Ok(())
}

#[test]
fn ci_workflow_runs_the_documented_local_gate() -> Result<(), Box<dyn Error>> {
    let workflow = std::fs::read_to_string(repository_root().join(".github/workflows/ci.yml"))?;

    for required in [
        "cargo fmt --check",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "cargo test --workspace",
        "cargo run -p rava -- demo flight-booking",
        "cargo package --workspace",
        "cargo check --manifest-path fuzz/Cargo.toml",
        "cargo check -p rava-wasm --target wasm32-unknown-unknown",
        "npm --prefix packages/rava-wasm-js test",
        "npm pack --dry-run",
        "actions/checkout@v6",
        "actions/setup-node@v6",
    ] {
        assert!(
            workflow.contains(required),
            "CI workflow missing local gate command: {required}"
        );
    }

    Ok(())
}

#[test]
fn codeowners_marks_security_sensitive_protocol_surfaces_for_review() -> Result<(), Box<dyn Error>>
{
    let root = repository_root();
    let codeowners = std::fs::read_to_string(root.join(".github/CODEOWNERS"))?;
    let security_policy = std::fs::read_to_string(root.join("SECURITY.md"))?;
    let review_plan = std::fs::read_to_string(root.join("docs/security/review-plan-v0.md"))?;

    for required in [
        "# Rava security-sensitive review ownership",
        "@syndicalt",
        "/crates/rava-core/",
        "/crates/rava-cli/src/",
        "/crates/rava-wasm/",
        "/packages/rava-wasm-js/",
        "/fuzz/",
        "/docs/protocol/",
        "/docs/security/",
        "/docs/operations/",
        "/docs/release/",
        "/test-vectors/",
        "/examples/flight-booking/",
        "/.github/workflows/",
        "/.github/ISSUE_TEMPLATE/security-review-finding.yml",
        "/SECURITY.md",
        "/README.md",
    ] {
        assert!(
            codeowners.contains(required),
            "CODEOWNERS missing security-sensitive surface: {required}"
        );
    }

    for docs in [security_policy, review_plan] {
        assert!(
            docs.contains(".github/CODEOWNERS"),
            "security intake docs must link CODEOWNERS review ownership"
        );
    }

    Ok(())
}

#[test]
fn pull_request_template_requires_security_boundary_and_gate_evidence() -> Result<(), Box<dyn Error>>
{
    let root = repository_root();
    let template = std::fs::read_to_string(root.join(".github/pull_request_template.md"))?;
    let security_policy = std::fs::read_to_string(root.join("SECURITY.md"))?;
    let review_plan = std::fs::read_to_string(root.join("docs/security/review-plan-v0.md"))?;

    for required in [
        "## Summary",
        "## Security Boundary",
        "Does this change affect canonicalization, signing, verification, expiry, revocation, replay, receipts, attestations, wrappers, workflows, release docs, or security docs?",
        "No new cryptographic primitives",
        "No verifier shortcuts",
        "No test-only bypasses",
        "No production-ready or externally audited claim",
        "## Required Evidence",
        "cargo fmt --check",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "cargo test --workspace",
        "cargo run -p rava -- demo flight-booking",
        "cargo package --workspace",
        "cargo check --manifest-path fuzz/Cargo.toml",
        "cargo check -p rava-wasm --target wasm32-unknown-unknown",
        "npm --prefix packages/rava-wasm-js test",
        "(cd packages/rava-wasm-js && npm pack --dry-run)",
        "## Review Artifacts",
        "docs/security/threat-model-v0.md",
        "docs/security/review-register-v0.md",
        ".github/CODEOWNERS",
        ".github/ISSUE_TEMPLATE/security-review-finding.yml",
    ] {
        assert!(
            template.contains(required),
            "pull request template missing: {required}"
        );
    }

    for docs in [security_policy, review_plan] {
        assert!(
            docs.contains(".github/pull_request_template.md"),
            "security docs must link PR template guardrails"
        );
    }

    Ok(())
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
