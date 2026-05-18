# Rava V0 Security Review Register

This register tracks external security review findings and remediation for the Rava V0 draft.

No external security review has been completed yet. This file is a tracking artifact, not evidence that Rava has been externally audited or certified.

The current review-candidate notes are `docs/release/v0-review-candidate-2026-05-18-r2.md`.

The review execution plan is [review-plan-v0.md](review-plan-v0.md), whose repo-relative path is `docs/security/review-plan-v0.md`. Review kickoff should use [external-review-kickoff-checklist-v0.md](external-review-kickoff-checklist-v0.md), whose repo-relative path is `docs/security/external-review-kickoff-checklist-v0.md`. Reviewer engagement requests should use [external-review-request-v0.md](external-review-request-v0.md), whose repo-relative path is `docs/security/external-review-request-v0.md`. Review closeout should use [external-review-closeout-template-v0.md](external-review-closeout-template-v0.md), whose repo-relative path is `docs/security/external-review-closeout-template-v0.md`. Individual findings should use [review-findings/template-v0.md](review-findings/template-v0.md), whose repo-relative path is `docs/security/review-findings/template-v0.md`. GitHub issue tracking should use the repository issue form `.github/ISSUE_TEMPLATE/security-review-finding.yml`. Optional longer fuzz campaigns should be recorded using [fuzz-campaigns/template-v0.md](fuzz-campaigns/template-v0.md), whose repo-relative path is `docs/security/fuzz-campaigns/template-v0.md`.

## Current Review Target

- Tag: `v0-review-candidate-2026-05-18-r2`
- Commit: `d611c6d1c2fd00d7a3d46a4031bdea65820fe78b`
- Candidate notes: `docs/release/v0-review-candidate-2026-05-18-r2.md`
- Tracking issue: https://github.com/syndicalt/rava/issues/87

This target is frozen for reviewer handoff. It is not evidence that Rava has been externally reviewed, certified, or approved for production use.

## Scope

Findings should reference the review scope in [review-guide-v0.md](review-guide-v0.md), including:

- Rust verifier semantics;
- canonicalization and signature binding;
- nonce validation;
- replay and revocation semantics;
- receipt and attestation verification;
- wrapper and adapter fail-closed behavior;
- documentation gaps that could lead to unsafe deployments.

Production deployment findings about key custody, key discovery, distributed replay, distributed revocation, caller identity, distributed rate limiting, audit storage, and monitoring should also reference [../operations/production-trust-v0.md](../operations/production-trust-v0.md) and the relevant detailed runbook:

- [key-custody-v0.md](../operations/key-custody-v0.md);
- [key-discovery-v0.md](../operations/key-discovery-v0.md);
- [distributed-replay-v0.md](../operations/distributed-replay-v0.md);
- [distributed-revocation-v0.md](../operations/distributed-revocation-v0.md);
- [audit-storage-v0.md](../operations/audit-storage-v0.md);
- [caller-identity-v0.md](../operations/caller-identity-v0.md);
- [distributed-rate-limits-v0.md](../operations/distributed-rate-limits-v0.md);
- [monitoring-v0.md](../operations/monitoring-v0.md).

## Finding States

Use one of these states for each recorded finding:

- `reported`: reviewer or maintainer reported the finding and it has not been triaged.
- `accepted`: maintainers agree the finding describes a real issue or ambiguity.
- `accepted-risk`: maintainers intentionally accept the residual risk for a draft release with explicit non-production language.
- `remediated`: a code, test, or documentation change has landed to address the finding.
- `verified`: the reviewer or maintainer verified the remediation with concrete evidence.
- `out-of-scope`: the finding is outside the stated V0 scope and is tracked as production or future work.

Findings that could weaken fail-closed verification should not remain in `accepted-risk` for a release represented as production-ready.

## Register

No external findings are recorded yet.

When a review starts, record findings in this table:

| ID | Source | Area | State | Summary | Remediation | Verification |
| --- | --- | --- | --- | --- | --- | --- |
| _none_ | _none_ | _none_ | _none_ | No external findings are recorded yet. | _none_ | _none_ |
