# Rava V0 Release Audit Notes

This is an internal publication-readiness audit for the V0 draft repository. It is not a formal external cryptographic audit, penetration test, or production readiness certification.

## Scope Reviewed

- README status, setup path, verification commands, examples, and warning language.
- V0 protocol documentation.
- V0 threat model.
- Interop roadmap.
- CLI regression coverage for examples, test vectors, wire schemas, receipts, attestations, replay, revocation, inspection, and verifier service preview.
- Workspace verification gates.

## Findings

### Draft posture needed to be more explicit

The repository had strong technical documentation, but the README did not clearly state that Rava V0 is draft software and not production-ready. The README now states this near the top and links to the threat model, audit notes, and roadmap.

### Top-level roadmap was missing

The repository had an interop roadmap, but not a functional roadmap for the core protocol project. [../roadmap.md](../roadmap.md) now separates current baseline, release readiness, V0 hardening, V1 developer preview, interop, and production operations work.

### HTTP verifier preview needed stronger warning language

The preview service is useful for local integration, but it does not implement production service responsibilities such as request authentication, replay consumption, distributed revocation freshness, rate limiting, or persistent audit storage. The README now says this directly.

### Local state should not be shipped as release content

The working tree currently contains `.eventloom/rava-default.jsonl` as modified local state. It should be reviewed before any commit or release packaging step and excluded if it is not intended project content.

## Current Positive Coverage

- The Rust verifier rejects malformed, unsigned, tampered, expired, revoked, replayed, and over-scoped inputs in focused regression tests.
- Test vectors and committed examples are checked by CLI integration tests.
- Wire schemas are documented as preflight shape checks only, not verifier substitutes.
- The threat model documents V0 assets, trusted computing base, attacker capabilities, assumptions, and non-goals.

## Remaining Risks

- No external security review has been completed.
- No fuzzing or property-test suite is currently part of the gate.
- DID/key resolution, distributed replay, distributed revocation freshness, key custody, and audit storage are caller or roadmap responsibilities.
- The preview HTTP verifier is not hardened as a production service boundary.

## Release Gate

A draft release candidate should pass:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p rava -- demo flight-booking
```

Passing this gate means the local reference implementation and regression suite are coherent. It does not mean the protocol is production-ready.
