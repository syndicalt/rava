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

## HTTP Service Shape

`POST /verify/action` still wraps Rust verification and returns:

- `service`;
- `accepted`;
- `rejection`, with stable `code` and optional `subject`.

`GET /healthz` now reports local configuration flags for request limits, replay/revocation stores, audit logging, auth, and rate limiting. Health output is local process state only; it does not prove key freshness, revocation freshness, replay coordination, or production readiness.

## Audit Output

`--audit-log` writes newline-delimited JSON decision metadata. It intentionally omits raw action intent, resource, constraints, capability envelopes, and signatures.

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
- external security review coverage.

Those requirements are documented in `docs/operations/production-trust-v0.md`.
Production caller identity is tracked in [../operations/caller-identity-v0.md](../operations/caller-identity-v0.md), and distributed rate limiting is tracked in [../operations/distributed-rate-limits-v0.md](../operations/distributed-rate-limits-v0.md).
