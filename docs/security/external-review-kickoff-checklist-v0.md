# Rava V0 External Review Kickoff Checklist

This checklist is the operational runbook for starting an external security review of the Rava V0 draft reference implementation. It is not evidence that Rava has been externally reviewed, certified, or approved for production use.

Use this file to keep review setup, finding intake, remediation tracking, and optional longer fuzz campaigns on one path. Do not use it to broaden V0 guarantees or imply production readiness.

## Before Contacting Reviewers

- Record the frozen commit SHA or signed tag in `docs/security/review-register-v0.md`, the release notes draft, and reviewer correspondence.
- Confirm the review target is a clean commit on `master`.
- Run the full local gate:

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

- Confirm `docs/security/external-review-cover-note-v0.md` names the frozen target, scope, non-production boundary, and expected verification commands.
- Confirm `docs/security/external-review-packet-v0.md` lists the packet artifacts from the frozen target.
- Use `docs/security/external-review-selection-v0.md` before scheduling review work so reviewer fit, conflicts, report constraints, and non-claim boundaries are recorded.
- Use `docs/security/external-review-outreach-template-v0.md` for reviewer-facing first-contact wording.
- Use `docs/security/external-review-request-v0.md` when contacting reviewers so the engagement request names expertise, deliverables, finding intake, out-of-scope production requirements, and the non-claim boundary.
- Track reviewer outreach in `docs/security/external-review-outreach-v0.md` so contact state, scope alignment, report constraints, and next actions are recorded without creating an audit claim.
- Confirm `docs/security/review-register-v0.md` still says no external review has been completed unless real findings have been recorded.

## Reviewer Handoff

Ask reviewers to prioritize these V0 security boundaries:

- canonicalization;
- signature binding;
- delegation attenuation;
- replay semantics;
- revocation semantics;
- receipt and attestation verification;
- fail-closed behavior for malformed, tampered, expired, revoked, replayed, or over-scoped inputs;
- documentation ambiguity that could cause unsafe integrations.

State that production key custody, public-key discovery, DID resolution, distributed replay, distributed revocation, caller identity, distributed rate limiting, managed audit storage, hosted verifier operations, and production monitoring are not implemented V0 guarantees.

## Finding Intake

- Ask reviewers to file findings with `.github/ISSUE_TEMPLATE/security-review-finding.yml` when GitHub issue tracking is appropriate.
- Apply the `security-review` label to each external review finding.
- Assign findings to the `V0 external security review` milestone.
- Mirror each finding into `docs/security/review-register-v0.md` with an ID such as `RAVA-REVIEW-001`.
- Use `docs/security/review-findings/template-v0.md` for findings that need more detail than the register table can safely hold.

## Remediation Tracking

Use these states exactly:

- `reported`: reviewer or maintainer reported the finding and it has not been triaged.
- `accepted`: maintainers agree the finding describes a real issue or ambiguity.
- `accepted-risk`: maintainers intentionally accept the residual risk for a draft release with explicit non-production language.
- `remediated`: a code, test, or documentation change has landed to address the finding.
- `verified`: the reviewer or maintainer verified the remediation with concrete evidence.
- `out-of-scope`: the finding is outside the stated V0 scope and is tracked as production or future work.

For accepted findings, remediation should be a small pull request with concrete verification evidence. Code findings should include rejection tests when the behavior can be exercised locally. Documentation ambiguity should be fixed with the same discipline as implementation findings when it could lead an integrator to rely on a non-guarantee.

No finding that weakens fail-closed verification may remain unresolved for any release represented as production-ready. For a V0 draft release, accepted risk must be explicit, narrow, and paired with non-production language.

## Optional Longer Fuzz Campaigns

Longer fuzz campaigns are useful review evidence when they are reproducible and tied to the target under review.

- Start from `docs/security/fuzz-campaigns/template-v0.md`.
- Record the exact command, for example `cargo fuzz run v0_wire_entrypoints -- -max_total_time=86400`.
- Record the target commit, duration, host, cargo-fuzz version, seed if known, corpus path, artifacts path, crash count, timeout count, OOM count, sanitizer output, and final rerun evidence.
- Preserve minimized crash inputs outside the committed repository if they contain sensitive data; otherwise add a small regression fixture or test when appropriate.
- Link resulting campaign logs from the external review packet or review register only as supplemental evidence.

Fuzzing does not prove security, does not replace external review, and does not create a production-ready or externally audited claim.

## Closeout

- Update `docs/security/review-register-v0.md` with every external finding, state transition, remediation PR, and verification note.
- Keep GitHub issues, milestone state, and register entries consistent.
- Confirm unresolved findings are either out of scope for V0 or accepted risks with explicit non-production language.
- Use `docs/security/external-review-closeout-template-v0.md` to record reviewed target, scope, findings, remediation evidence, residual risk, fuzz evidence, and release claim boundaries after a real review completes.
- Confirm there is `No production-ready or externally audited claim` in the review closeout.
- Confirm no public docs contain a `Rava is production-ready` claim, a `Rava has been externally audited` claim, or any equivalent production-ready or externally audited claim unless a real review and production readiness audit have happened.
- Record final review evidence in release notes before any release announcement.
