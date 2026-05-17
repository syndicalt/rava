# Rava Production Trust and Operations V0

This document defines production systems that must surround Rava before it is represented as a production authorization system. These are not implemented guarantees in the V0 core protocol, CLI, preview service, WASM wrapper, or TypeScript package.

Rava V0 verifies signed actions and attenuated capability chains against caller-supplied keys and caller-supplied replay/revocation state. Production deployments must own the surrounding trust, freshness, custody, and monitoring systems.

The preview verifier service is not a production ingress boundary. Caller identity requirements are detailed in [caller-identity-v0.md](caller-identity-v0.md), and distributed rate-limit requirements are detailed in [distributed-rate-limits-v0.md](distributed-rate-limits-v0.md).

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
- wrapper/adaptor fail-closed behavior;
- audit and monitoring controls.

Findings and remediation should be tracked in release notes or a dedicated review register.
The V0 review register is [../security/review-register-v0.md](../security/review-register-v0.md).
