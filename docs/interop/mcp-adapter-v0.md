# Rava MCP Adapter V0

This proof of concept describes how an MCP server can use Rava as an authorization check before tool execution. It is integration guidance, not a new core protocol guarantee.

The adapter rule is verify before tool execution and deny by default. If verification is missing, malformed, rejected, stale, or unavailable, the adapter must not execute the tool.

## Tool Request Shape

An MCP tool request can carry Rava authorization material alongside normal tool arguments:

```json
{
  "tool": "book_flight",
  "arguments": {
    "route": "ORD-SFO"
  },
  "rava": {
    "action": {},
    "capability_chain": [],
    "actor_public_key_hex": "hex-encoded-actor-public-key",
    "issuer_public_keys": {},
    "now_unix": 1760000000
  }
}
```

The `rava` object is passed to the Rust verifier, preview service, WASM wrapper, or TypeScript `verifyAction` wrapper. Tool arguments are not authorization by themselves.

## Adapter Pseudocode

```ts
const decision = verifyAction(request.rava);

if (decision.accepted !== true) {
  throw new Error(decision.rejection?.code ?? "rava_verification_required");
}

return executeTool(request.tool, request.arguments);
```

The adapter must preserve the verifier rejection code and subject for auditability. It must not replace capability attenuation with broad tool grants, local allowlists, or OAuth scopes.

## Trust Inputs

The MCP adapter must define how it obtains:

- authentic actor public keys;
- authentic issuer public keys;
- a sufficiently fresh capability chain;
- replay state when one-time action use is required;
- revocation freshness;
- audit or receipt emission policy.

Those inputs are caller policy. They are not supplied by MCP and are not discovered by `rava-core`.
