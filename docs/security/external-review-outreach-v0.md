# Rava V0 External Review Outreach Tracker

This tracker records reviewer outreach for the Rava V0 draft reference implementation. It is not evidence that Rava has been externally reviewed, certified, or approved for production use.

Rava V0 is not production-ready security software. Outreach, quotes, availability checks, and scheduling discussions do not create an external audit claim.

## Current Review Target

- Tag: `v0-review-candidate-2026-05-18-r4`
- Commit: `b1b65fb6263b4f9143bdac8a5b46fbce6fdc532d`
- Candidate notes: `docs/release/v0-review-candidate-2026-05-18-r4.md`
- Review packet: `docs/security/external-review-packet-v0.md`
- Request artifact: `docs/security/external-review-request-v0.md`
- Selection rubric: `docs/security/external-review-selection-v0.md`
- Tracking issue: https://github.com/syndicalt/rava/issues/87

Do not change the review target during outreach unless the reviewer explicitly agrees to evaluate the new commit or tag and the full verification baseline is rerun.

## Outreach States

Use one of these states for each reviewer or firm:

- `candidate`: reviewer is being considered and has not been contacted.
- `contacted`: reviewer has been contacted, with no scope or schedule response yet.
- `declined`: reviewer declined, was unavailable, or was not a fit for the Rava V0 scope.
- `scoping`: reviewer is discussing scope, availability, report handling, or engagement terms.
- `scheduled`: reviewer accepted the scope and a review window is planned.
- `in-review`: reviewer has the frozen target and review is underway.
- `complete`: reviewer delivered findings or a report for the frozen target.
- `cancelled`: outreach or engagement ended before review completion.

Only `complete` with findings, remediation decisions, and verification evidence can feed a closeout record. It still does not imply production readiness.

## Outreach Table

| Reviewer or firm | State | Contact date | Scope alignment | Report constraints | Next action |
| --- | --- | --- | --- | --- | --- |
| _none_ | candidate | _none_ | No reviewer outreach has been recorded yet. | _none_ | _none_ |

## Intake Rules

- Use `docs/security/external-review-request-v0.md` for outreach so scope, deliverables, non-goals, and non-claim boundaries stay consistent.
- Use `docs/security/external-review-outreach-template-v0.md` for first-contact wording so the frozen target, requested deliverables, production non-goals, and non-claim boundary stay consistent.
- Use `docs/security/external-review-response-intake-template-v0.md` when a reviewer responds so target acceptance, scope decisions, report constraints, and next actions are recorded without implying review completion.
- Use `docs/security/external-review-selection-v0.md` before scheduling work so reviewer fit, conflicts of interest, report disclosure constraints, and attribution constraints are recorded.
- Confirm reviewers can evaluate the frozen target in `docs/security/external-review-packet-v0.md`.
- Record constraints that affect public reporting, disclosure, remediation evidence, or attribution before the review starts.
- Mirror findings into `docs/security/review-register-v0.md` with IDs such as `RAVA-REVIEW-001`.
- Use `docs/security/review-findings/template-v0.md` for findings that need more detail than the register table can safely hold.
- Use `docs/security/external-review-closeout-template-v0.md` only after a real review has a reviewer identity or report reference, finding list, remediation decisions, and verification evidence.

## Non-Claim Boundary

No production-ready or externally audited claim may be made from candidate selection, outreach, scheduling, price quotes, NDAs, preliminary calls, or reviewer interest.

Permitted wording before review completion is limited to process status, for example:

- "Rava is seeking external review for the R4 V0 draft target."
- "Reviewer outreach is tracked in `docs/security/external-review-outreach-v0.md`."

Forbidden wording includes broad claims such as:

- "Rava is externally audited."
- "Rava is production-ready."
- "Reviewers approved Rava" before completed findings, remediation decisions, and verification evidence exist.
