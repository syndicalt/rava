# Rava Interop Roadmap

This roadmap describes wrapper and adapter work that can sit around Rava V0. It does not add implemented protocol guarantees by itself.

## Core Rule

Rust verifier remains the trusted implementation. WASM, TypeScript, service, DID, MCP, and OAuth integration work must call into the Rust verifier or preserve its exact semantics.

No wrapper may weaken fail-closed verifier behavior. Malformed, unsigned, unverifiable, expired, revoked, replayed, or over-scoped inputs must remain rejected.

## WASM and TypeScript

WASM and TypeScript wrappers must not reimplement verification logic. Their job is to expose the Rust verifier to browser, Node, and agent-tooling environments.

The current WASM wrapper is documented in [wasm-v0.md](wasm-v0.md).
The current TypeScript package is documented in [typescript-v0.md](typescript-v0.md).
DID/key-resolution caller-policy guidance is documented in [did-resolution-v0.md](did-resolution-v0.md).

Expected wrapper responsibilities:

- marshal JSON wire objects into Rust verifier inputs;
- return accepted/rejected results with stable rejection codes and subjects;
- expose canonical test vectors as compatibility tests;
- avoid private-key logging and avoid permissive defaults.

Non-goals for wrappers:

- no independent cryptographic implementation;
- no alternate canonicalization;
- no local policy shortcuts that turn verifier rejection into acceptance.

## DID and Key Resolution

DID/key resolution is a caller trust-policy layer. Rava V0 verifies signatures against public keys supplied by the caller; it does not decide which DID method, resolver, cache, or trust root is authoritative.

Future DID work should:

- resolve keys before invoking the Rust verifier;
- preserve the signer-ID/public-key binding checks already in core;
- document resolver freshness and cache policy separately from verifier guarantees;
- fail closed when resolution is unavailable or ambiguous.

## MCP Adapters

MCP adapters pass signed action envelopes to the verifier. An MCP server integration should treat Rava verification as an authorization check before tool execution.

Expected adapter flow:

1. Receive a tool request and attached Rava action envelope.
2. Resolve or receive the capability chain and public keys according to caller policy.
3. Invoke the Rust verifier or verifier service.
4. Execute the tool only on acceptance.
5. Emit a signed receipt or attestation when configured.

MCP adapters must not replace capability attenuation with broad tool grants.

## OAuth Exchange

OAuth exchange is integration glue, not a replacement for Rava capabilities. OAuth may be useful for acquiring service-specific API access after Rava authorizes a signed action, but OAuth scopes do not prove delegation-chain attenuation by themselves.

Future OAuth work should:

- keep Rava action verification before token exchange for agent actions;
- bind token use to the verified action context where possible;
- document which party is responsible for token custody;
- avoid claiming Rava is an OAuth replacement.

## Sequencing

1. Keep Rust core and CLI test vectors stable.
2. Keep WASM bindings around the Rust verifier compiling against `wasm32-unknown-unknown`.
3. Keep the TypeScript package calling WASM and running the V0 test vectors.
4. Keep DID/key-resolution examples as caller policy, not core trust.
5. Add MCP adapter proof of concept.
6. Add OAuth exchange examples only after the verifier wrapper is stable.

Each integration must run the language-neutral V0 test vectors and must document any trust inputs it expects from callers.
