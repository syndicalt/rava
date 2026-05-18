# Rava V0 External Review Closeout Template

Use this template after an external security review has completed. This template is not evidence that Rava has been externally reviewed, certified, or approved for production use.

Do not fill this file in preemptively. Copy it to a dated closeout file only when a real review has a frozen commit SHA or signed tag, reviewer identity or report reference, finding list, remediation decisions, and verification evidence.

## Reviewed Target

- Frozen commit SHA or signed tag:
- Review start date:
- Review end date:
- Reviewer or report reference:
- Related issue or milestone:

## Reviewer and Scope

Summarize what the reviewer evaluated.

Include whether the review covered:

- canonicalization;
- signature binding;
- delegation attenuation;
- replay semantics;
- revocation semantics;
- receipt and attestation verification;
- fail-closed behavior for malformed, tampered, expired, revoked, replayed, or over-scoped inputs;
- documentation ambiguity that could cause unsafe integrations.

Record anything explicitly excluded from the engagement.

## Finding Summary

Mirror the finding state in `docs/security/review-register-v0.md`.

| ID | State | Severity | Area | Summary | Tracking |
| --- | --- | --- | --- | --- | --- |
| RAVA-REVIEW-001 | reported | _severity_ | _area_ | _summary_ | _issue or PR_ |

Use these states exactly:

- `reported`;
- `accepted`;
- `remediated`;
- `verified`;
- `accepted-risk`;
- `out-of-scope`.

## Remediation Evidence

For each accepted finding, record:

- pull request or commit;
- regression tests;
- verification commands;
- reviewer verification or maintainer verification note;
- remaining follow-up issues, if any.

Do not mark a finding `verified` unless the evidence is concrete and reproducible.

## Residual Risk

Record every accepted-risk rationale.

Each accepted-risk rationale should include:

- affected finding ID;
- why the risk is acceptable for a V0 draft release;
- what production requirements remain before production-ready claims are allowed;
- what future issue, milestone, or external dependency owns the residual risk.

Production key custody, public-key discovery, distributed replay, distributed revocation, caller identity, distributed rate limiting, managed audit storage, hosted operations, and production monitoring remain production requirements unless explicitly implemented and reviewed.

## Fuzz Evidence

If fuzzing contributed to review closeout, link the campaign log and summarize the result.

Use `docs/security/fuzz-campaigns/template-v0.md` for campaign logs. Include command, duration, seed if known, corpus path, artifacts path, crash count, minimized inputs, remediation, and final rerun evidence.

Fuzzing is supplemental evidence. It does not prove security and does not replace external review.

## Release Claim Boundary

No production-ready or externally audited claim may be made unless the closeout evidence supports the exact claim.

Even after an external review, Rava V0 remains not production-ready security software unless the production trust requirements are separately satisfied.

Permitted wording should be narrow, for example:

- "Rava V0 was externally reviewed by _reviewer_ on _target_ for _scope_."
- "Findings and remediation evidence are recorded in _closeout file_ and `docs/security/review-register-v0.md`."

Forbidden wording includes broad claims such as:

- "Rava is production-ready."
- "Rava is externally audited" without naming target, scope, reviewer or report reference, and residual risk.
- "No security issues exist."
