# Rava V0 External Review Selection Rubric

This rubric records how to select external reviewers for the Rava V0 draft reference implementation. It is not evidence that Rava has been externally reviewed, certified, or approved for production use.

Rava V0 is not production-ready security software. Candidate evaluation, quotes, availability checks, scheduling, and reviewer interest do not create an external audit claim.

## Required Fit

Select reviewers or firms with practical experience in the boundaries Rava actually implements:

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

The reviewer does not need to cover production systems that V0 explicitly treats as caller or deployment responsibilities, but they should be able to identify when those requirements are being confused with implemented protocol guarantees.

## Disqualifying Gaps

Do not treat a reviewer as a fit for the V0 review if they:

- cannot review the frozen target in `docs/security/external-review-packet-v0.md`;
- cannot produce written findings with enough detail to reproduce, triage, remediate, or explicitly accept risk;
- requires production-ready or audited language before a completed review, finding list, remediation decisions, and verification evidence exist;
- requires changing the frozen target without an explicit reviewer agreement and a rerun of the full verification baseline;
- cannot distinguish V0 draft protocol guarantees from production trust requirements such as key custody, public-key discovery, distributed replay, distributed revocation, caller identity, distributed rate limiting, audit storage, or monitoring.

## Conflict and Constraint Recording

Before scheduling review work, record constraints that affect trust in the result or what can be published:

- conflicts of interest;
- report disclosure constraints;
- attribution constraints;
- NDA or embargo terms;
- whether public issue references are allowed;
- whether remediation verification can cite the reviewer or report reference.

Record these constraints in `docs/security/external-review-outreach-v0.md` before the review starts. If constraints prevent useful public remediation evidence, treat that as a selection risk and document how findings can still be tracked.

## Candidate Evaluation Table

Use this table when evaluating candidates. Do not add a candidate row unless outreach or selection work has actually started.

| Reviewer or firm | Fit | Constraints | Decision |
| --- | --- | --- | --- |
| _none_ | No reviewer has been selected yet. | _none_ | _none_ |

## Non-Claim Boundary

No production-ready or externally audited claim may be made from reviewer selection, candidate evaluation, outreach, quotes, availability checks, scheduling, NDAs, or reviewer interest.

Permitted wording before review completion is limited to process status, for example:

- "Rava is evaluating external reviewers for the R3 V0 draft target."
- "Reviewer selection criteria are recorded in `docs/security/external-review-selection-v0.md`."

Forbidden wording includes broad claims such as:

- "Rava is externally audited."
- "Rava has reviewer approval."
- "Rava is production-ready."
