# Rava V0 Draft Release Checklist

This checklist is for a Rava V0 draft release candidate. It is not a production readiness checklist, an external security review, or permission to describe Rava as production-ready security software.

Use this checklist before tagging, publishing crates, publishing wrapper artifacts, or announcing a V0 draft release candidate.

## Pre-Tag Gate

Run the full local gate from a clean working tree:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p rava -- demo flight-booking
cargo package --workspace
cargo check --manifest-path fuzz/Cargo.toml
cargo check -p rava-wasm --target wasm32-unknown-unknown
npm --prefix packages/rava-wasm-js test
(cd packages/rava-wasm-js && npm pack --dry-run)
```

Regenerate deterministic fixtures and confirm the diff is expected:

```bash
cargo run -p rava -- demo flight-booking --write-fixtures examples/flight-booking --deterministic-fixtures
cp examples/flight-booking/*.json test-vectors/v0/flight-booking/
git diff -- examples test-vectors
```

Unexpected fixture or test-vector drift must be investigated before tagging.

## Optional Fuzz Campaign

Long-running fuzz campaigns are not part of the default gate. If a fuzz campaign is run for this release candidate, record the command, corpus, duration, and any minimized crash input in the release notes or review register.

```bash
cargo fuzz run v0_wire_entrypoints
```

## Review Artifacts

Before publishing a V0 draft release candidate, update or confirm:

- `SECURITY.md`;
- `docs/security/release-audit-v0.md`;
- `docs/security/review-plan-v0.md`;
- `docs/security/review-register-v0.md`;
- `docs/security/threat-model-v0.md`;
- `docs/security/fuzz-campaigns/template-v0.md`;
- `docs/operations/production-trust-v0.md`;
- `docs/operations/key-custody-v0.md`;
- `docs/operations/key-discovery-v0.md`;
- `docs/operations/distributed-replay-v0.md`;
- `docs/operations/distributed-revocation-v0.md`;
- `docs/operations/audit-storage-v0.md`;
- `docs/operations/caller-identity-v0.md`;
- `docs/operations/distributed-rate-limits-v0.md`;
- `docs/operations/monitoring-v0.md`;
- `docs/roadmap.md`;
- [../roadmap.md](../roadmap.md);
- [../security/release-audit-v0.md](../security/release-audit-v0.md);
- [../security/review-register-v0.md](../security/review-register-v0.md);
- [../security/threat-model-v0.md](../security/threat-model-v0.md);
- [../protocol/compatibility-policy-v0.md](../protocol/compatibility-policy-v0.md);
- [../protocol/v1-preview-surface.md](../protocol/v1-preview-surface.md), if the preview surface changed;
- [../protocol/v1-preview-migration.md](../protocol/v1-preview-migration.md), if the preview surface changed;
- release notes for compatibility-impacting changes, using [notes-template-v0.md](notes-template-v0.md).

The repo-relative template path is `docs/release/notes-template-v0.md`.

If no outside review has happened, release notes and audit docs must preserve the statement: No external security review has been completed.

## Publication Guardrails

A V0 draft release must not claim production readiness.

Before publishing, confirm package descriptions, README text, release notes, and announcement copy:

- describe Rava as a draft/reference implementation or developer preview;
- do not describe the preview service as a production authorization service;
- do not claim distributed trust, global identity, managed custody, managed audit storage, distributed replay, distributed revocation freshness, distributed rate limiting, or external audit coverage;
- link to the threat model, release audit, and production operations requirements;
- state any known compatibility-impacting changes.

## Artifact Checks

Before publishing Rust crates or wrapper packages:

- confirm `Cargo.toml` workspace repository and license metadata are correct;
- confirm local path dependencies include publishable versions;
- run `cargo package --workspace` without `--allow-dirty`;
- confirm the TypeScript package test builds the WASM wrapper and runs V0 vectors;
- from `packages/rava-wasm-js`, run `npm pack --dry-run` and confirm only intended package files are included;
- confirm generated build output and local package caches are not committed unless intentionally documented.

## Post-Tag Verification

After tagging or publishing a draft release candidate:

- verify the pushed tag points at the reviewed commit;
- verify package artifacts install or unpack as expected;
- rerun the demo from the tagged checkout or published artifact when practical;
- record any release-blocking issue in [../security/review-register-v0.md](../security/review-register-v0.md) or release notes.
