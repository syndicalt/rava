# Rava V1 Preview Surface

This document pins the developer-facing surface that V1 preview wrappers and operators can build against. Rava V1 preview is not a production authorization boundary, and it does not add trust guarantees beyond the Rust verifier.

The Rust core remains authoritative. Wrappers, services, and adapters must preserve fail-closed verification for malformed, unsigned, unverifiable, expired, revoked, replayed, or over-scoped inputs.

## Stable CLI Commands

The following command names are treated as stable for the V1 preview unless a compatibility note updates this document, fixtures, and tests in the same change:

- `rava version`
- `rava key generate`
- `rava demo flight-booking`
- `rava inspect action`
- `rava inspect capability-chain`
- `rava verify action`
- `rava verify receipt`
- `rava verify attestation`
- `rava attest sign`
- `rava serve verify`

The preview verifier service supports these service-boundary options:

- `--addr`
- `--max-request-bytes`
- `--replay-store`
- `--revocation-store`
- `--audit-log`
- `--auth-token-env`
- `--rate-limit-per-minute`

`--auth-token-env` and `--rate-limit-per-minute` are local preview controls.
They do not replace production caller identity, distributed rate limiting, or
network-edge access control.

`rava verify action` supports `--trust-bundle` for a local `rava-static-trust-bundle-v0` signer-ID to public-key map. This is explicit static trust-policy input; it does not add dynamic key discovery, resolver freshness, rotation, rollback, or outage guarantees.

`rava verify action` also supports `--require-fresh-revocations` with `--revocation-store`. The local snapshot must include `fresh_until_unix` greater than verifier `now_unix`; otherwise verification fails closed before verifier execution. This is local snapshot freshness checking, not distributed revocation freshness.

## HTTP Request Shape

`POST /verify/action` accepts a JSON object with these fields:

- `action`: signed action envelope;
- `capability_chain`: ordered signed capability envelopes;
- `actor_public_key_hex`: authentic public key for the action actor;
- `issuer_public_keys`: map from capability issuer ID to authentic public key;
- `now_unix`: verifier time as Unix seconds.

Callers are responsible for authentic public-key selection, replay/revocation freshness, and any request authentication before the preview service is exposed beyond a local development boundary.

## HTTP Response Shape

Successful HTTP parsing and verifier execution returns `200 OK` with:

- `service`: `rava-verifier-preview-v0`;
- `accepted`: boolean verifier decision;
- `rejection`: `null` when accepted, otherwise an object with `rejection.code` and `rejection.subject`.

Stable rejection codes and subject meanings are documented in `docs/operators/rejection-codes-v0.md`.

## HTTP Error Shape

Service-boundary failures return JSON with:

- `service`: `rava-verifier-preview-v0`;
- `error`: stable local error string.

The preview service currently emits these non-verifier HTTP statuses before
authorization decisions:

- `401 Unauthorized` for missing or mismatched bearer tokens when `--auth-token-env` is configured;
- `429 Too Many Requests` for local per-process rate-limit exhaustion, with `rate_limit_per_minute`;
- `413 Payload Too Large` when `Content-Length` exceeds `max_request_bytes`, with `max_request_bytes`;
- `431 Request Header Fields Too Large` when request headers exceed the local header limit;
- `404 Not Found` for routes other than `GET /healthz` and `POST /verify/action`.

## Health Shape

`GET /healthz` returns `200 OK` with service status and local configuration flags:

- `service`;
- `status`;
- `max_request_bytes`;
- `replay_store_configured`;
- `revocation_store_configured`;
- `audit_log_configured`;
- `auth_required`;
- `rate_limit_per_minute`.

This endpoint reports local process configuration only. It does not prove key freshness, revocation freshness, replay coordination, or external reachability.

## Audit Log Shape

When `--audit-log` is configured, the service appends one newline-delimited JSON entry per verifier decision. Entries contain:

- `service`;
- `action_id`;
- `actor_id`;
- `controller_id`;
- `capability_id`;
- `accepted`;
- `rejection`;
- `verified_at_unix`.

Audit entries intentionally omit raw action intent, resource, constraints, capability envelopes, and signatures. On Unix, local audit log files are created owner-only, group/world-accessible existing logs are rejected, the final path is opened without following a symlink, and each append is flushed and synced before the verifier response is returned. Operators that need managed audit retention, export, tamper evidence, or privacy controls must provide those systems outside the preview service.

## Production Requirements

The preview surface is not a production authorization boundary. Production deployments must define the surrounding systems outside the V0 core and preview service:

- [key custody](../operations/key-custody-v0.md);
- [public-key discovery](../operations/key-discovery-v0.md);
- [distributed replay coordination](../operations/distributed-replay-v0.md);
- [distributed revocation freshness](../operations/distributed-revocation-v0.md);
- [managed audit storage](../operations/audit-storage-v0.md);
- [caller identity](../operations/caller-identity-v0.md);
- [distributed rate limiting](../operations/distributed-rate-limits-v0.md);
- [monitoring](../operations/monitoring-v0.md).

## Rejection-Code Subjects

`rejection.code` and `rejection.subject` follow `docs/operators/rejection-codes-v0.md`. Wrappers should preserve both fields instead of translating them into broad local categories.

Adding stricter fail-closed rejection behavior can remain V0-compatible when fixtures, docs, and tests are updated together. Making an existing rejected object accepted requires a new protocol version.
