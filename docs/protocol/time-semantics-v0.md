# Rava V0 Time Semantics

Rava V0 verifier time is an explicit caller input. The core verifier does not read wall-clock time by itself during action verification.

## Verifier Time

CLI and service callers pass verifier time as `now_unix`, which is a Unix timestamp in seconds. Library callers pass the equivalent `OffsetDateTime`.

Use one verifier time source for all checks in a verification decision. Do not mix local process time, remote registry time, and client-provided time inside one decision unless a higher-level caller policy explicitly defines that behavior.

## Expiry Rule

A capability is expired when `expires_at <= now`.

This means:

- a capability is valid before its `expires_at` timestamp;
- a capability is not valid exactly at its `expires_at` timestamp;
- child capabilities must expire no later than their parent capability.

Boundary regression coverage verifies that a capability is accepted immediately before `expires_at` and rejected exactly at `expires_at`.

## Clock Skew

Rava V0 does not apply implicit clock skew.

If a deployment needs tolerance for clock drift, the verifier caller must choose and document that policy before invoking the core verifier. The core should receive the final policy-adjusted `now_unix` value or a capability chain whose expiry already reflects that policy.

## Replay and Revocation Freshness

Revocation and replay freshness are caller responsibilities.

The core verifier checks the replay and revocation registries it is given. It does not prove that those registries are globally current, distributed, replicated, or synchronized. A production deployment must define registry freshness, outage behavior, and consistency guarantees outside the V0 core.

## Receipts and Attestations

Verification receipts include the verifier's decision time. Post-action attestations include an occurrence time. These timestamps are signed audit data, but V0 does not prove that an external clock source was correct.

Operators should treat receipt and attestation timestamps as statements by the signing verifier or evaluator, not as independent time oracle proofs.
