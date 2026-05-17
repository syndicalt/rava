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
- Full local gate passes: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo run -p rava -- demo flight-booking`.

## V0 Hardening

Goal: strengthen the draft core without adding speculative product surface.

Candidate work:

- Add property tests for canonicalization, attenuation monotonicity, replay consumption, and malformed signed objects.
- Keep fuzz targets current for JSON parsing, canonicalization, and signature verification entry points.
- Keep deterministic fixture regeneration checks current so committed examples cannot drift silently.
- Keep rejection-code documentation current for operators and wrapper authors.
- Use the V0 review guide for independent review of canonicalization, signature binding, nonce validation, replay semantics, and revocation semantics.
- Keep clock-skew and expiry expectations current for verifier callers.

Hardening work must not introduce alternate cryptographic primitives or verifier shortcuts.

## V1 Developer Preview

Goal: provide a stable developer-facing package while keeping Rust core semantics authoritative.

Candidate work:

- Stabilize CLI command names, JSON request and response shapes, and rejection-code subjects.
- Continue hardening the verifier service boundary beyond current request-size limits, health checks, local file-backed replay/revocation stores, and local audit output: authenticated ingress assumptions and distributed replay/revocation backends.
- Publish versioned crates and artifacts only after release gates and review notes are current.
- Add migration notes for any wire or CLI change from V0 draft to V1 preview.
- Keep compatibility policy current for test vectors and schemas.

V1 developer preview still should not claim distributed trust, global identity, managed custody, or external audit coverage unless those workstreams are complete.

## Interop

Goal: enable wrappers and adapters around the Rust verifier without reimplementing trusted verification logic.

Sequencing:

- Keep Rust core and CLI test vectors stable.
- Add WASM bindings around the Rust verifier.
- Add a TypeScript package that calls WASM and runs the V0 test vectors.
- Add DID/key-resolution examples as caller policy, not core trust.
- Add MCP adapter proof of concept.
- Add OAuth exchange examples only after the verifier wrapper is stable.

Detailed wrapper guidance lives in [interop/roadmap-v0.md](interop/roadmap-v0.md).

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

## Non-Goals

The core protocol roadmap does not include:

- inventing new cryptographic primitives;
- replacing OAuth, DID methods, wallets, or custody providers;
- blockchain anchoring as a core security requirement;
- reputation scoring or model-behavior proofs;
- broad product features that bypass signed action verification.

If a future feature weakens fail-closed verification, it does not belong in the core protocol.
