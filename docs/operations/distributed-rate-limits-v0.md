# Rava Production Distributed Rate Limits V0

This runbook defines distributed rate-limit requirements for production verifier deployments. Distributed rate limits are not implemented by the V0 preview service.

The preview service can enforce a local per-process request limit. That is useful for development and single-process demos, but it is not a shared quota system across nodes, tenants, regions, or deployments.

## Local Preview Guardrails

When `rava serve verify --rate-limit-per-minute <N>` is configured, the preview service enforces a local in-memory request limit for the running process. The configured limit must be greater than zero; invalid zero limits fail closed at startup. If `--caller-id <label>` is also configured, health and 429 responses report `rate_limit_scope` as `caller`; otherwise they report `process`.

The caller-scoped preview label depends on `--caller-id`, which requires `--auth-token-env`. It is not inferred from action actors and does not create a shared quota across processes, nodes, tenants, regions, or deployments.

This is local abuse-control evidence for development and controlled single-process deployments. It is not a distributed rate-limit system, burst policy, outage policy, abuse-response process, or cross-node consistency guarantee.

## Required Properties

A production rate-limit system should define:

- shared quota state;
- caller identity used for quota keys;
- per-tenant and per-service limits;
- cross-node consistency expectations;
- burst behavior;
- outage behavior;
- audit and monitoring for accepted and rejected requests;
- abuse-response escalation.

## Caller Identity

Distributed rate limits depend on authenticated caller identity from the ingress layer. They should not derive quota identity only from action actors, because one caller may submit actions for many actors and one actor may appear across many callers.

## Fail-Closed Rule

If a deployment requires rate limiting and cannot read or update shared quota state, it should fail closed or route to a documented degraded policy.

Continuing without a working quota system is accepted deployment risk and does not change Rava verifier semantics.

## Cross-Node Behavior

Cross-node rate limits should state whether quotas are globally consistent, region-local, eventually consistent, or scoped to a smaller boundary.

Operators should document how duplicate requests, retries, and replay attempts affect quotas.
