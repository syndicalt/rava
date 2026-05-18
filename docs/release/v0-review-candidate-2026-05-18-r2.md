# Rava V0 Review Candidate Notes: 2026-05-18 R2

These notes identify the second frozen Rava V0 draft reference implementation target prepared for external security review. This is not a production release, not a crate publication decision, and not evidence that an external security review has been completed.

R2 supersedes `docs/release/v0-review-candidate-2026-05-18.md` for new reviewer handoffs because it includes the later review-readiness documentation, dependency guardrails, closeout template, and locked fuzz compile gate. The original candidate note remains historical evidence for the earlier frozen target.

## Review Target

- Tag: `v0-review-candidate-2026-05-18-r2`
- Commit: `d611c6d1c2fd00d7a3d46a4031bdea65820fe78b`
- Tracking issue: External security review: V0 review candidate
- Tracking issue URL: https://github.com/syndicalt/rava/issues/87

The tag is an immutable review target. If the target changes, create a new candidate note and rerun the full verification baseline.

## Reviewer Packet

Reviewers should start with:

- `docs/security/external-review-packet-v0.md`;
- `docs/security/external-review-kickoff-checklist-v0.md`;
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
- `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints-1800s.md`.

## Verification Baseline

The review target passed the full local gate through master CI run `26017062950`:

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

Pages run `26017062966` also passed for the same commit.

## Included Review-Readiness Updates

R2 includes the original candidate materials plus:

- security-sensitive code ownership;
- security pull request checklist;
- dependency update guardrails;
- external review kickoff checklist;
- reviewer engagement request;
- external review closeout template;
- structured security review finding issue template;
- locked fuzz compile gate with `cargo check --manifest-path fuzz/Cargo.toml --locked`;
- GitHub Pages, Rust dependency, and TypeScript wrapper maintenance updates.

## Fuzz Evidence

The packet includes the bounded `v0_wire_entrypoints` fuzz campaign recorded in `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints.md`.

The packet also links the supplemental 1800-second campaign recorded in `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints-1800s.md`.

Those campaigns completed without crashes in bounded runs. They are useful review evidence, not proof of security and not replacements for external review.

## Status Boundaries

- Rava V0 is a draft reference implementation.
- No external security review has been completed.
- Production key custody, public-key discovery, distributed replay, distributed revocation, caller identity, distributed rate limiting, managed audit storage, hosted verifier operations, and monitoring remain external production requirements.
- This review candidate must not be described as production-ready security software.

## Finding Handling

Record findings in `docs/security/review-register-v0.md`. Use `docs/security/review-findings/template-v0.md` for detailed finding records that need target, classification, impact, remediation, regression, and verification evidence.
