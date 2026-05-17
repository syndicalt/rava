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

## Emergency Revocation

Emergency revocation should define:

- who can publish emergency revocations;
- how verifiers learn about them;
- maximum propagation time;
- how caches are invalidated;
- how operators confirm enforcement.

## Outage Behavior

Outage policy should state whether verification stops, serves only low-risk actions, or continues under accepted risk when revocation infrastructure is unavailable.
