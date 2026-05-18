# Rava Functional Roadmap

This roadmap tracks the core protocol project. It separates implemented V0 draft behavior from future work and does not turn roadmap ideas into security guarantees.

## Current Baseline

Rava V0 is a local Rust reference implementation of action-native authorization. The current baseline includes signed actions, signed capabilities, attenuated delegation checks, one-time replay registries, revocation registries, verification receipts, attestations, CLI flows, examples, test vectors, wire-shape schemas, and a preview HTTP verifier.

The baseline is useful for review, demos, compatibility testing, and early integration design. It is not production-ready security software.

## Release Readiness

Goal: make the draft protocol understandable and reviewable without implying production maturity.

Acceptance gates:

- README explains status, guarantees, assumptions, non-goals, commands, examples, and verification gates.
- Threat model distinguishes implemented guarantees from caller assumptions and non-goals.
- Release audit notes document remaining risks and what was not audited.
- All committed examples and test vectors are exercised by regression tests.
- Full local gate passes: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo run -p rava -- demo flight-booking`, `cargo package --workspace`, `cargo check --manifest-path fuzz/Cargo.toml`, `cargo check -p rava-wasm --target wasm32-unknown-unknown`, `npm --prefix packages/rava-wasm-js test`, and `(cd packages/rava-wasm-js && npm pack --dry-run)`.

Draft release checklist and release-note guidance live in [release/v0-draft-checklist.md](release/v0-draft-checklist.md) and [release/notes-template-v0.md](release/notes-template-v0.md).

The V0 draft completion audit lives in [security/v0-draft-completion-audit.md](security/v0-draft-completion-audit.md). It marks the draft reference implementation complete for review and integration experiments, while preserving the explicit non-production posture.

## V0 Hardening

Goal: strengthen the draft core without adding speculative product surface.

Candidate work:

- Keep property-style regression tests current for canonicalization, attenuation monotonicity, replay consumption, and malformed signed objects.
- Keep fuzz targets current for JSON parsing, canonicalization, and signature verification entry points.
- Keep deterministic fixture regeneration checks current so committed examples cannot drift silently.
- Keep rejection-code documentation current for operators and wrapper authors.
- Use the V0 review guide for independent review of canonicalization, signature binding, nonce validation, replay semantics, and revocation semantics.
- Keep clock-skew and expiry expectations current for verifier callers.

Hardening work must not introduce alternate cryptographic primitives or verifier shortcuts.

Property-style regression coverage currently guards canonical JSON stability, capability operation canonicalization, attenuation monotonicity, replay consumption, and malformed signed-object rejection. The local evidence lives in `crates/rava-core/src/canonical.rs`, `crates/rava-core/src/capability.rs`, and `crates/rava-core/src/verifier.rs`.

Additional V0 hardening evidence lives in `fuzz/fuzz_targets/v0_wire_entrypoints.rs`, `crates/rava-cli/tests/flight_booking.rs`, `crates/rava-cli/tests/test_vectors.rs`, `crates/rava-cli/tests/publication_docs.rs`, `docs/operators/rejection-codes-v0.md`, `docs/protocol/time-semantics-v0.md`, and `docs/security/review-guide-v0.md`.

## V1 Developer Preview

Goal: provide a stable developer-facing package while keeping Rust core semantics authoritative.

Candidate work:

- Stabilize CLI command names, JSON request and response shapes, and rejection-code subjects.
- Continue hardening the verifier service boundary beyond current request-size limits, health checks, local file-backed replay/revocation stores, local audit output, optional bearer-token ingress, and local per-process rate limiting: caller identity and distributed replay/revocation/rate-limit backends.
- Publish versioned crates and artifacts only after release gates and review notes are current.
- Keep V1 preview migration notes current for wire, CLI, service, and wrapper changes.
- Keep compatibility policy current for test vectors and schemas.

V1 developer preview still should not claim distributed trust, global identity, managed custody, or external audit coverage unless those workstreams are complete.

V1 developer preview evidence lives in `docs/protocol/v1-preview-surface.md`, `docs/protocol/v1-preview-migration.md`, `docs/protocol/compatibility-policy-v0.md`, `docs/release/v0-draft-checklist.md`, `docs/release/notes-template-v0.md`, `crates/rava-cli/tests/publication_docs.rs`, `crates/rava-cli/tests/serve_verify.rs`, `crates/rava-cli/tests/test_vectors.rs`, and `crates/rava-cli/tests/wire_schemas.rs`.

## Interop

Goal: enable wrappers and adapters around the Rust verifier without reimplementing trusted verification logic.

Sequencing:

- Keep Rust core and CLI test vectors stable.
- Keep WASM bindings around the Rust verifier compiling against `wasm32-unknown-unknown`.
- Keep the TypeScript package calling WASM and running the V0 test vectors.
- Keep DID/key-resolution examples as caller policy, not core trust.
- Keep MCP adapter proof-of-concept guidance verification-first.
- Keep OAuth exchange examples subordinate to verified Rava action context.

Detailed wrapper guidance lives in [interop/roadmap-v0.md](interop/roadmap-v0.md).

The current V1 preview surface contract is documented in [protocol/v1-preview-surface.md](protocol/v1-preview-surface.md).
V1 preview migration notes are documented in [protocol/v1-preview-migration.md](protocol/v1-preview-migration.md).

Interop evidence lives in `docs/interop/roadmap-v0.md`, `docs/interop/wasm-v0.md`, `docs/interop/typescript-v0.md`, `docs/interop/did-resolution-v0.md`, `docs/interop/mcp-adapter-v0.md`, `docs/interop/oauth-exchange-v0.md`, `crates/rava-cli/tests/interop_plan.rs`, `crates/rava-wasm/src/lib.rs`, `packages/rava-wasm-js/src/index.ts`, and `packages/rava-wasm-js/test/vectors.test.ts`.

## Production Trust and Operations

Goal: define the systems around the core verifier that production deployments need.

Candidate work:

- Key custody, rotation, recovery, and compromise response.
- Public-key discovery and resolver freshness policy.
- Distributed replay coordination.
- Distributed revocation publication, freshness, and outage policy.
- Persistent audit-log storage, retention, privacy, and export.
- Operational monitoring for verifier availability and rejection patterns.
- External security review and remediation tracking.

These items are not implemented guarantees today. They are required before Rava can be represented as a production authorization system.

Production trust requirements are defined in [operations/production-trust-v0.md](operations/production-trust-v0.md).
Detailed production runbooks live in:

- [operations/key-custody-v0.md](operations/key-custody-v0.md);
- [operations/key-discovery-v0.md](operations/key-discovery-v0.md);
- [operations/distributed-replay-v0.md](operations/distributed-replay-v0.md);
- [operations/distributed-revocation-v0.md](operations/distributed-revocation-v0.md);
- [operations/audit-storage-v0.md](operations/audit-storage-v0.md);
- [operations/caller-identity-v0.md](operations/caller-identity-v0.md);
- [operations/distributed-rate-limits-v0.md](operations/distributed-rate-limits-v0.md);
- [operations/monitoring-v0.md](operations/monitoring-v0.md).

External review findings and remediation are tracked in [security/review-register-v0.md](security/review-register-v0.md).
The external review execution plan is [security/review-plan-v0.md](security/review-plan-v0.md) (`docs/security/review-plan-v0.md`), and optional longer fuzz campaigns should use [security/fuzz-campaigns/template-v0.md](security/fuzz-campaigns/template-v0.md) (`docs/security/fuzz-campaigns/template-v0.md`).

Production trust evidence lives in `SECURITY.md`, `docs/operations/production-trust-v0.md`, `docs/operations/key-custody-v0.md`, `docs/operations/key-discovery-v0.md`, `docs/operations/distributed-replay-v0.md`, `docs/operations/distributed-revocation-v0.md`, `docs/operations/audit-storage-v0.md`, `docs/operations/caller-identity-v0.md`, `docs/operations/distributed-rate-limits-v0.md`, `docs/operations/monitoring-v0.md`, `docs/security/review-register-v0.md`, `docs/security/release-audit-v0.md`, and `crates/rava-cli/tests/publication_docs.rs`.

## Non-Goals

The core protocol roadmap does not include:

- inventing new cryptographic primitives;
- replacing OAuth, DID methods, wallets, or custody providers;
- blockchain anchoring as a core security requirement;
- reputation scoring or model-behavior proofs;
- broad product features that bypass signed action verification.

If a future feature weakens fail-closed verification, it does not belong in the core protocol.
