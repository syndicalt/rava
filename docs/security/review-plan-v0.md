# Rava V0 External Security Review Plan

This plan describes how to run an external security review for the Rava V0 draft reference implementation. It is not evidence that Rava has been externally audited, certified, or approved for production use.

## Review Target

Record the exact commit SHA under review in the review register and release notes draft before sending material to reviewers. Do not change the review target during review unless the reviewer explicitly agrees to review the new commit.

The target should be a clean commit on `master` with the full local gate passing.

Security-sensitive paths are mapped in `.github/CODEOWNERS`; changes to protocol core, verifier service, wrappers, review docs, release docs, workflows, examples, and test vectors should preserve explicit review ownership.

Pull requests should use `.github/pull_request_template.md` to record boundary impact, verification evidence, and affected review artifacts before merge.

Dependency updates for Cargo, the TypeScript wrapper, and GitHub Actions are tracked in `.github/dependabot.yml` and should still pass the full local gate before merge.

## Reviewer Packet

The concrete handoff manifest is `docs/security/external-review-packet-v0.md`.

Use `docs/security/external-review-cover-note-v0.md` as the reviewer-facing cover note for the frozen target, scope, non-production boundary, and expected verification commands.

Use `docs/security/external-review-kickoff-checklist-v0.md` to execute the review kickoff, issue intake, remediation tracking, and optional longer fuzz campaign workflow.

Send reviewers these repository artifacts:

- `README.md`;
- `docs/security/threat-model-v0.md`;
- `docs/protocol/rava-v0.md`;
- `docs/protocol/time-semantics-v0.md`;
- `docs/operators/rejection-codes-v0.md`;
- `docs/security/v0-draft-completion-audit.md`;
- `docs/security/review-guide-v0.md`;
- `docs/security/review-register-v0.md`;
- `docs/release/v0-draft-checklist.md`;
- `docs/operations/production-trust-v0.md`;
- `test-vectors/v0`;
- `examples/flight-booking`.

The packet should state that Rava V0 is a draft reference implementation, not production-ready security software.

## In Scope

External review should focus on:

- canonicalization of signed payloads;
- signature binding between signer IDs, public keys, payload fields, and derived IDs;
- Ed25519 verification usage;
- nonce validation;
- delegation attenuation;
- replay semantics and replay persistence failure behavior;
- revocation semantics;
- receipt and attestation verification;
- fail-closed behavior for malformed, tampered, expired, revoked, replayed, or over-scoped inputs;
- documentation ambiguity that could cause unsafe integrations.

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

Out-of-scope findings should still be recorded if they affect deployment safety, but they should be classified as `out-of-scope` or production work rather than implied V0 guarantees.

## Finding Intake

Record each finding in `docs/security/review-register-v0.md` with an ID such as `RAVA-REVIEW-001`.

Each finding should include:

- source reviewer or report reference;
- affected area;
- state: `reported`, `accepted`, `remediated`, `verified`, `accepted-risk`, or `out-of-scope`;
- summary;
- remediation PR or accepted-risk rationale;
- concrete verification evidence.

## Release Rule

No finding that weakens fail-closed verification may remain unresolved for a release represented as production-ready. For a V0 draft release, accepted risk must be explicit, narrow, and paired with non-production language.

Documentation-only ambiguity should be fixed with the same discipline as code findings when it could lead an integrator to rely on a non-guarantee.

## Fuzz Evidence

If longer fuzzing is run before or during review, record the campaign using `docs/security/fuzz-campaigns/template-v0.md`.

Fuzzing is useful review evidence, but it does not prove security and does not replace external review.
