# Rava Product Direction

## Thesis

Rava is an agent authentication and authorization protocol built around signed, constrained actions rather than broad agent accounts. Modern auth retrofits human and machine-to-machine patterns onto agents. Rava starts from the opposite premise: autonomous agents need a native way to prove what they are allowed to do, how that authority was delegated, under what constraints, and what happened afterward.

The atomic unit of Rava is not the login session. It is the verifiable action.

## Problem

OAuth scopes, API keys, service accounts, short-lived JWTs, mTLS, and MCP-style tool grants can work for bounded integrations, but they do not fully address autonomous, long-lived, multi-hop agents. The main failures are:

- Permissions are too broad for complex delegated work.
- Delegation chains are hard to inspect, attenuate, and revoke.
- Audit logs are separate from authorization decisions.
- Agent identity is usually just "user plus app" or "service account".
- Reputation is either absent, centralized, or globally overbroad.
- Cross-domain agents cannot carry portable proof of authority.

Rava should make it normal for a service to ask: "Can this actor perform this specific action right now, under these constraints, through this delegation chain?"

## Product Definition

Rava is a protocol, SDK, and verification runtime for action-native agent authorization.

The first product surface should be a developer toolkit that can:

- Create agent and controller identities.
- Mint constrained capabilities.
- Delegate capabilities with attenuation.
- Sign action invocations.
- Verify action invocations against capability chains.
- Revoke capabilities and keys.
- Emit attestations that can become scoped reputation inputs.

Rava should not start as a full identity provider, blockchain, wallet, marketplace, or reputation oracle. Those can emerge later from the action layer.

Rava's trusted core should be written in Rust. The protocol should be revolutionary, but its implementation posture should be conservative: memory-safe systems language, strict types, small dependency surface, audited cryptographic crates, deterministic encodings, and testable verification rules. TypeScript should be treated as a later integration layer for agent ecosystems, not the source of truth for protocol correctness.

## Licensing Model

Rava should use an open-core infrastructure model:

- The protocol specification should be open and permissively licensed.
- The Rust reference core should be open source so implementers can inspect, verify, embed, and audit the security-critical logic.
- The CLI, examples, and local verifier should be open source to drive adoption and interoperability.
- Commercial value should be built above the open core: hosted verification, managed revocation, enterprise policy engines, compliance/audit tooling, key custody integrations, insurance/bonding workflows, support, and certification.

This keeps the trust foundation inspectable while still leaving room for a serious business. A closed core would weaken adoption for a protocol that needs cross-domain trust. The default license for implementation artifacts is Apache-2.0 unless changed deliberately.

## Core Concept

Every meaningful agent action is represented by a signed action envelope:

```json
{
  "version": "rava-action-v1",
  "action_id": "act_...",
  "actor": "did:key:...",
  "controller": "did:key:...",
  "intent": "book_flight",
  "capability": {
    "capability_id": "cap_...",
    "resource": "travel.booking",
    "operations": ["purchase"],
    "constraints": {
      "max_amount_usd": 1200,
      "merchant_category": "airline",
      "expires_at": "2026-05-17T00:00:00Z"
    }
  },
  "delegation_chain": ["cap_root", "cap_search_only", "cap_purchase_exact"],
  "context": {
    "session": "sess_...",
    "request_hash": "sha256:..."
  },
  "proof": {
    "type": "Ed25519Signature2020",
    "created": "2026-05-16T00:00:00Z",
    "verification_method": "did:key:...#key-1",
    "signature": "..."
  }
}
```

The envelope gives services enough structure to verify the actor, authority, delegation path, constraints, expiry, and signature without trusting a central platform on every request.

## Principles

