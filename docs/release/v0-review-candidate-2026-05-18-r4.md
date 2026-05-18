# Rava V0 Review Candidate Notes: 2026-05-18 R4

These notes identify the fourth frozen Rava V0 draft reference implementation target prepared for external security review. This is not a production release, not a crate publication decision, and not evidence that an external security review has been completed.

R4 supersedes `docs/release/v0-review-candidate-2026-05-18-r3.md` for new reviewer handoffs because it includes the local preview service guardrail evidence merged after R3. Earlier candidate notes remain historical evidence for earlier frozen targets.

## Review Target

- Tag: `v0-review-candidate-2026-05-18-r4`
- Commit: `b1b65fb6263b4f9143bdac8a5b46fbce6fdc532d`
- Tracking issue: External security review: V0 review candidate
- Tracking issue URL: https://github.com/syndicalt/rava/issues/87

The tag is an immutable review target. If the target changes, create a new candidate note and rerun the full verification baseline.

This note is a post-target control-plane artifact. It is not part of the frozen target tree because it records the tag and commit after the target was selected and verified. The frozen target is the tag and commit above. Reviewers should use this note to locate the target, then evaluate the contents of `v0-review-candidate-2026-05-18-r4` unless they explicitly agree to evaluate later commits.

## Reviewer Packet

Reviewers should start with:

- `docs/security/external-review-packet-v0.md`;
- `docs/security/external-review-kickoff-checklist-v0.md`;
- `docs/security/external-review-selection-v0.md`;
- `docs/security/external-review-request-v0.md`;
- `docs/security/external-review-closeout-template-v0.md`;
- `docs/security/review-plan-v0.md`;
- `docs/security/review-guide-v0.md`;
- `docs/security/review-register-v0.md`;
- `docs/security/review-findings/template-v0.md`;
- `docs/security/threat-model-v0.md`;
- `docs/protocol/rava-v0.md`;
- `docs/release/v0-draft-checklist.md`;
- `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints.md`;
- `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints-1800s.md`;
- `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints-3600s.md`.

## Verification Baseline

The review target passed the full local gate through master CI run `26045290663`:

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

Pages run `26045288832` also passed for the same commit.

## Included Review-Readiness Updates

R4 includes the R3 candidate materials plus local preview service hardening merged after R3:

- fail-closed `serve verify` startup checks for fresh revocation configuration;
- explicit startup guardrails for required replay store configuration with `--require-replay-store`;
- explicit startup guardrails for required audit log configuration with `--require-audit-log`;
- explicit startup guardrails for required bearer-token configuration with `--require-auth-token-env`;
- explicit startup guardrails for required local rate limits with `--require-rate-limit-per-minute`;
- explicit startup guardrails for required metrics exposure with `--require-metrics`;
- issue-tracker notes preserving the boundary between local preview guardrails and production guarantees.

No external reviewer outreach has been sent for R4.

## Fuzz Evidence

The packet includes the bounded `v0_wire_entrypoints` fuzz campaign recorded in `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints.md`.

The packet also links the supplemental 1800-second campaign recorded in `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints-1800s.md`.

The packet also links the 3600-second campaign recorded in `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints-3600s.md`.

Those campaigns completed without crashes in bounded runs. They are useful review evidence, not proof of security and not replacements for external review.

## Status Boundaries

- Rava V0 is a draft reference implementation.
- No external security review has been completed.
- Production key custody, public-key discovery, distributed replay, distributed revocation, caller identity, distributed rate limiting, managed audit storage, hosted verifier operations, and monitoring remain external production requirements.
- This review candidate must not be described as production-ready security software.

## Finding Handling

Record findings in `docs/security/review-register-v0.md`. Use `docs/security/review-findings/template-v0.md` for detailed finding records that need target, classification, impact, remediation, regression, and verification evidence.
