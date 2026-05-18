# Rava V1 Preview Migration Notes

These notes summarize the developer-facing changes from the V0 draft baseline toward the V1 preview surface. Rava V1 preview is still not production-ready security software.

## Compatibility Summary

No V0 signed wire object version is changed. The signed versions remain:

- `rava-action-v0`;
- `rava-capability-v0`;
- `rava-verification-receipt-v0`;
- `rava-attestation-v0`.

The Rust verifier remains authoritative. Wrappers and services must preserve fail-closed behavior for malformed, unsigned, unverifiable, expired, revoked, replayed, or over-scoped input.

The pinned preview surface lives in `docs/protocol/v1-preview-surface.md`.

## CLI Changes

The `rava serve verify` preview service has gained optional service-boundary controls:

- `--max-request-bytes`;
- `--auth-token-env`;
- `--rate-limit-per-minute`;
- `--replay-store`;
- `--revocation-store`;
- `--audit-log`.

Operators using the preview service should prefer all six controls for nontrivial local integration testing. The bearer token is read from an environment variable so token material is not exposed in command arguments.

`rava verify action` also supports `--trust-bundle` for a local `rava-static-trust-bundle-v0` signer-ID to public-key map. `--require-fresh-trust-bundle` requires `fresh_until_unix` greater than verifier `now_unix`; missing or stale freshness fails closed before verifier execution. This is static caller trust-policy input for controlled deployments; it does not add dynamic resolver selection, cache invalidation, rotation, rollback, or outage guarantees.

`rava verify action --require-fresh-revocations` requires a local `--revocation-store` snapshot to include `fresh_until_unix` greater than verifier `now_unix`. Missing or stale freshness fails closed before verifier execution. This is local snapshot freshness checking, not distributed revocation publication or outage handling.

`rava key revoke --id <signer-id> --revocation-store <path>` updates the local revocation snapshot for a suspected compromised signer. File-backed updates are lock-serialized, merge existing revoked IDs before persisting, and preserve existing `fresh_until_unix` metadata. This remains local compromise-response tooling, not production custody, distributed revocation, or emergency propagation.

## HTTP Service Shape

`POST /verify/action` still wraps Rust verification and returns:

- `service`;
- `accepted`;
- `rejection`, with stable `code` and optional `subject`.

`GET /healthz` now reports local configuration flags for request limits, replay/revocation stores, audit logging, auth, caller labels, rate limiting, and metrics. Health output is local process state only; it does not prove key freshness, revocation freshness, replay coordination, caller-to-policy mapping, monitoring coverage, or production readiness.

`rava serve verify --metrics` enables `GET /metrics` with process-local Prometheus-style counters. This is metadata-only preview evidence, not managed monitoring, alerting, retention, cross-node aggregation, or incident response.

`rava serve verify --caller-id <label>` records an explicit deployment caller label in audit entries and requires `--auth-token-env`. It is local audit correlation evidence, not tenant isolation or production caller identity.

When `--rate-limit-per-minute` is configured, health and 429 responses now report `rate_limit_scope`. The value is `caller` when the local preview limit is tied to an explicit `--caller-id` label and `process` otherwise. This is not shared quota state or distributed rate limiting.

## Audit Output

`--audit-log` writes newline-delimited JSON decision metadata. It intentionally omits raw action intent, resource, constraints, capability envelopes, and signatures.

`rava audit export --audit-log <path>` converts local preview audit NDJSON into a JSON array and rejects raw payload-style fields. This is local review tooling, not managed audit export.

Operators that need managed retention, export, tamper evidence, privacy controls, or legal holds must provide those systems outside the preview service.

## Wrapper Changes

The `rava-wasm` crate exposes `verify_action_json` as a WASM boundary around the Rust verifier. It does not reimplement signatures, canonicalization, or attenuation.

The `rava-wasm-js` package builds the WASM wrapper and exposes a TypeScript `verifyAction` function. Its tests run the committed V0 flight-booking vectors through the TypeScript API.

Wrapper callers must still provide authentic public keys, sufficient revocation freshness, and any replay coordination required by their deployment.

## Remaining Production Gaps

The preview surface still does not provide:

- caller identity;
- distributed replay coordination;
- distributed revocation freshness;
- distributed rate limiting;
- key custody;
- public-key discovery trust policy;
- managed audit retention/export;
- operational monitoring for verifier availability and rejection patterns;
- external security review coverage.

Those requirements are documented in `docs/operations/production-trust-v0.md`.
Detailed owners are:

- caller identity: [../operations/caller-identity-v0.md](../operations/caller-identity-v0.md);
- distributed replay coordination: [../operations/distributed-replay-v0.md](../operations/distributed-replay-v0.md);
- distributed revocation freshness: [../operations/distributed-revocation-v0.md](../operations/distributed-revocation-v0.md);
- distributed rate limiting: [../operations/distributed-rate-limits-v0.md](../operations/distributed-rate-limits-v0.md);
- key custody: [../operations/key-custody-v0.md](../operations/key-custody-v0.md);
- public-key discovery trust policy: [../operations/key-discovery-v0.md](../operations/key-discovery-v0.md);
- managed audit retention/export: [../operations/audit-storage-v0.md](../operations/audit-storage-v0.md);
- operational monitoring: [../operations/monitoring-v0.md](../operations/monitoring-v0.md);
- external security review coverage: [../security/review-register-v0.md](../security/review-register-v0.md).
