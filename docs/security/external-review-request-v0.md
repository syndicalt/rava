# Rava V0 External Review Request

This request scopes an external security review engagement for the Rava V0 draft reference implementation. It is not evidence that Rava has been externally reviewed, certified, or approved for production use.

Rava V0 is not production-ready security software. The purpose of this request is to obtain independent review findings and remediation guidance without broadening V0 guarantees.

Reviewer outreach status should be tracked in `docs/security/external-review-outreach-v0.md`.
Reviewer fit, conflicts, and report constraints should be evaluated with `docs/security/external-review-selection-v0.md` before scheduling review work.

## Engagement Goal

Review whether the Rava V0 draft reference implementation fails closed for attacker-controlled protocol inputs and whether the documentation clearly separates implemented guarantees from caller assumptions, production requirements, and non-goals.

The review should produce actionable findings that can be tracked in the repository, remediated in small pull requests, and verified with concrete evidence.

## Review Target

Reviewers should evaluate a frozen commit SHA or signed tag only.

Before the engagement starts, maintainers should record the target in:

- `docs/security/review-register-v0.md`;
- the release notes draft;
- reviewer correspondence;
- the external review issue or milestone.

Do not change the target during review unless the reviewer explicitly agrees to evaluate the new commit or tag. If the target changes, rerun the full local gate and record the new evidence.

## Required Reviewer Expertise

The review should be assigned to reviewers comfortable with:

- Rust security review;
- canonicalization of signed payloads;
- signature binding between signer IDs, public keys, payload fields, and derived IDs;
- Ed25519 verification usage;
- delegation attenuation;
- replay semantics;
- revocation semantics;
- receipt and attestation verification;
- fail-closed parser and verifier behavior;
- security documentation review for unsafe integration ambiguity.

## Requested Deliverables

Ask reviewers to return a finding list. Each finding should include:

- severity;
- affected boundary;
- reproduction or exploit story;
- affected files, modules, or documents;
- remediation recommendation;
- verification note describing how the remediation should be confirmed.

For findings that cannot be reproduced locally, reviewers should explain the reasoning, assumptions, and deployment conditions that would make the issue relevant.

## Finding Intake

When GitHub issue tracking is appropriate, use `.github/ISSUE_TEMPLATE/security-review-finding.yml`.

Mirror each finding into `docs/security/review-register-v0.md` with an ID such as `RAVA-REVIEW-001`.

Use `docs/security/review-findings/template-v0.md` when a finding needs more detail than the register table can safely hold.

Accepted findings should include a remediation pull request or accepted-risk rationale. Do not mark findings `verified` until concrete evidence is recorded.

After the review and remediation pass are complete, use `docs/security/external-review-closeout-template-v0.md` to summarize reviewed target, scope, findings, remediation evidence, residual risk, fuzz evidence, and release claim boundaries.

## Out of Scope

These are production requirements, not implemented V0 guarantees:

- production key custody;
- public-key discovery and DID resolution;
- distributed replay;
- distributed revocation;
- caller identity;
- distributed rate limiting;
- managed audit storage;
- hosted verifier operations;
- production monitoring;
- model-behavior proofs, reputation systems, or blockchain anchoring.

Out-of-scope findings should still be recorded when they affect deployment safety, but they should not be described as implemented V0 guarantees.

## Fuzzing Option

If reviewers request longer fuzzing, use `docs/security/fuzz-campaigns/template-v0.md` and record the exact target, command, duration, seed if known, corpus path, artifacts path, crash count, minimized inputs, remediation, and final rerun evidence.

Fuzzing is supplemental evidence. It does not prove security and does not replace external review.

## Non-Claim Boundary

No production-ready or externally audited claim may be made from sending this request, receiving reviewer interest, running fuzz campaigns, or opening findings.

Only a completed external review with recorded findings, remediation decisions, and verification evidence can support a statement about what was reviewed. Even then, production readiness remains separate from V0 draft review unless the production trust requirements are also satisfied.
