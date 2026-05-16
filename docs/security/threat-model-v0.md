# Rava V0 Threat Model

Rava V0 is a local reference implementation for action-native agent authorization. This document describes the security boundary that is implemented today, the assumptions verifier callers must satisfy, and the risks that are explicitly outside V0 scope.

## Assets

Rava V0 treats the following as security-sensitive assets:

- signer private keys;
- signed action envelopes;
- signed capabilities and delegated capability chains;
- signed verification receipts;
- signed attestations;
- replay registry state for accepted action IDs;
- revocation registry state for revoked signer and capability IDs.

Compromise or substitution of any asset can change authorization outcomes, audit evidence, or post-action accountability.

## Trusted Computing Base

The V0 trusted computing base is intentionally small:

- the Rust protocol core in `rava-core`;
- deterministic canonical JSON encoding for signed payloads;
- Ed25519 signing and verification through existing cryptographic crates;
- SHA-256 object IDs and content references;
- verifier inputs supplied by the caller, including authentic public keys;
- replay and revocation registry trait implementations supplied to the verifier.

The core forbids `unsafe` code. Rava does not invent new cryptographic primitives.

## Attacker Capabilities

V0 assumes an attacker can:

- submit arbitrary signed or unsigned wire objects;
- tamper with any unsigned transport wrapper around protocol objects;
- reorder, truncate, or splice capability chains;
- replay previously accepted signed actions;
- recompute object IDs for fields they can sign;
- validly sign child capabilities for keys they control;
- substitute receipts or attestations;
- withhold, delay, or omit fresh revocation state before it reaches a verifier.

The verifier must fail closed for malformed, unverifiable, expired, revoked, replayed, or over-scoped inputs it can observe.

## Implemented Guarantees

Rava V0 currently enforces these guarantees in the Rust core:

- signed protocol object versions must match the V0 version strings;
- signed protocol object nonces must be canonical UUID v4 strings;
- action and capability IDs must match their canonical signed contents;
- action, capability, receipt, and attestation signatures must verify against the presented public keys;
- the root capability issuer must match the action controller;
- the final capability must be the action capability;
- delegation chains must preserve parent links;
- delegated capability issuers must be the parent subjects;
- delegated capabilities cannot broaden parent authority by resource, operation, expiry, constraints, or non-delegability;
- final capability subject, resource, operation, and constraints must cover the signed action;
- expired, revoked, or malformed capabilities are rejected;
- revoked action actors and capability issuers are rejected;
- one-time verification records only accepted action IDs and rejects accepted-action replay;
- receipt verification rejects malformed nonces and malformed capability chain hashes;
- receipt signatures bind the verifier decision to the action ID, actor, capability ID, context hash, and capability chain hash;
- attestation verification rejects malformed nonces and malformed evidence hashes.

## V0 Assumptions

Verifier callers are responsible for inputs that V0 does not authenticate by itself:

- the public key supplied for an action actor is authentic for the actor being evaluated;
- each capability issuer public key map entry is authentic for that issuer;
- verifier public keys used to check receipts are authentic;
- evaluator public keys used to check attestations are authentic;
- replay registries are shared widely enough for the caller's one-time-use boundary;
- revocation registries are sufficiently fresh for the caller's risk tolerance;
- off-chain payloads referenced by `context_hash` and `evidence_hash` are fetched, retained, and interpreted by caller policy.

These are assumptions, not implemented distributed guarantees.

## Non-Goals

Rava V0 does not implement:

- key custody, backup, recovery, or rotation;
- distributed revocation freshness, consistency, or gossip;
- DID resolution or global identity discovery;
- reputation scoring or trust marketplace behavior;
- blockchain anchoring;
- legal identity binding or liability assignment;
- model-behavior proofs;
- proof that an off-chain service performed an action correctly after authorization.

Future protocol versions may narrow some of these assumptions, but V0 documentation and tests must not describe roadmap ideas as implemented guarantees.
