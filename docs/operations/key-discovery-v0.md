# Rava Production Public-Key Discovery V0

This runbook defines public-key discovery requirements for production deployments. Public-key discovery is a caller trust-policy layer, not a Rava V0 core guarantee.

The Rava core checks that signatures match supplied public keys and that signer IDs match those keys. It does not decide which registry, DID method, cache, or resolver is authoritative.

## Required Properties

A production discovery policy should define:

- accepted registries, DID methods, resolver sources, or static trust bundles;
- trust roots;
- resolver freshness;
- cache lifetime;
- ambiguity handling;
- outage behavior;
- downgrade and rollback handling;
- audit evidence for key-source decisions.

## Fail-Closed Rule

If the caller cannot select an authentic public key for the actor, issuer, verifier, or evaluator being checked, verification must fail closed.

Using an unauthenticated fallback key can turn signature verification into a false proof of authority.

## Freshness

Resolver freshness should define:

- maximum accepted age for cached keys;
- how revocations or rotations invalidate cached keys;
- whether stale-but-present keys are rejected or allowed under accepted risk;
- how resolver outages affect verification.

## Rollback

Resolver rollback protection should define how operators detect and reject older key states after rotation or compromise response.
