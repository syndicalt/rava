# Rava V1 Preview Surface

This document pins the developer-facing surface that V1 preview wrappers and operators can build against. Rava V1 preview is not a production authorization boundary, and it does not add trust guarantees beyond the Rust verifier.

The Rust core remains authoritative. Wrappers, services, and adapters must preserve fail-closed verification for malformed, unsigned, unverifiable, expired, revoked, replayed, or over-scoped inputs.

## Stable CLI Commands

The following command names are treated as stable for the V1 preview unless a compatibility note updates this document, fixtures, and tests in the same change:

- `rava version`
- `rava key generate`
- `rava key revoke`
- `rava demo flight-booking`
- `rava inspect action`
- `rava inspect capability-chain`
- `rava verify action`
- `rava verify receipt`
- `rava verify attestation`
- `rava attest sign`
- `rava audit export`
- `rava serve verify`

The preview verifier service supports these service-boundary options:

- `--addr`
- `--max-request-bytes`
- `--replay-store`
- `--revocation-store`
- `--audit-log`
- `--auth-token-env`
- `--caller-id`
- `--rate-limit-per-minute`
- `--metrics`

`--auth-token-env`, `--caller-id`, and `--rate-limit-per-minute` are local preview controls. `--caller-id` requires `--auth-token-env`, uses an audit-safe label syntax, and records an explicit deployment label in local audit entries; it is not inferred from the action actor or from request headers. `--rate-limit-per-minute` must be greater than zero when configured. These controls do not replace production caller identity, distributed rate limiting, or network-edge access control.

`rava verify action` supports `--trust-bundle` for a local `rava-static-trust-bundle-v0` signer-ID to public-key map. `--require-fresh-trust-bundle` requires that local bundle to include `fresh_until_unix` greater than verifier `now_unix`; missing or stale freshness fails closed before verifier execution. This is explicit static trust-policy input; it does not add dynamic key discovery, resolver selection, cache invalidation, rotation, rollback, or outage guarantees.

`rava verify action` also supports `--require-fresh-revocations` with `--revocation-store`. The local snapshot must include `fresh_until_unix` greater than verifier `now_unix`; otherwise verification fails closed before verifier execution. This is local snapshot freshness checking, not distributed revocation freshness.

`rava key revoke --id <signer-id> --revocation-store <path>` records a signer ID in the local revocation snapshot. Local file-backed updates are lock-serialized and merge existing revoked IDs before persisting. This is a local compromise-response helper, not managed key custody, rotation, key discovery, distributed revocation, or emergency propagation.

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
- when local rate limiting is configured, `rate_limit_scope` is `process` or `caller`;
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
- `caller_id_configured`;
- `rate_limit_per_minute`;
- `rate_limit_scope`;
- `metrics_configured`.

This endpoint reports local process configuration only. It does not prove key freshness, revocation freshness, replay coordination, shared quota state, cross-node rate-limit consistency, or external reachability.

## Metrics Shape

When `--metrics` is configured, `GET /metrics` returns Prometheus-style text counters for local HTTP statuses, verifier accepted/rejected decisions, verifier rejection codes, and audit-write failures. If `--auth-token-env` is configured, the same bearer-token gate protects `GET /metrics`.

Metric labels are bounded local categories such as route, status, decision, and rejection code. Metrics intentionally omit raw action payloads, capability envelopes, signatures, public or private keys, credentials, access tokens, action IDs, actor IDs, controller IDs, resource names, and constraints.

This endpoint is process-local preview evidence. Operators that need managed monitoring, alerting, dashboards, long-term retention, cross-node aggregation, or incident response must provide those systems outside the preview service.

## Audit Log Shape

When `--audit-log` is configured, the service appends one newline-delimited JSON entry per verifier decision. Entries contain:

- `service`;
- `action_id`;
- `actor_id`;
- `caller_id`;
- `controller_id`;
- `capability_id`;
- `accepted`;
- `rejection`;
- `verified_at_unix`.

Audit entries intentionally omit raw action intent, resource, constraints, capability envelopes, and signatures. On Unix, local audit log files are created owner-only, group/world-accessible existing logs are rejected, the final path is opened without following a symlink, and each append is flushed and synced before the verifier response is returned. Operators that need managed audit retention, export, tamper evidence, or privacy controls must provide those systems outside the preview service.

`rava audit export --audit-log <path>` converts local preview audit NDJSON into a JSON array. Optional `--since-unix` and `--until-unix` bounds are inclusive and filter on `verified_at_unix`; when a time filter is used, entries missing `verified_at_unix` fail closed. The exporter rejects entries that contain raw payload-style fields such as `action`, `capability_chain`, `intent`, `resource`, `constraints`, or `proof`. This is a local export helper, not managed audit export.

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
