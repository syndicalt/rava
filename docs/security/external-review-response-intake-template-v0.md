# Rava V0 External Review Response Intake Template

This template records a response from a potential external reviewer for the Rava V0 draft reference implementation. It is not evidence that Rava has been externally reviewed, certified, or approved for production use.

Rava V0 is not production-ready security software. Reviewer responses, scoping calls, quotes, NDAs, scheduling notes, and report constraints do not create an audit claim.

## Use Rules

Copy this template into the private tracking location or issue where reviewer correspondence is being managed. Do not fill this template in preemptively.

Do not record private keys, credentials, access tokens, raw sensitive action payloads, or private commercial terms. Public docs should record process state, scope decisions, and evidence boundaries, not confidential correspondence.

Use `docs/security/external-review-outreach-v0.md` for the public outreach state. Use `docs/security/review-register-v0.md` only for actual findings, remediation states, and verification evidence.

## Response Metadata

- Reviewer or firm:
- Response date:
- Current state: `candidate`, `declined`, `scoping`, `scheduled`, `in-review`, `complete`, or `cancelled`
- Related outreach row:
- Related issue, milestone, or private tracker:
- Maintainer owner:

## Scope Decision

- Frozen target accepted: yes or no
- Frozen target: `v0-review-candidate-2026-05-18-r3`
- Frozen target commit: `416d8a9661e75bd66dd60bed72d9485d833f36e2`
- Review packet: `docs/security/external-review-packet-v0.md`
- Reviewer can evaluate the frozen target without a target change: yes or no
- Target change requested: yes or no
- If the target must change, new target and required verification baseline:

Do not move the frozen target during review unless the reviewer explicitly agrees to evaluate the new commit or tag and the full local gate is rerun.

## Constraints to Record

- Scope exclusions:
- Report disclosure constraints:
- Attribution constraints:
- Embargo or coordinated disclosure constraints:
- Finding intake path:
- Expected finding list format:
- Expected remediation verification path:
- Longer fuzz campaign requested: yes or no

The fields above preserve searchable terms for report disclosure, attribution, embargo, and finding intake decisions without exposing confidential correspondence.

If GitHub issue tracking is appropriate, findings should use `.github/ISSUE_TEMPLATE/security-review-finding.yml`. If a finding needs more detail than the register table can hold, use `docs/security/review-findings/template-v0.md`.

## Next Action

Choose one next action:

- update `docs/security/external-review-outreach-v0.md` with the reviewer state and next action;
- send the full `docs/security/external-review-packet-v0.md`;
- schedule the review window for the accepted frozen target;
- ask for scope clarification before scheduling work;
- record declined, cancelled, or unavailable state without creating a review claim;
- mirror delivered findings into `docs/security/review-register-v0.md`;
- open structured finding issues with `.github/ISSUE_TEMPLATE/security-review-finding.yml`;
- use `docs/security/external-review-closeout-template-v0.md` only after a real review has a reviewer identity or report reference, finding list, remediation decisions, and verification evidence.

## Non-Claim Boundary

No production-ready or externally audited claim may be made from a reviewer response, scoping call, quote, NDA, schedule hold, or private expression of interest.

Permitted wording before completed review evidence is limited to process status, for example:

- "Reviewer response intake is tracked for the Rava V0 draft target."
- "The reviewer is scoping the frozen V0 draft target."

Forbidden wording includes:

- "Rava is externally audited."
- "Rava is production-ready."
- "A reviewer approved Rava" before completed findings, remediation decisions, and verification evidence exist.
