# Rava V0 External Review Outreach Template

This template provides a reviewer-facing outreach message for the Rava V0 draft reference implementation. It is not evidence that Rava has been externally reviewed, certified, or approved for production use.

Rava V0 is not production-ready security software. Sending this message, receiving reviewer interest, scheduling a call, or receiving a quote does not create an audit claim.

## Use Rules

- Do not send outreach until a maintainer explicitly authorizes real contact.
- Current allowed path is zero-budget OSS/security-community review only.
- Do not mark anyone `contacted` until a message is actually sent.
- Do not send this template until reviewer fit has been evaluated with `docs/security/external-review-selection-v0.md`.
- Record contact state, scope alignment, report constraints, and next action in `docs/security/external-review-outreach-v0.md`.
- Keep the frozen review target unchanged unless the reviewer explicitly agrees to evaluate a new commit or tag and the full local gate is rerun.
- Use `docs/security/external-review-request-v0.md` as the scope source for engagement terms, deliverables, finding intake, out-of-scope production requirements, and the non-claim boundary.
- Do not claim production readiness, external audit completion, reviewer approval, or certification.

## Message Template

Subject: Rava V0 draft external security review request

Hello _reviewer_,

I am seeking external security review for Rava V0, a draft Rust reference implementation for action-native agent authorization.

The current frozen review target is:

- Tag: `v0-review-candidate-2026-05-18-r4`
- Commit: `b1b65fb6263b4f9143bdac8a5b46fbce6fdc532d`
- Tracking issue: https://github.com/syndicalt/rava/issues/87

Please start with:

- `docs/security/external-review-packet-v0.md`
- `docs/security/external-review-request-v0.md`
- `docs/security/threat-model-v0.md`
- `docs/protocol/rava-v0.md`
- `docs/security/review-guide-v0.md`

The requested review focus is whether the V0 draft fails closed for attacker-controlled protocol inputs and whether the documentation clearly separates implemented guarantees from caller assumptions, production requirements, and non-goals.

Please return a finding list. Useful findings should include:

- severity;
- affected boundary;
- reproduction or exploit story;
- affected files, modules, or documents;
- remediation recommendation;
- verification note describing how the remediation should be confirmed.

Out of scope for V0 implementation guarantees: production key custody, public-key discovery, DID resolution, distributed replay, distributed revocation, caller identity, distributed rate limiting, managed audit storage, hosted verifier operations, production monitoring, model-behavior proofs, reputation, and blockchain anchoring.

No production-ready or externally audited claim will be made from this outreach, reviewer interest, scheduling, or quote activity. Any public statement after a completed review must name the reviewed target, scope, reviewer or report reference, findings, remediation decisions, verification evidence, and residual risk.

Please let me know whether this scope fits your review practice, what report disclosure constraints would apply, and what availability or engagement terms you would need.

## After Sending

- Set the reviewer state to `contacted` in `docs/security/external-review-outreach-v0.md`.
- If the reviewer discusses scope, availability, report handling, or engagement terms, move the state to `scoping`.
- Record conflicts of interest, report disclosure constraints, attribution constraints, NDA or embargo terms, and whether remediation verification can cite the reviewer or report reference.
- Do not mark any reviewer `scheduled`, `in-review`, or `complete` without concrete evidence for that state.
