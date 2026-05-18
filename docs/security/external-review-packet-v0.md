# Rava V0 External Review Packet

This packet is a handoff manifest for an external security review of the Rava V0 draft reference implementation. It is not evidence that Rava has been externally reviewed, certified, or approved for production use.

## Freeze Rule

Before sending this packet, record the immutable commit SHA or signed tag under review in `docs/security/review-register-v0.md`, release notes draft, and reviewer correspondence.

The current review-candidate notes are `docs/release/v0-review-candidate-2026-05-18-r3.md`.

Review candidate notes are handoff metadata outside the frozen target tree when they are created after a tag is selected and verified. They are a post-target control-plane artifact. Do not treat post-target control-plane docs as reviewed target contents unless a reviewer explicitly agrees to include that later commit in scope.

The historical R2 review-candidate notes are `docs/release/v0-review-candidate-2026-05-18-r2.md`.

The historical first review-candidate notes are `docs/release/v0-review-candidate-2026-05-18.md`.

The reviewer cover note is `docs/security/external-review-cover-note-v0.md`.

The kickoff runbook is `docs/security/external-review-kickoff-checklist-v0.md`.

Reviewer outreach is tracked in `docs/security/external-review-outreach-v0.md`.

The reviewer-facing outreach template is `docs/security/external-review-outreach-template-v0.md`.

The reviewer response intake template is `docs/security/external-review-response-intake-template-v0.md`.

The reviewer engagement request is `docs/security/external-review-request-v0.md`.

Do not change the target during review unless the reviewer explicitly agrees to review the new commit or tag. If the target changes, rerun the full verification baseline and record the new evidence.

The review target should be a clean commit on `master` with no uncommitted files and a passing full local gate.

## Packet Manifest

Send reviewers these repository artifacts from the frozen target:

- `README.md`;
- `SECURITY.md`;
- `docs/security/threat-model-v0.md`;
- `docs/protocol/rava-v0.md`;
- `docs/protocol/time-semantics-v0.md`;
- `docs/protocol/compatibility-policy-v0.md`;
- `docs/operators/rejection-codes-v0.md`;
- `docs/security/v0-draft-completion-audit.md`;
- `docs/security/review-plan-v0.md`;
- `docs/security/review-guide-v0.md`;
- `docs/security/review-register-v0.md`;
- `docs/security/external-review-kickoff-checklist-v0.md`;
- `docs/security/external-review-outreach-v0.md`;
- `docs/security/external-review-selection-v0.md`;
- `docs/security/external-review-response-intake-template-v0.md`;
- `docs/security/external-review-request-v0.md`;
- `docs/security/external-review-closeout-template-v0.md`;
- `docs/security/release-audit-v0.md`;
- `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints.md`;
- `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints-1800s.md`;
- `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints-3600s.md`;
- `docs/release/v0-draft-checklist.md`;
- `docs/operations/production-trust-v0.md`;
- `crates/rava-core/src`;
- `crates/rava-core/tests`, if present in the frozen target;
- `crates/rava-cli/tests`;
- `fuzz/fuzz_targets/v0_wire_entrypoints.rs`;
- `test-vectors/v0`;
- `examples/flight-booking`.

State plainly in the cover note that Rava V0 is a draft reference implementation, not production-ready security software.

## Verification Baseline

The frozen target should pass:

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

If fixture drift is relevant to the review, also run:

```bash
cargo run -p rava -- demo flight-booking --write-fixtures examples/flight-booking --deterministic-fixtures
cp examples/flight-booking/*.json test-vectors/v0/flight-booking/
git diff -- examples test-vectors
```

The expected fixture-drift diff is empty unless the review target intentionally changes examples or test vectors.

## Review Focus

Ask reviewers to prioritize:

- canonicalization of signed payloads;
- signer ID and public-key binding;
- Ed25519 verification usage;
- derived IDs for signed protocol objects;
- nonce validation;
- delegation attenuation;
- replay persistence and replay failure semantics;
- revocation semantics;
- receipt and attestation verification;
- fail-closed behavior for malformed, tampered, expired, revoked, replayed, or over-scoped inputs;
- documentation ambiguity that could cause unsafe integrations.

Production key custody, DID resolution, distributed replay, distributed revocation freshness, caller identity, distributed rate limiting, managed audit storage, hosted verifier operations, production monitoring, model-behavior proofs, reputation, and blockchain anchoring are not implemented V0 guarantees.

## Finding Handling

Record every external finding in `docs/security/review-register-v0.md` with an ID such as `RAVA-REVIEW-001`.

Use `docs/security/review-findings/template-v0.md` for each individual finding when more detail is needed than the register table can hold.

Use one of these states:

- `reported`;
- `accepted`;
- `remediated`;
- `verified`;
- `accepted-risk`;
- `out-of-scope`.

Each accepted finding should map to concrete remediation evidence:

- code, test, or documentation changes;
- remediation PR or commit;
- verification commands;
- reviewer or maintainer verification notes.

No finding that weakens fail-closed verification may remain unresolved for any release represented as production-ready. For a V0 draft release, accepted risk must be explicit, narrow, and paired with non-production language.

## Fuzz Evidence

The packet includes the bounded `v0_wire_entrypoints` fuzz campaign recorded at `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints.md`.

That campaign is useful negative evidence for one bounded run. It is not a proof of security and does not replace external review. If reviewers request longer fuzzing, copy `docs/security/fuzz-campaigns/template-v0.md` to a dated campaign file and record command, duration, seed, corpus, artifacts, crashes, remediation, and final rerun evidence.

## Additional Post-Candidate Evidence

A bounded 1800-second `v0_wire_entrypoints` fuzz campaign is recorded at `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints-1800s.md`.

That campaign ran on commit `becbff9e2326f5304822decf636aadcd0e37bb48` after the first frozen review-candidate target was recorded. It does not change the frozen review target for the first candidate. It ran before the R2 frozen review-candidate target was recorded and should be treated as supplemental review evidence for the R2 handoff.

A bounded 3600-second `v0_wire_entrypoints` fuzz campaign is recorded at `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints-3600s.md`.

That campaign ran on commit `78857884bd7f6feafcf781cfdde2ce4b89fcb8db` after the R2 frozen review-candidate target was recorded. It is included as supplemental evidence in the R3 frozen review target.

Like the shorter bounded campaigns, it is not a proof of security and not evidence that Rava has been externally reviewed.
