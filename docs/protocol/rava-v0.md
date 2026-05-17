# Rava V0 Protocol Draft

Rava V0 is a local reference protocol for action-native agent authorization.

Wire-shape schemas for V0 JSON objects are documented in `docs/schemas/v0`. Schemas only describe object shape. They do not verify signatures, replay state, revocation state, expiry, delegation attenuation, key authenticity, or trust policy.

Interop planning for wrappers and adapters is documented in `docs/interop/roadmap-v0.md`. Interop layers must preserve Rust verifier semantics and must not be described as new V0 guarantees until implemented and tested.

Compatibility policy is documented in `docs/protocol/compatibility-policy-v0.md`. Verifier time and expiry semantics are documented in `docs/protocol/time-semantics-v0.md`. Operator-facing rejection codes are documented in `docs/operators/rejection-codes-v0.md`.

## Identity

A Rava signer is a local Ed25519 keypair plus a protocol-facing signer ID.

Signer IDs currently use this form:

```text
rava:<kind>:<public-key-prefix>
```

Kinds are `human`, `agent`, `service`, and `runtime`.

The private key signs canonical JSON payloads. The public key verifies signatures. The current implementation does not provide custody, recovery, key rotation, DID resolution, or legal identity binding.

Signature verification requires the signer ID kind to be one of `human`, `agent`, `service`, or `runtime`, and requires the signer ID key prefix to match the presented public key. A valid signature from one key is not accepted under another signer ID.

## Canonicalization

All signed payloads are encoded as canonical JSON before signing:

- object keys are sorted recursively;
- arrays preserve order;
- scalar values are serialized with `serde_json`;
- signatures are over the resulting UTF-8 bytes.

Canonicalization matters because signatures verify bytes, not human intent. Two verifiers must produce the same byte string for the same logical payload.

## Signatures

Rava V0 uses Ed25519 signatures through `ed25519-dalek`.

The verifier rejects modified payloads. If an action amount, capability ID, resource, operation, actor, or other signed field changes after signing, verification fails.

Signed protocol object nonces must be canonical lowercase UUID v4 strings. Verifiers reject malformed, non-canonical, or non-v4 nonces.

## Capabilities

A capability is a signed permission object:

```text
Capability {
  version,
  id,
  nonce,
  issuer,
  subject,
  resource,
  operations,
  constraints,
  expires_at,
  delegable,
  parent_id,
  proof
}
```

The issuer signs the canonical unsigned capability payload. Capability IDs are derived from a SHA-256 digest of the canonical pending capability payload.

Capability `operations` must be non-empty, sorted in ascending bytewise string order, and contain no duplicates. Constructors normalize operations before signing. Verifiers reject signed capability objects whose operation list is empty or not canonical.

## Delegation

Delegation creates a child capability from a parent capability. V0 enforces attenuation:

- the parent must be delegable;
- the delegating issuer must be the parent subject;
- child operations must be included in parent operations;
- child expiry must not outlive parent expiry;
- child constraints must preserve every parent constraint key;
- child `max_amount_usd` must not exceed parent `max_amount_usd`;
- child constraints must not introduce broader authority.

V0 constraint values have intentionally small semantics:

- integer child constraints may be equal to or less than the parent integer value;
- text constraints must match exactly;
- boolean constraints must match exactly;
- changing a constraint's value type is treated as broadening and is rejected;
- action `amount_usd` is covered by final capability `max_amount_usd`;
- other action constraints are covered only by exact constraint keys on the final capability, using the same integer-less-than-or-equal or exact text/boolean rules.

Invalid delegation returns typed errors and fails closed.

## Actions

An action is a signed attempt to do something:

```text
Action {
  version,
  id,
  nonce,
  actor,
  controller,
  intent,
  resource,
  operation,
  constraints,
  capability_id,
  context_hash,
  proof
}
```

The actor signs the canonical unsigned action payload.

`context_hash` must be `sha256:` followed by 64 lowercase hexadecimal characters. It is the action's signed reference to off-chain context, such as the full request body or task bundle.

## Replay Defense

Rava V0 includes in-memory and local file-backed replay registries for one-time action verification. `verify_action_once` rejects an action ID that has already been accepted and recorded.

Rejected actions are not recorded as consumed. This means a failed attempt does not burn a newly signed action; only successful authorization consumes the action ID.

Replay recording is fallible. If an accepted action cannot be recorded, one-time verification returns an error instead of claiming the action was safely consumed.

Replay registry contract:

- a replay registry answers whether an action ID has already been consumed inside the caller's one-time-use boundary;
- recording an accepted action ID must be durable before the verifier reports one-time verification success;
- recording failure must return an error instead of an accepted one-time verification result;
- Rejected actions must not be recorded;
- Registry lookup failures must fail closed before verification claims acceptance.

## Verification

The verifier accepts an action only if all checks pass:

