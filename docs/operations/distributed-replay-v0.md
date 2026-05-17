# Rava Production Distributed Replay V0

This runbook defines distributed replay requirements for production deployments. Distributed replay coordination is not implemented by the V0 preview service.

The Rust core provides a replay registry trait and local development implementations. Production deployments need a shared one-time-use boundary when the same signed action can reach more than one verifier.

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
