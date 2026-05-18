# Rava

Rava is action-native authorization for autonomous agents.

Most auth systems start by asking who is logged in. Rava starts by asking whether this exact signed action is allowed right now by this exact signed delegation chain.

Rava V0 is a draft reference implementation, not production-ready security software. It is suitable for protocol development, examples, interop work, and review. It has not received an external cryptographic or security audit.

## Table of Contents

- [Status](#status)
- [What Rava Implements](#what-rava-implements)
- [What Rava Does Not Implement](#what-rava-does-not-implement)
- [Security Posture](#security-posture)
- [Mental Model](#mental-model)
- [Repository Layout](#repository-layout)
- [Requirements](#requirements)
- [Quickstart](#quickstart)
- [CLI Reference](#cli-reference)
- [HTTP Verifier Preview](#http-verifier-preview)
- [Examples, Test Vectors, and Schemas](#examples-test-vectors-and-schemas)
- [Verification Gates](#verification-gates)
- [Roadmap](#roadmap)
- [License Model](#license-model)

## Status

Rava V0 is complete as a draft reference implementation for review, examples, interop work, and integration design. The current implementation focuses on signed actions, delegated capabilities, replay/revocation checks, receipts, attestations, examples, and language-neutral wire artifacts.

This repository should still be treated as a draft protocol package, not a production authorization system.

## What Rava Implements

The current Rust core and CLI implement:

- deterministic canonical JSON for signed payloads;
- strict V0 protocol version checks for signed protocol objects;
- canonical UUID v4 nonce checks for signed protocol objects;
- non-empty, sorted, duplicate-free capability operation lists;
- local Ed25519 signer identities;
- signer IDs bound to their public key prefix;
- signed capability minting;
- attenuated capability delegation;
- verifier-enforced delegation attenuation for received capability chains;
- delegation cannot remove parent constraint keys;
- signed action envelopes;
- root capability issuer must match the action controller;
- strict SHA-256 action context references;
- in-memory and file-backed one-time action replay checks;
- in-memory and file-backed revocation checks;
- delegated action verification;
- action constraints must be contained by final capability constraints;
- signed verification receipts bound to the verified capability chain hash;
- signed post-action attestations;
- strict SHA-256 attestation evidence references;
- local key generation and owner-only private-key file handling on Unix;
- a read-only inspector for wire objects;
- a local preview HTTP verifier;
- committed flight-booking examples, test vectors, and wire-shape schemas.

## What Rava Does Not Implement

Rava V0 is not a global identity provider, reputation market, wallet, custody system, OAuth replacement, blockchain protocol, or distributed revocation network.

V0 assumes verifier callers provide authentic public keys and sufficiently fresh replay/revocation registries. DID resolution, key custody, distributed freshness, managed audit storage, and production service operations are roadmap work, not implemented guarantees today.

## Security Posture

The trusted core is Rust. `unsafe` is forbidden. Rava uses existing cryptographic crates and does not invent cryptographic primitives.

The verifier fails closed: malformed, unsigned, tampered, expired, revoked, replayed, or over-scoped inputs are rejected.

The V0 threat model is documented in [docs/security/threat-model-v0.md](docs/security/threat-model-v0.md). It distinguishes implemented guarantees from verifier input assumptions and non-goals.

The current repo audit note is [docs/security/release-audit-v0.md](docs/security/release-audit-v0.md). It is an internal readiness audit, not an external security assessment.

The V0 draft completion audit is [docs/security/v0-draft-completion-audit.md](docs/security/v0-draft-completion-audit.md). It defines what "complete" means for the draft reference implementation without converting production requirements into implemented guarantees.

Vulnerability reporting and external review intake are documented in [SECURITY.md](SECURITY.md).

## Mental Model

- **Identity:** who can sign.
- **Signature:** proof that a signer approved exact bytes.
- **Capability:** what a signer is allowed to do.
- **Delegation:** a narrowed capability passed from one signer to another.
- **Action:** what an agent is trying to do now.
- **Replay check:** prevents the same accepted signed action from being consumed twice.
- **Verifier:** the judge that checks signatures, expiry, revocation, delegation, resource, operation, actor, and constraints.
- **Verification receipt:** signed proof that a verifier accepted or rejected an action at a specific time.
- **Attestation:** signed evidence about what happened after verification.

## Repository Layout

- `crates/rava-core`: trusted protocol core, canonicalization, signing, verification, replay, revocation, receipts, and attestations.
- `crates/rava-cli`: CLI, demos, fixture generation, inspection, and local verifier service preview.
- `docs/protocol/rava-v0.md`: V0 protocol specification.
- `docs/protocol/compatibility-policy-v0.md`: compatibility and versioning rules for V0 draft artifacts.
- `docs/protocol/time-semantics-v0.md`: verifier time, expiry, replay, and revocation freshness assumptions.
- `docs/operators/rejection-codes-v0.md`: operator-facing verifier rejection codes.
- `docs/security/threat-model-v0.md`: V0 threat model.
- `docs/security/review-guide-v0.md`: review scope, evidence map, and reviewer questions.
- `docs/security/review-register-v0.md`: external review findings and remediation tracking.
- `docs/security/release-audit-v0.md`: current publication-readiness audit notes.
- `docs/security/v0-draft-completion-audit.md`: completion audit for the V0 draft reference implementation.
- `docs/roadmap.md`: functional protocol roadmap.
- `docs/release/v0-draft-checklist.md`: draft release gate, artifact, and publication guardrail checklist.
- `docs/release/notes-template-v0.md`: draft release notes template for verification and compatibility notes.
- `docs/operations/production-trust-v0.md`: production trust requirements around the V0 verifier.
- `docs/operations/key-custody-v0.md`: production key custody, rotation, and compromise response requirements.
- `docs/operations/key-discovery-v0.md`: production public-key discovery and resolver freshness requirements.
- `docs/operations/distributed-replay-v0.md`: production distributed replay coordination requirements.
- `docs/operations/distributed-revocation-v0.md`: production revocation publication, freshness, and outage requirements.
- `docs/operations/audit-storage-v0.md`: production audit retention, privacy, export, and failure-policy requirements.
- `docs/operations/caller-identity-v0.md`: production caller identity requirements.
- `docs/operations/distributed-rate-limits-v0.md`: production distributed rate-limit requirements.
- `docs/operations/monitoring-v0.md`: production monitoring and incident-response requirements.
- `docs/interop/roadmap-v0.md`: wrapper and adapter sequencing.
- `docs/schemas/v0`: wire-shape JSON Schemas.
- `examples/flight-booking`: committed example wire objects.
- `test-vectors/v0`: language-neutral compatibility vectors.

## Requirements

- Rust toolchain with Cargo.
- A shell capable of running the commands below.
- No external services are required for the local demo or test suite.

## Quickstart

Run the full local demo:

```bash
cargo run -p rava -- demo flight-booking
```

Expected output:

```text
Rava verification accepted: true
Rava replay rejected: true
Rava receipt verified: true
```

Generate fresh demo fixtures:

```bash
cargo run -p rava -- demo flight-booking --write-fixtures /tmp/rava-fixtures
```

Regenerate the committed flight-booking corpus deterministically:

```bash
cargo run -p rava -- demo flight-booking --write-fixtures examples/flight-booking --deterministic-fixtures
cp examples/flight-booking/*.json test-vectors/v0/flight-booking/
```

Run the workspace tests:

```bash
cargo test --workspace
```

## CLI Reference

The examples below assume `rava` is installed or otherwise available on `PATH`. From this repository, prefix CLI commands with `cargo run -p rava --`.

Generate a local verifier key:

```bash
rava key generate --kind service --out verifier-key.json
```

Verify a signed action against a capability chain:

```bash
rava verify action \
  --action action.json \
  --capability-chain capability-chain.json \
  --actor-key <actor-public-key-hex> \
  --issuer-key <issuer-id=issuer-public-key-hex> \
  --now-unix 1650000000 \
  --replay-store replay.json \
  --revocation-store revocations.json \
  --require-fresh-revocations \
  --receipt-out receipt.json \
  --receipt-key verifier-key.json
```

For controlled deployments that use explicit static trust roots, `rava verify action` can also read signer public keys from a local trust bundle:

```json
{
  "version": "rava-static-trust-bundle-v0",
  "fresh_until_unix": 1650003600,
  "keys": {
    "rava:agent:<actor-public-key-prefix>": "<actor-public-key-hex>",
    "rava:human:<issuer-public-key-prefix>": "<issuer-public-key-hex>"
  }
}
```

```bash
rava verify action \
  --action action.json \
  --capability-chain capability-chain.json \
  --trust-bundle trust-bundle.json \
  --require-fresh-trust-bundle \
  --now-unix 1650000000
```

If explicit `--actor-key` or `--issuer-key` values conflict with the trust bundle, verification fails before an authorization decision is reported. `--require-fresh-trust-bundle` requires the bundle to include `fresh_until_unix` greater than verifier `now_unix`; missing or stale freshness fails closed before verifier execution. Static trust bundles are local trust-policy input; they do not provide dynamic DID, web, registry, cache invalidation, rollback, outage, or rotation guarantees.

Verify a signed receipt:

```bash
rava verify receipt \
  --receipt receipt.json \
  --verifier-key <verifier-public-key-hex>
```

Sign and verify a post-action attestation:

```bash
rava attest sign \
  --key evaluator-key.json \
  --out attestation.json \
  --action-id act_demo \
  --outcome accepted \
  --subject travel.booking \
  --occurred-at-unix 1650000000 \
  --evidence-hash sha256:<64-lowercase-hex>

rava verify attestation \
  --attestation attestation.json \
  --evaluator-key <evaluator-public-key-hex>
```

Inspect wire objects without making an authorization decision:

```bash
rava inspect action --action action.json
rava inspect capability-chain --capability-chain capability-chain.json
```

`--receipt-out` signs the decision and prints the verifier public key needed to verify that receipt. If `--receipt-key` is omitted, the CLI uses an ephemeral verifier key. Local key files contain private key material and should be treated as secrets.

`rava inspect` is read-only and does not verify signatures, replay state, revocation state, expiry, or attenuation. Use `rava verify` for authorization decisions.

`rava key generate` refuses to overwrite an existing key file unless `--force` is passed. On Unix, Rava writes local key files with owner-only permissions and rejects loading private key files that are readable, writable, or executable by group or others.

`rava key revoke --id <signer-id> --revocation-store revocations.json` records a signer ID in the local revocation snapshot used by `rava verify action --revocation-store`. Local file-backed updates are lock-serialized and merge existing revoked IDs before persisting. This is a local compromise-response helper, not managed custody, rotation, recovery, key-discovery infrastructure, or distributed revocation propagation.

## HTTP Verifier Preview

Start the local preview verifier:

```bash
RAVA_VERIFIER_TOKEN="$(openssl rand -hex 32)" \
rava serve verify --addr 127.0.0.1:8787 --max-request-bytes 1048576 --replay-store replay.json --revocation-store revocations.json --require-fresh-revocations --audit-log audit.ndjson --auth-token-env RAVA_VERIFIER_TOKEN --caller-id local-operator --rate-limit-per-minute 120 --metrics
```

It exposes `POST /verify/action` for JSON requests containing an action, capability chain, actor public key, issuer public key map, and `now_unix`.
It also exposes `GET /healthz` with the service name, status, and configured request body limit.
`--max-request-bytes` rejects oversized request bodies before verifier parsing.
`--replay-store` records accepted action IDs in a local file and rejects later replays. Local file-backed consumption is lock-serialized for stale handles in the same filesystem boundary; it is not distributed replay coordination.
`--revocation-store` loads a local revoked-ID snapshot for signer and capability checks on each request.
`--require-fresh-revocations` requires the local revocation snapshot to include `fresh_until_unix` greater than verifier `now_unix`; missing or stale freshness fails closed before verifier execution.
`--audit-log` appends newline-delimited JSON decision metadata without raw action intent, resource, constraints, capability envelopes, or signatures. On Unix, local audit log files are created owner-only, and group/world-accessible existing logs are rejected.
`--auth-token-env` requires `Authorization: Bearer <token>` on every request and reads the token from an environment variable so it is not passed on the command line.
`--caller-id` records an explicit deployment-configured caller label in audit entries and requires `--auth-token-env`. Caller labels must use the audit-safe ASCII syntax accepted by the CLI; they are not inferred from the signed action actor and do not implement production caller-to-policy mapping.
`--rate-limit-per-minute` applies a positive local request limit. Invalid zero limits fail closed at startup. Health and 429 responses report whether the preview limit is process-scoped or scoped to the explicit `--caller-id` label.
`--metrics` enables `GET /metrics` with Prometheus-style process-local counters for HTTP statuses, verifier decisions, rejection codes, and audit-write failures without raw action payloads, capability envelopes, signatures, keys, credentials, or tokens.

The pinned preview service request, response, health, audit, and error shapes are documented in [docs/protocol/v1-preview-surface.md](docs/protocol/v1-preview-surface.md).

The preview service is not a production authorization service. It does not implement key discovery, distributed replay coordination, distributed revocation freshness, caller identity, distributed rate limiting, managed audit retention/export, or managed monitoring and alerting.

Export local preview audit metadata:

```bash
rava audit export --audit-log audit.ndjson --since-unix 1650000000 --until-unix 1650003600
```

`rava audit export` converts local newline-delimited audit metadata into a JSON array. Optional `--since-unix` and `--until-unix` bounds are inclusive and filter on `verified_at_unix`; when a time filter is used, entries missing `verified_at_unix` fail closed. The exporter also fails closed if an entry contains raw payload-style fields such as action intent, resource, constraints, proofs, actions, or capability chains. It is a local development export helper, not managed retention, access control, tamper evidence, legal hold, or production audit export.

## Examples, Test Vectors, and Schemas

Committed wire examples live in `examples/flight-booking`. They include a signed action, capability chain, receipt, attestation, public keys, empty replay/revocation stores, and tampered receipt/attestation examples for verifier checks. They intentionally do not include private key material.

Language-neutral V0 test vectors live in `test-vectors/v0`. They are intended for independent implementations and are validated by the Rust CLI regression suite.

Wire-shape schemas live in `docs/schemas/v0`. They are parser/preflight aids only; they are not a substitute for verifier checks.

## Verification Gates

Before claiming a release candidate is ready, run:

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

These gates verify formatting, lint cleanliness, Rust regression tests, the local flight-booking demo, crate packaging, fuzz-target compilation, the WASM verifier wrapper build, the TypeScript WASM package, and the wrapper package dry-run contents. They do not replace an external security review.

The full V0 draft release checklist is [docs/release/v0-draft-checklist.md](docs/release/v0-draft-checklist.md).

## Roadmap

The functional roadmap is [docs/roadmap.md](docs/roadmap.md). Interop sequencing for WASM, TypeScript, DID/key resolution, MCP adapters, and OAuth exchange is documented in [docs/interop/roadmap-v0.md](docs/interop/roadmap-v0.md).

Release process guardrails are documented in [docs/release/v0-draft-checklist.md](docs/release/v0-draft-checklist.md) and [docs/release/notes-template-v0.md](docs/release/notes-template-v0.md). Production trust requirements are documented in [docs/operations/production-trust-v0.md](docs/operations/production-trust-v0.md), and external review findings are tracked in [docs/security/review-register-v0.md](docs/security/review-register-v0.md).

## License Model

Rava uses an open-core model. The protocol specification, Rust core, CLI, and examples are open source under Apache-2.0. Commercial products can be built above the core: hosted verification, managed revocation, enterprise policy, audit tooling, custody integrations, support, and certification.
