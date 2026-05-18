# Rava Production Trust and Operations V0

This document defines production systems that must surround Rava before it is represented as a production authorization system. These are not implemented guarantees in the V0 core protocol, CLI, preview service, WASM wrapper, or TypeScript package.

Rava V0 verifies signed actions and attenuated capability chains against caller-supplied keys and caller-supplied replay/revocation state. Production deployments must own the surrounding trust, freshness, custody, and monitoring systems.

The preview verifier service is not a production ingress boundary. Caller identity requirements are detailed in [caller-identity-v0.md](caller-identity-v0.md), and distributed rate-limit requirements are detailed in [distributed-rate-limits-v0.md](distributed-rate-limits-v0.md).

## Selected Deployment Profile

The selected V0 production-trust profile is a controlled deployment profile:

- single-tenant or small explicitly trusted tenant set;
- self-hosted or otherwise controlled verifier operation;
- explicit static trust roots before dynamic resolver policy;
- managed key storage owned by the deployment, not by Rava core;
- authenticated service ingress before caller-specific policy, quotas, or audit claims;
- fail-closed behavior when key, replay, revocation, caller, or audit state is missing, stale, ambiguous, or unverifiable;
- metadata-first audit and monitoring records that avoid raw sensitive action payloads;
- no production-ready authorization-system claim until the external review and production-trust evidence are complete.

This profile fits Rava's V0 audience: Rust and security-conscious protocol evaluators, agent-tool builders, and early controlled integrators. It does not fit broad unauthenticated public API operation, regulated production authorization, or multi-tenant managed service claims without additional professional security review and deployment evidence.

## Decision Baseline

These decisions convert the production-trust tracker into the default architecture target for V0 operations work. They do not mean the systems are implemented today.

