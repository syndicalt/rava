# Rava OAuth Exchange V0

OAuth exchange is integration glue. Rava verification happens before token exchange for agent actions, OAuth scopes do not prove delegation-chain attenuation, and unavailable verification must fail closed.

The intended ordering is:

1. Receive a signed Rava action and capability chain.
2. Resolve or receive authentic public keys.
3. Verify the Rava action.
4. Fail closed if the verifier rejects or verification inputs are unavailable.
5. Request or use an OAuth token only for the verified action context.
6. Store any receipt, attestation, or audit evidence required by local policy.

## Binding Token Use

Where possible, OAuth token use should be bound to the verified action context:

- resource or account;
- operation;
- amount or other constraints;
- actor;
- action ID;
- expiry or freshness window.

OAuth scopes are often broader than a single delegated action. They must not be treated as a substitute for Rava attenuation checks.

## Token Custody

Integrations must document token custody:

- which component requests the token;
- where the token is stored;
- how long it is retained;
- how compromise or revocation is handled;
- whether token use is logged with the Rava action ID.

Rava is not an OAuth replacement. It can authorize an agent action before OAuth access is used, but OAuth trust, custody, refresh, and provider policy remain external responsibilities.
