# Rava V0 Security Review Finding Template

This template records one external review finding for the Rava V0 draft reference implementation. It is not evidence that Rava has been externally reviewed, certified, or approved for production use.

Copy this file to a dated finding file such as `docs/security/review-findings/RAVA-REVIEW-001.md` when a reviewer or maintainer reports a finding.

## Target

- Finding ID: `RAVA-REVIEW-001`
- Immutable commit SHA or signed tag under review:
- Reviewer or source report:
- Date reported:
- Affected area:

## Classification

Choose the primary classification:

- protocol correctness issue;
- implementation bug;
- documentation ambiguity;
- test coverage gap;
- V0 non-goal or future production requirement.

## Triage

- State: `reported`, `accepted`, `remediated`, `verified`, `accepted-risk`, or `out-of-scope`
- Maintainer owner:
- Reviewer contact:
- Release impact:
- Blocks draft release: yes or no
- Blocks production-ready claims: yes or no

## Impact

Describe what an attacker, confused integrator, or faulty operator could do if this finding is real.

Call out whether the finding affects fail-closed verification, canonicalization, signature binding, delegation attenuation, replay, revocation, receipt verification, attestation verification, verifier service boundaries, wrappers, or documentation safety.

## Evidence

- Reviewer evidence:
- Reproduction steps:
- Affected files or docs:
- Related tests:

## Remediation Plan

Describe the smallest production-quality remediation that addresses the root cause without adding speculative protocol surface.

- Code changes:
- Regression tests:
- Documentation changes:
- Compatibility impact:
- Accepted-risk rationale, if any:

## Required Regression Evidence

For accepted implementation or protocol findings, add rejection or fail-closed tests before changing code.

For documentation ambiguity, add or update publication-doc regression coverage when the wording affects security posture, verifier assumptions, non-goals, or release claims.

Regression tests should fail before the remediation and pass after it. Do not use test-only bypasses, fixture rewrites without explanation, or weaker verifier behavior to make the test pass.

## Verification Evidence

Record exact commands and results:

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

If fuzzing is relevant, record the campaign using `docs/security/fuzz-campaigns/template-v0.md`.

## Register Update

After triage or remediation, update `docs/security/review-register-v0.md` with:

- finding ID;
- source;
- area;
- state;
- summary;
- remediation PR, commit, or accepted-risk rationale;
- verification evidence.

Do not mark a finding `verified` until concrete evidence has been recorded.
