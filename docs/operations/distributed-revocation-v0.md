# Rava Production Distributed Revocation V0

This runbook defines distributed revocation requirements for production deployments. Distributed revocation freshness is not implemented by the V0 core.

The Rava verifier checks the revocation registry it is given. It does not prove that the registry is current, replicated, authoritative, or complete.

## Required Properties

A production revocation system should define:

- revoked signer and capability ID publication;
- freshness target;
- maximum tolerated staleness;
- emergency revocation propagation;
- cache invalidation;
- outage behavior;
- audit evidence for revocation checks;
- operator monitoring for stale or failed revocation reads.

## Fail-Closed Rule

If revocation freshness cannot satisfy local policy, verification should fail closed.

Allowing verification to continue with stale or missing revocation state is an accepted deployment risk, not a Rava core guarantee.

## Local Snapshot Freshness

Local revocation snapshots may include `fresh_until_unix`. When `rava verify action --require-fresh-revocations` is used with `--revocation-store`, verification fails closed before verifier execution unless `fresh_until_unix` is present and greater than verifier `now_unix`.

This is a local snapshot guardrail for controlled deployments. It does not provide revocation publication, network distribution, cache invalidation, emergency propagation, cross-node freshness, or outage handling.

`rava key revoke --id <signer-id> --revocation-store <path>` updates local file-backed snapshots under a lock file and reloads existing file state before persisting. This prevents stale local handles in the same filesystem boundary from losing earlier revoked IDs. Lock acquisition, read, write, or rename failure is a local revocation-store error and fails closed.

This local update hardening is not a revocation distribution system, emergency propagation channel, cross-node consistency model, outage policy, or managed audit trail.

## Emergency Revocation

Emergency revocation should define:

- who can publish emergency revocations;
- how verifiers learn about them;
- maximum propagation time;
- how caches are invalidated;
- how operators confirm enforcement.

## Outage Behavior

Outage policy should state whether verification stops, serves only low-risk actions, or continues under accepted risk when revocation infrastructure is unavailable.
