# Rava V0 Compatibility Policy

This policy defines what should remain stable during the V0 draft and what requires new fixtures, schemas, or protocol versions. It is intended for wrapper authors, test-vector users, and reviewers.

No compatibility change may weaken fail-closed verification.

## Stable During V0 Draft

These surfaces are treated as stable unless a documented V0 draft change updates fixtures and tests in the same patch:

- wire object versions: `rava-action-v0`, `rava-capability-v0`, `rava-verification-receipt-v0`, and `rava-attestation-v0`;
- signed field sets for actions, capabilities, receipts, and attestations;
- deterministic ID derivation from canonical pending objects;
- canonical JSON behavior for signed payloads;
- signer ID prefix binding to supplied public keys;
- verifier rejection code strings;
- V0 test-vector layout under `test-vectors/v0`;
- JSON schemas under `docs/schemas/v0`;
- deterministic flight-booking fixture regeneration behavior.

The V1 preview developer surface for CLI names, service JSON shapes, audit output, and rejection subjects is pinned in [v1-preview-surface.md](v1-preview-surface.md).

## Changes That Require New Test Vectors

New or updated V0 test vectors are required when a change affects:

- signed JSON fields;
- canonicalization output;
- derived IDs;
- signatures;
- fixture public keys or nonces;
- verifier acceptance or rejection for a committed vector;
- receipt or attestation verification behavior;
- schema-visible wire shape.

The Rust regression suite must continue to run the committed `test-vectors/v0` corpus.

## Changes That Require Schema Updates

Update schemas when a change affects parser-visible wire shape:

- object fields;
- enum values;
- required fields;
- additional-property policy;
- scalar formats such as hashes, timestamps, nonces, or public keys.

Schemas are preflight aids only. Updating schemas does not replace verifier tests.

## Changes That Require a New Protocol Version

Use a new protocol version instead of silently changing V0 semantics when a change:

- removes or renames a signed field;
- changes canonicalization rules;
- changes ID derivation rules;
- changes signature algorithms or key formats;
- changes attenuation semantics in a way that could alter authorization decisions;
- changes replay or revocation acceptance semantics;
- changes a rejection code meaning incompatibly;
- makes an existing rejected object accepted.

Adding a stricter rejection for malformed or over-scoped input can remain V0 if fixtures, docs, and tests are updated.

## Rejection Code Compatibility

Existing rejection code strings should remain stable during V0. New verifier checks may add new rejection codes. If an existing code must change, update:

- `docs/operators/rejection-codes-v0.md`;
- receipt/rejection tests;
- wrapper compatibility tests;
- release notes.

## Release Checklist

Before tagging a V0 draft release:

- run the full local gate;
- regenerate deterministic fixtures and confirm no unexpected diff;
- confirm `test-vectors/v0` and schemas match the intended wire shape;
- update `docs/security/release-audit-v0.md`;
- document any compatibility-impacting change in release notes.
