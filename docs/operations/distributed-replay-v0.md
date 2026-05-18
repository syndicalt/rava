# Rava Production Distributed Replay V0

This runbook defines distributed replay requirements for production deployments. Distributed replay coordination is not implemented by the V0 preview service.

The Rust core provides a replay registry trait and local development implementations. Production deployments need a shared one-time-use boundary when the same signed action can reach more than one verifier.

The verifier contract now calls an atomic consume operation after an action is otherwise accepted. A replay backend must report duplicate consumption in that operation instead of relying on a separate stale pre-check.

The local file-backed replay registry serializes consume operations with a lock file and reloads the replay store while holding that lock. This prevents stale local handles in the same filesystem boundary from both consuming the same action ID. Lock acquisition, read, write, or rename failure is a replay-store error and fails closed before one-time verification reports acceptance.

This local hardening is useful for controlled single-host or shared-filesystem tests. It is not a distributed replay system, cross-node lock service, cross-region consistency model, outage policy, or managed replay audit trail.

The preview service also supports `--require-replay-store`, which fails closed at startup unless `--replay-store` is configured. That guardrail helps controlled preview deployments avoid accidentally running without local one-time-use enforcement, but it does not add a shared replay backend, distributed locking, durability guarantees, cross-node consistency, outage policy, or audit evidence.

## Required Properties

A production replay system should provide:

- atomic action-ID consumption;
- durability before acceptance;
- consistency expectations across regions or replicas;
- retry behavior after partial failure;
- outage behavior;
- audit evidence for consumed action IDs;
- monitoring for replay attempts and store errors.

## Acceptance Rule

An action should be reported as accepted only after the action ID is durably consumed inside the deployment's one-time-use boundary.

If durable consumption cannot be confirmed, verification should fail closed rather than risk accepting the same signed action in multiple places.

If the replay backend reports that another verifier already consumed the action ID during the consume operation, the verifier reports `action_replayed` even if an earlier local view would have considered the action unseen.

## Partial Failure

The deployment should define behavior for:

- verifier timeout after replay-store write;
- replay-store timeout before write confirmation;
- duplicate requests racing across nodes;
- cross-region replication delay;
- recovery after replay-store outage.

## Cross-Region Deployments

Cross-region deployments should state whether replay consumption is globally serializable, region-local, or explicitly scoped to a smaller boundary.

Accepting the same one-time action in two places is an authorization failure unless the deployment intentionally scopes one-time use more narrowly and documents that limitation.
