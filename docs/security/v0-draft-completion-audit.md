# Rava V0 Draft Completion Audit

Rava is 100% complete for the V0 draft reference implementation described by this repository. That means the local protocol core, CLI, examples, test vectors, wrapper boundary, documentation, and release gates are coherent enough to publish as a draft for review and integration experiments.

Rava V0 is not production-ready security software. No external security review has been completed. Production systems remain external requirements, not implemented guarantees in the V0 draft.

## Completion Definition

V0 draft completion means:

- the Rust core implements signed actions, delegated capabilities, attenuated delegation, replay, revocation, receipts, and attestations;
- the CLI exposes local key, demo, inspect, verify, receipt, attestation, and preview verifier flows;
- the preview HTTP verifier documents its local boundary and does not claim distributed production trust;
- committed examples and `test-vectors/v0` are regression-tested;
- the WASM wrapper and TypeScript package call the Rust verifier instead of reimplementing trusted protocol logic;
- security posture, threat model, release audit, release checklist, and roadmap documents distinguish implemented guarantees from assumptions and non-goals;
- the full local verification gate passes before a draft release is tagged or published.

## Prompt-to-Artifact Checklist

| Requirement | Evidence |
| --- | --- |
| Signed actions | `crates/rava-core/src/action.rs`, `crates/rava-core/src/verifier.rs`, `crates/rava-cli/tests/verify_action.rs` |
| Delegated capabilities | `crates/rava-core/src/capability.rs`, `crates/rava-core/src/verifier.rs`, `crates/rava-cli/tests/flight_booking.rs` |
| Attenuated delegation | `crates/rava-core/src/verifier.rs`, `docs/security/threat-model-v0.md`, `docs/operators/rejection-codes-v0.md` |
| Replay | `crates/rava-core/src/replay.rs`, `crates/rava-cli/tests/verify_action.rs`, `crates/rava-cli/tests/serve_verify.rs` |
| Revocation | `crates/rava-core/src/revocation.rs`, `crates/rava-cli/tests/verify_action.rs`, `docs/protocol/time-semantics-v0.md` |
| Receipts | `crates/rava-core/src/audit.rs`, `crates/rava-cli/tests/receipts.rs`, `examples/flight-booking/receipt.json` |
| Attestations | `crates/rava-core/src/attestation.rs`, `crates/rava-cli/tests/attestations.rs`, `examples/flight-booking/attestation.json` |
| CLI flows | `README.md`, `crates/rava-cli/tests`, `examples/flight-booking` |
| Preview HTTP verifier | `docs/protocol/v1-preview-surface.md`, `crates/rava-cli/tests/serve_verify.rs` |
| WASM wrapper | `crates/rava-wasm/src/lib.rs`, `docs/interop/wasm-v0.md` |
| TypeScript package | `packages/rava-wasm-js`, `docs/interop/typescript-v0.md` |
| Test vectors | `test-vectors/v0`, `crates/rava-cli/tests/test_vectors.rs` |
| Release posture | `docs/security/release-audit-v0.md`, `docs/release/v0-draft-checklist.md`, `docs/release/notes-template-v0.md` |
| Roadmap | `docs/roadmap.md`, `docs/interop/roadmap-v0.md` |

## Verification Gate

Before claiming a V0 draft release candidate is ready, run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p rava -- demo flight-booking
cargo package --workspace
cargo check --manifest-path fuzz/Cargo.toml --locked
cargo check -p rava-wasm --target wasm32-unknown-unknown
npm --prefix packages/rava-wasm-js test
(cd packages/rava-wasm-js && npm pack --dry-run)
```

The gate proves the draft reference implementation, examples, wrappers, and package contents are internally coherent. It does not replace external cryptographic review, deployment review, or production operations work.

## Remaining Non-Draft Work

The remaining work is outside the V0 draft implementation:

- external security review and remediation tracking;
- optional long-running fuzz campaigns recorded with corpus, duration, and results;
- tag and publication decisions for crates, wrapper packages, and release assets;
- production key custody, public-key discovery, distributed replay, distributed revocation, caller identity, distributed rate limits, monitoring, and managed audit storage.

Those systems are documented in `docs/operations`, but they are not implemented production systems in V0.
