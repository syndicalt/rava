# Rava V0 Security Review Guide

This guide prepares Rava V0 for outside protocol and security review. Rava V0 is a draft reference implementation, not production-ready security software.

## Review Scope

In scope:

- canonicalization of signed JSON payloads;
- signer ID and public-key binding;
- Ed25519 signature verification;
- signed action and capability ID derivation;
- nonce validation for signed protocol objects;
- delegation attenuation;
- replay semantics;
- revocation semantics;
- receipt and attestation verification;
- CLI and fixture behavior that exercises the Rust core.

Out of scope for V0 review:

- key custody, rotation, and recovery;
- DID resolution and public-key discovery;
- distributed replay coordination;
- distributed revocation freshness;
- caller identity;
- distributed rate limiting;
- managed audit storage;
- production monitoring;
- hosted verifier operations;
- model-behavior proofs, reputation, or blockchain anchoring.

Production deployment requirements outside V0 review scope are documented in [../operations/production-trust-v0.md](../operations/production-trust-v0.md).

## High-Value Review Questions

- Does canonicalization produce one stable byte string for each signed payload?
- Are all security-relevant fields included in the signing payload and derived ID?
- Does signature binding cover the signer ID, supplied public key, signed fields, and derived ID?
- Can a validly signed child capability broaden any parent attenuation dimension?
- Are malformed or non-canonical nonces rejected before relying on signed objects?
- Does replay recording happen only after an action is accepted and durably consumed?
- Do replay failures fail closed instead of reporting successful one-time verification?
- Do revocation checks cover action actors, capability issuers, and capability IDs?
- Are receipt and attestation IDs/signatures bound to the fields operators rely on?
- Are caller assumptions about authentic public keys and fresh registries documented clearly?

## Evidence Map

| Review area | Primary artifact |
| --- | --- |
| Threat model | `docs/security/threat-model-v0.md` |
| Protocol spec | `docs/protocol/rava-v0.md` |
| Time semantics | `docs/protocol/time-semantics-v0.md` |
| Rejection codes | `docs/operators/rejection-codes-v0.md` |
| Compatibility policy | `docs/protocol/compatibility-policy-v0.md` |
| Canonicalization | `crates/rava-core/src/canonical.rs` |
| Signer binding | `crates/rava-core/src/identity.rs` |
| Actions | `crates/rava-core/src/action.rs` |
| Capabilities and delegation | `crates/rava-core/src/capability.rs` |
| Verification | `crates/rava-core/src/verifier.rs` |
| Replay registry | `crates/rava-core/src/replay.rs` |
| Revocation registry | `crates/rava-core/src/revocation.rs` |
| Receipts | `crates/rava-core/src/audit.rs` |
| Attestations | `crates/rava-core/src/attestation.rs` |
| CLI fixtures | `crates/rava-cli/src/demo.rs` |
| Test vectors | `test-vectors/v0` |

## Verification Commands

Run the full local gate before and after review changes:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p rava -- demo flight-booking
cargo package --workspace
cargo check --manifest-path fuzz/Cargo.toml
cargo check -p rava-wasm --target wasm32-unknown-unknown
npm --prefix packages/rava-wasm-js test
(cd packages/rava-wasm-js && npm pack --dry-run)
```

Optional fixture drift check:

```bash
cargo run -p rava -- demo flight-booking --write-fixtures examples/flight-booking --deterministic-fixtures
cp examples/flight-booking/*.json test-vectors/v0/flight-booking/
git diff -- examples test-vectors
```

The diff should be empty if the committed corpus is current.

Optional fuzzing entry point:

```bash
cargo fuzz run v0_wire_entrypoints
```

The fuzz target exercises JSON parsing, canonicalization, action verification, receipt verification, and attestation verification entry points. Long-running fuzz campaigns should record corpus, duration, and any minimized crash input in review notes.

## Review Output

A useful review should classify findings as:

- protocol correctness issue;
- implementation bug;
- documentation ambiguity;
- test coverage gap;
- V0 non-goal or future production requirement.

Findings that would weaken fail-closed verification should block release until fixed or explicitly moved out of scope with clear non-production language.

Record external findings, remediation, accepted risk, and verification evidence in [review-register-v0.md](review-register-v0.md).