| Tracker | Decision | Required Evidence |
| --- | --- | --- |
| [#118](https://github.com/syndicalt/rava/issues/118) key custody | Rava core verifies signatures but does not custody private keys. Production deployments must use managed or OS-backed key storage with rotation, recovery, and compromise response. Development keys are non-production only. | Documented custody boundary, operator access review, rotation and emergency rotation procedure, backup/recovery test, and compromise-response path. |
| [#119](https://github.com/syndicalt/rava/issues/119) public-key discovery | Start with explicit static trust bundles. Dynamic DID, web, registry, or resolver discovery requires a reviewed resolver policy before production use. | Trust roots, freshness target, cache lifetime, rollback handling, ambiguity handling, outage behavior, and fail-closed tests. |
| [#123](https://github.com/syndicalt/rava/issues/123) caller identity | Hosted verifier callers must authenticate independently from the signed action actor. Caller identity must not be inferred from unsigned headers or from the action actor alone. | Ingress authentication, caller-to-policy mapping, tenant boundary, rejection behavior for unknown callers, and audit-safe caller identifiers. |
| [#120](https://github.com/syndicalt/rava/issues/120) distributed replay | If more than one verifier can accept the same signed action, action-ID consumption must be atomic and durable before acceptance is reported. | Shared replay boundary, consume-before-accept semantics, duplicate rejection tests, outage behavior, and recovery procedure. |
| [#121](https://github.com/syndicalt/rava/issues/121) distributed revocation | Verifiers must fail closed unless revocation state satisfies local freshness policy. Stale or unavailable revocation state is a security decision, not a cache miss. | Revocation publication path, maximum staleness, emergency propagation, cache invalidation, outage behavior, and audit evidence. |
| [#122](https://github.com/syndicalt/rava/issues/122) audit storage | Store decision metadata and stable identifiers by default. Raw sensitive action payloads require an explicit data-handling policy. | Retention, access control, tamper evidence, privacy review, export path, deletion/legal-hold policy, and audit-write failure behavior. |
| [#124](https://github.com/syndicalt/rava/issues/124) distributed rate limits | Rate limits and abuse controls must be keyed to authenticated caller and tenant identity. Production multi-node deployments require shared quota state. | Caller/tenant quotas, shared quota consistency, retry behavior, accepted/rejected request accounting, and dependency outage behavior. |
| [#125](https://github.com/syndicalt/rava/issues/125) monitoring | Monitor verifier availability and trust-decision failures without logging private keys, credentials, or sensitive payload fields. | Metrics and alerts for rejection codes, replay attempts, missing keys, stale revocation, caller-auth failures, audit-write failures, latency, saturation, and dependency outages. |

## Local Controlled-Deployment Exercise

This exercise is zero-budget local readiness evidence for Rava's controlled deployment profile. It helps an operator rehearse the V0 guardrails with local files and process-local metrics before selecting production infrastructure.

1. Generate local development keys with `rava key generate`; do not commit or publish private-key files.
2. Build an explicit `rava-static-trust-bundle-v0` with `fresh_until_unix` greater than the verifier time.
3. Create a local revocation snapshot with `fresh_until_unix` greater than the verifier time.
4. Start the preview verifier with every local require flag enabled:

   ```sh
   rava serve verify \
     --addr 127.0.0.1:8787 \
     --max-request-bytes 1048576 \
     --replay-store replay.json \
     --require-replay-store \
     --revocation-store revocations.json \
     --require-fresh-revocations \
     --audit-log audit.ndjson \
     --require-audit-log \
     --auth-token-env RAVA_VERIFIER_TOKEN \
     --require-auth-token-env \
     --caller-id local-operator \
     --require-caller-id \
     --rate-limit-per-minute 120 \
     --require-rate-limit-per-minute \
     --metrics \
     --require-metrics
   ```

5. Verify an action through the explicit static trust bundle with `--require-fresh-trust-bundle`; keep the `Rava key source: static-trust-bundle` line as local key-source evidence.
6. Confirm `GET /healthz` reports the required guardrails for replay, revocation freshness, audit, ingress authentication, caller ID, rate limit, and metrics.
7. Exercise a duplicate action ID, a stale revocation snapshot, a missing public key, and an invalid caller configuration; each should fail closed or report the documented local preview rejection.
8. Export the local audit metadata with `rava audit export --output <path>` and confirm it contains decision metadata rather than raw action payloads, capability envelopes, signatures, private keys, credentials, or tokens.
9. Capture `GET /metrics` output without raw action payloads, capability envelopes, signatures, private keys, credentials, access tokens, or local store contents.
10. Record the command lines, tool versions, commit SHA, audit export path, metrics capture path, and observed failure modes in local operator notes.

This is not production key custody, distributed replay, distributed revocation, managed audit storage, production caller identity, distributed rate limiting, managed monitoring, or external security review evidence. It is a local rehearsal that can expose missing guardrails before a deployment commits to managed infrastructure.

## Key Custody

Production deployments need documented key custody for human, agent, service, evaluator, and verifier keys:

- generation ceremony and entropy source;
- storage boundary and access controls;
- backup and recovery;
- rotation schedule;
- emergency rotation;
- compromise response;
- operator access review.

Private keys must not be logged, shipped in examples, embedded in wrappers, or exposed to browser code unless the deployment explicitly accepts that custody boundary.

Production key-custody requirements are detailed in [key-custody-v0.md](key-custody-v0.md).

## Public-Key Discovery

Rava V0 does not discover keys. A production deployment must define public-key discovery as caller policy:

- accepted registries, DID methods, or resolver sources;
- trust roots;
- resolver freshness;
- cache lifetime;
- ambiguity handling;
- outage handling;
- downgrade and rollback behavior.

If an authentic public key cannot be selected, verification must fail closed.

Production public-key discovery requirements are detailed in [key-discovery-v0.md](key-discovery-v0.md).

## Distributed Replay

Local file-backed replay is useful for development and single-node previews. Production deployments need distributed replay coordination when the same signed action can reach more than one verifier:

- atomic action-ID consumption;
- durability before reporting acceptance;
- cross-region consistency expectations;
- retry behavior after partial failure;
- recovery from replay-store outages.

Accepting the same one-time action in two places is an authorization failure.

Production distributed replay requirements are detailed in [distributed-replay-v0.md](distributed-replay-v0.md).

## Distributed Revocation

Production revocation requires publication, freshness, and outage policy:

- revoked signer and capability ID publication;
- freshness target and maximum tolerated staleness;
- cache invalidation;
- verifier outage behavior;
- emergency revocation propagation;
- audit evidence for revocation checks.

If revocation freshness cannot satisfy local policy, verification must fail closed.

Production distributed revocation requirements are detailed in [distributed-revocation-v0.md](distributed-revocation-v0.md).

## Audit Storage

The preview service can append local decision metadata, but production audit storage needs a managed system:

- retention;
- privacy;
- export;
- access control;
- tamper evidence;
- deletion or legal hold policy;
- correlation with receipts, attestations, resolver evidence, and downstream tool/API use.

Audit systems should avoid storing raw action payloads unless the deployment has an explicit data-handling reason and policy.

Production audit-storage requirements are detailed in [audit-storage-v0.md](audit-storage-v0.md).

## Monitoring

Production monitoring should cover verifier availability and rejection patterns:

- accepted/rejected decision rates;
- rejection code distribution;
- missing-key and stale-revocation events;
- replay attempts;
- audit-write failures;
- service latency and saturation;
- dependency outages.

Monitoring must not leak private keys, credentials, or sensitive action payload fields.

Production monitoring requirements are detailed in [monitoring-v0.md](monitoring-v0.md).

## External Security Review

An external security review is required before Rava is represented as a production authorization system. Review scope should include:

- Rust core verifier semantics;
- canonicalization and signature binding;
- replay and revocation freshness;
- key custody and discovery;
- service ingress controls;
- caller identity and distributed rate limiting;
- wrapper/adaptor fail-closed behavior;
- audit and monitoring controls.

Findings and remediation should be tracked in release notes or a dedicated review register.
The V0 review register is [../security/review-register-v0.md](../security/review-register-v0.md).
