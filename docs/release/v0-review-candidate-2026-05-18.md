# Rava V0 Review Candidate Notes: 2026-05-18

These notes identify the frozen Rava V0 draft reference implementation target prepared for external security review. This is not a production release, not a crate publication decision, and not evidence that an external security review has been completed.

## Review Target

- Tag: `v0-review-candidate-2026-05-18`
- Commit: `0672e61fcf46b472aee4e32d1915a0c975a0bbda`
- Tracking issue: External security review: V0 review candidate
- Tracking issue URL: https://github.com/syndicalt/rava/issues/87

The tag is an immutable review target. If the target changes, create a new candidate note and rerun the full verification baseline.

## Reviewer Packet

Reviewers should start with:

- `docs/security/external-review-packet-v0.md`;
- `docs/security/review-plan-v0.md`;
- `docs/security/review-guide-v0.md`;
- `docs/security/review-register-v0.md`;
- `docs/security/review-findings/template-v0.md`;
- `docs/security/threat-model-v0.md`;
- `docs/protocol/rava-v0.md`;
- `docs/release/v0-draft-checklist.md`;
- `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints.md`.

## Verification Baseline

The review target passed the full local gate through master CI run `26011396149`:

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

Pages run `26011396147` also passed for the same commit.

## Fuzz Evidence

The packet includes the bounded `v0_wire_entrypoints` fuzz campaign recorded in `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints.md`.

That campaign completed without crashes in a bounded run. It is useful review evidence, not a proof of security and not a replacement for external review.

## Status Boundaries

- Rava V0 is a draft reference implementation.
- No external security review has been completed.
- Production key custody, public-key discovery, distributed replay, distributed revocation, caller identity, distributed rate limiting, managed audit storage, and monitoring remain external production requirements.
- This review candidate must not be described as production-ready security software.

## Finding Handling

Record findings in `docs/security/review-register-v0.md`. Use `docs/security/review-findings/template-v0.md` for detailed finding records that need target, classification, impact, remediation, regression, and verification evidence.
