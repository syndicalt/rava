# Rava V0 Draft Release Notes Template

Use this template for Rava V0 draft release notes. Rava V0 is not production-ready security software.

## Status

- Release: `v0.x.y`
- Commit: `<git-sha>`
- Date: `<yyyy-mm-dd>`
- Posture: draft reference implementation / developer preview.
- External review: No external security review has been completed.

## Summary

Describe the protocol, implementation, documentation, and compatibility changes in this release.

## Verification

Record the exact gate output or CI run used for the release:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p rava -- demo flight-booking`
- `cargo package --workspace`
- `cargo check --manifest-path fuzz/Cargo.toml`
- `cargo check -p rava-wasm --target wasm32-unknown-unknown`
- `npm --prefix packages/rava-wasm-js test`

Record whether deterministic fixtures and `test-vectors/v0` were regenerated and whether the diff was expected.

## Compatibility

State whether this release changes:

- protocol version;
- signed wire fields;
- test vectors;
- schemas;
- rejection codes;
- CLI command names or arguments;
- preview service request, response, health, audit, or error shapes;
- wrapper API behavior.

If any compatibility-impacting change exists, link to the relevant migration note and tests.

## Roadmap Status

Link to `docs/roadmap.md` and summarize any roadmap evidence maps that changed in this release.

List remaining external blockers, including external security review, publishing or tagging decisions, and production systems that are still documented requirements rather than implemented guarantees.

## Known Non-Goals and External Requirements

Repeat any important non-goals or external requirements:

- no key custody guarantee;
- no public-key discovery guarantee;
- no distributed replay guarantee;
- no distributed revocation freshness guarantee;
- no managed audit-storage guarantee;
- no production caller identity or distributed rate-limit guarantee;
- no production monitoring guarantee;
- no external audit coverage unless a completed review is linked.

## Review Register

Link to `docs/security/review-register-v0.md`.

Summarize any findings that are new, accepted-risk, remediated, or verified for this release.

## Artifacts

List any published crates, packages, tags, checksums, or release assets.