- action version is exactly `rava-action-v0`;
- action nonce is a canonical UUID v4;
- action signature is valid;
- action ID has not already been consumed when one-time verification is used;
- capability chain is non-empty;
- root capability issuer equals action controller;
- final capability ID equals action capability ID;
- every capability version is exactly `rava-capability-v0`;
- every capability nonce is a canonical UUID v4;
- every capability signature is valid;
- no capability is revoked;
- action actor signer ID is not revoked;
- capability issuer signer IDs are not revoked;
- no capability is expired;
- parent links are valid;
- each delegated capability issuer equals the parent subject;
- each parent in a delegation step is delegable;
- each child capability resource equals the parent resource;
- each child capability operation is granted by the parent;
- each child capability expiry is no later than the parent expiry;
- each child capability preserves parent constraint keys;
- each child capability constraint is no broader than the parent constraint;
- final capability subject equals action actor;
- resource matches;
- operation is allowed;
- every action constraint is covered by the final capability;
- action `amount_usd` must have a capability `max_amount_usd` and must not exceed it.

Any failed check returns a rejected verification result.

V0 verifier callers must provide authentic public keys for action actors and capability issuers. The core checks that signatures match the supplied keys and that signer IDs match those keys, but it does not discover keys, resolve DIDs, or prove that a caller selected the correct key source.

## Local Verifier Service Preview

The Rust CLI includes a local preview service:

```text
RAVA_VERIFIER_TOKEN=<secret> rava serve verify --addr 127.0.0.1:8787 --max-request-bytes 1048576 --replay-store replay.json --revocation-store revocations.json --audit-log audit.ndjson --auth-token-env RAVA_VERIFIER_TOKEN --rate-limit-per-minute 120
```

It exposes `POST /verify/action` as an HTTP wrapper around the V0 Rust verifier. Request JSON must include:

- `action`;
- `capability_chain`;
- `actor_public_key_hex`;
- `issuer_public_keys`;
- `now_unix`.

It also exposes `GET /healthz`, which returns the service name, `ok` status, and configured `max_request_bytes`.
`--max-request-bytes` limits request bodies before JSON parsing and verification.
`--replay-store` records accepted action IDs in a local file and rejects later replays.
`--revocation-store` loads a local revoked-ID snapshot for signer and capability checks on each request.
`--audit-log` appends newline-delimited JSON decision metadata without raw action intent, resource, constraints, capability envelopes, or signatures.
`--auth-token-env` requires `Authorization: Bearer <token>` on every request and reads the token from an environment variable so it is not exposed in command arguments.
`--rate-limit-per-minute` applies a local per-process request limit.

The response includes:

- `service`;
- `accepted`;
- `rejection`, with stable verifier `code` and optional `subject` when rejected.

The preview service is not a new trust boundary. It does not discover keys, coordinate distributed replay state, distribute revocations, prove revocation freshness, identify callers, provide distributed rate limiting, provide managed audit retention/export, or make verifier trust-policy decisions.

## Verification Receipts

A verification receipt is signed evidence of the verifier's decision. It includes:

- verifier ID;
- action ID;
- actor ID;
- capability ID;
- capability chain hash;
- context hash;
- accepted or rejected decision;
- stable rejection reason code, when rejected;
- verification timestamp.

The capability chain hash is `sha256:` followed by 64 lowercase hexadecimal characters. It is computed over the canonical JSON array of signed capability envelopes supplied to the verifier.

The receipt intentionally omits raw action intent, resource, operation, constraints, and capability envelopes. It proves a decision happened without making the receipt a copy of the sensitive action payload.

Receipt verification rejects any version other than `rava-verification-receipt-v0` and any nonce that is not a canonical UUID v4.

Receipt verifiers must provide an authentic verifier public key. V0 verifies the receipt signature and signed fields against that key; it does not implement verifier discovery or external trust policy.

## Revocation

V0 includes in-memory and local file-backed revocation registries. Revocation is checked for:

- capability IDs;
- action actor signer IDs;
- capability issuer signer IDs.

The file-backed registry stores revoked IDs in deterministic JSON and rejects invalid JSON rather than silently ignoring corrupt state.

Future versions should define network distribution, freshness, and consistency rules for shared revocation state.

V0 assumes callers provide a revocation registry that is sufficiently fresh for their risk tolerance. Stale registry input is treated as caller-provided state, not as a freshness guarantee made by the core verifier.

Revocation registry contract:

- a revocation registry answers whether a signer or capability ID is revoked in the caller-provided snapshot;
- Registry lookup failures must fail closed before verification claims acceptance;
- file-backed registry parse failures must fail closed;
- Freshness and distribution are caller responsibilities in V0;
- V0 local file registries are reference implementations, not a distributed revocation protocol.

## Attestations

Attestations are post-action signed evidence:

- action reference;
- outcome;
- evaluator;
- timestamp;
- evidence hash.

`evidence_hash` must be `sha256:` followed by 64 lowercase hexadecimal characters.

V0 supports signing and verifying attestations. It does not yet define reputation indexes, dispute workflows, evaluator trust policy, or selective disclosure.

Attestations are the future substrate for scoped reputation, audit, compliance, and insurance workflows.

Attestation verification rejects any version other than `rava-attestation-v0` and any nonce that is not a canonical UUID v4.

Attestation verifiers must provide an authentic evaluator public key. V0 verifies signed attestation fields against that key; it does not decide whether the evaluator is trusted for a domain.

## Non-Goals

Rava V0 does not implement:

- global identity registry;
- production key custody;
- account recovery;
- OAuth replacement;
- blockchain anchoring;
- zero-knowledge reputation;
- legal liability assignment;
- formal model-behavior proofs.
