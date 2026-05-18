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

## Static Trust Bundles

`rava verify action` supports an explicit local static trust bundle for controlled deployments:

```json
{
  "version": "rava-static-trust-bundle-v0",
  "fresh_until_unix": 1650003600,
  "keys": {
    "rava:agent:<actor-public-key-prefix>": "<actor-public-key-hex>",
    "rava:human:<issuer-public-key-prefix>": "<issuer-public-key-hex>"
  }
}
```

When a trust bundle is provided, the CLI can select the actor and issuer public keys by signer ID. Explicit `--actor-key` or `--issuer-key` values must match any corresponding bundle entry; conflicts fail closed as CLI errors instead of silently choosing one source.

`rava verify action --require-fresh-trust-bundle` requires the local static trust bundle to include `fresh_until_unix` greater than verifier `now_unix`. Missing or stale trust-bundle freshness fails closed before verifier execution.

When a static trust bundle is used, `rava verify action` prints `Rava key source: static-trust-bundle`. This is local CLI key-source evidence for controlled deployments; it is not managed resolver audit evidence, dynamic key discovery, or production resolver policy.

Static trust bundles are the selected first production-trust step because they are inspectable and avoid unaudited network resolver behavior. Local `fresh_until_unix` checking gives operators an explicit cache-lifetime guard for static bundles. It does not provide dynamic DID, web, registry, resolver selection, cache invalidation, rotation, rollback, or outage guarantees.

## Freshness

Resolver freshness should define:

- maximum accepted age for cached keys;
- how revocations or rotations invalidate cached keys;
- whether stale-but-present keys are rejected or allowed under accepted risk;
- how resolver outages affect verification.

## Rollback

Resolver rollback protection should define how operators detect and reject older key states after rotation or compromise response.