- **Action-first:** Authorization decisions attach to concrete actions, not vague agent trust.
- **Least authority:** Every capability is narrow by default and can be attenuated during delegation.
- **Portable identity:** Agents, users, organizations, and runtimes can hold stable signing keys.
- **Delegation-native:** Human to agent to sub-agent chains are first-class.
- **Revocable:** Capabilities, delegation edges, and keys can be revoked.
- **Auditable:** Action envelopes and attestations form a verifiable trail.
- **Scoped reputation:** Reputation is derived from action history for a specific domain, not a universal trust score.
- **Privacy-preserving:** Services should verify necessary authority without requiring full behavioral history.
- **Interop-first:** Rava should integrate with OAuth, DID/VC ecosystems, MCP servers, and existing service APIs rather than require a clean-slate internet.

## Urbit-Inspired Direction

Urbit is relevant because it treats identity as a network substrate instead of an application account. Rava should adopt the substrate mindset without copying Urbit's exact scarcity or social hierarchy model.

The Rava equivalent is an action substrate:

- Stable signers provide continuity.
- Signed actions provide behavioral history.
- Capability chains define authority.
- Attestations become evidence for scoped reputation.
- Higher-order governance emerges above the protocol.

Identity and reputation should flow from verifiable action continuity. They should not be required as centralized prerequisites for participation.

## Initial Architecture

Rava should begin with four layers:

### 1. Identity Layer

Responsible for key material, identifiers, key rotation, and controller relationships.

Initial recommendation:

- Use DID-compatible identifiers where practical.
- Support local Ed25519 keys in the Rust core.
- Treat humans, organizations, agents, and runtimes as signers with explicit relationships.

### 2. Capability Layer

Responsible for minting, attenuation, expiry, delegation, and revocation.

Capabilities should include:

- Issuer
- Subject
- Resource
- Operations
- Constraints
- Expiry
- Delegability
- Parent capability
- Signature

### 3. Action Layer

Responsible for signing and verifying concrete invocations.

Actions should include:

- Actor
- Controller or beneficiary
- Intent
- Capability reference
- Delegation chain
- Context hash
- Execution constraints
- Signature

### 4. Attestation Layer

Responsible for post-action evidence.

Attestations should include:

- Action reference
- Outcome
- Evaluator
- Timestamp
- Optional cost, damage, refund, approval, or dispute status
- Signature

This layer is the seed of future reputation, insurance, compliance, and dispute systems.

## Initial Product Scope

The first milestone should prove that Rava can safely authorize multi-hop agent action.

Scenario:

1. A human controller grants a personal agent permission to book a flight under 1200 USD.
2. The personal agent delegates search-only capability to a travel-search agent.
3. The personal agent delegates exact-purchase capability to a booking agent.
4. A verifier accepts the booking action only if the capability chain is valid, unexpired, unrevoked, and within constraints.
5. The booking service emits an attestation.

This scenario is narrow enough to build, but expressive enough to validate the core thesis.

## Non-Goals for V1

- No global reputation marketplace.
- No blockchain requirement.
- No production custody product.
- No formal verification of model behavior.
- No universal agent registry.
- No broad OAuth replacement.
- No custom cryptographic primitives.

## Product Roadmap

### V0: Protocol Draft and Local Verifier

Deliver a written protocol draft and a Rust reference implementation that can create, delegate, sign, revoke, and verify capabilities and actions, plus sign post-action attestations.

### V1: Developer Preview

Deliver a Rust CLI, examples, and a verifier service that can sit in front of agent-native APIs.

### V2: Interop Layer

Add WASM and TypeScript bindings, plus adapters for MCP tools, OAuth token exchange, DID documents, and common service APIs.

### V3: Attestation and Reputation Substrate

Add signed outcome attestations, scoped reputation indexes, and selective disclosure proofs where useful.

### V4: Governance and Risk Controls

Add human approval policies, threshold signatures, insurance hooks, economic bonding, and dispute workflows.

## Key Open Decisions

- Exact canonicalization/signature format.
- Whether identifiers start with `did:key`, internal `rava:` URIs, or both.
- Where revocation state lives in V1.
- How much compatibility to maintain with W3C VC, UCAN, macaroons, GNAP, OAuth, and MCP.

## Recommended Next Move

Build a V0 reference implementation around one scenario: delegated flight booking. Avoid solving global identity or reputation until the action/capability layer is working and testable.
