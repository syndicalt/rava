# Rava DID and Key Resolution V0

DID and key resolution are a caller trust-policy layer. Rava V0 verifies signatures against public keys supplied by the caller; it does not decide which DID method, resolver, cache, trust root, or freshness window is authoritative.

Resolvers must resolve before invoking the Rust verifier, WASM wrapper, TypeScript wrapper, or preview service. If resolution is unavailable, ambiguous, stale, or inconsistent with local policy, the integration should fail closed and skip action execution.

## Example Resolution Envelope

An integration can keep resolver output adjacent to verifier input without making it part of the signed Rava object:

```json
{
  "resolver": "example-did-resolver",
  "resolver_freshness_unix": 1760000000,
  "actor_public_key_hex": "hex-encoded-actor-public-key",
  "issuer_public_keys": {
    "human_example": "hex-encoded-root-issuer-public-key",
    "agent_example": "hex-encoded-delegating-agent-public-key"
  }
}
```

The verifier receives `actor_public_key_hex` and `issuer_public_keys`. The resolver metadata is caller policy evidence, not a core verifier guarantee.

## Caller Responsibilities

Callers must define:

- accepted DID methods or key registries;
- trust roots;
- cache lifetime and resolver freshness policy;
- outage behavior;
- key-rotation and compromise behavior;
- whether resolver evidence is stored in local audit output.

These responsibilities are not implemented in `rava-core`. Adding DID resolution to an adapter must not weaken fail-closed verification or turn missing keys into acceptance.
