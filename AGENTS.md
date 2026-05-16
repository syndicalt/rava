# Rava Agent Instructions

## Standing Context

Rava is a security-sensitive agent authentication and authorization protocol. Treat every implementation decision as part of a future production security boundary unless the user explicitly says it is a throwaway prototype.

## Karpathy Development Rules

Follow the Karpathy-inspired development rules for all Rava work:

1. **Think Before Coding**
   - State assumptions explicitly.
   - Surface ambiguity, tradeoffs, and uncertainty.
   - Ask before guessing when the wrong choice would affect architecture, security, or protocol semantics.
   - Push back when a requested path weakens security or adds unnecessary complexity.

2. **Simplicity First**
   - Build the minimum correct system that satisfies the current goal.
   - Do not add speculative features, generic frameworks, or unnecessary abstractions.
   - Prefer small, inspectable modules over clever designs.

3. **Surgical Changes**
   - Touch only files required by the task.
   - Do not perform drive-by refactors.
   - Do not remove or rewrite code you do not understand.
   - Every changed line must trace to the current request.

4. **Goal-Driven Execution**
   - Convert work into explicit success criteria.
   - Use tests first for behavior changes.
   - Verify with concrete commands before claiming success.

## No Hacks Ever

No hacks, shortcuts, fake implementations, test-only bypasses, insecure defaults, or temporary protocol behavior are acceptable.

If a proper solution is too large for the current step, reduce scope instead of weakening the design. Use small production-quality increments.

## Security Engineering Rules

- Rust is the trusted implementation language for the protocol core.
- Keep `unsafe` forbidden unless the user explicitly approves a narrowly justified exception.
- Do not invent cryptographic primitives.
- Prefer audited, widely used cryptographic crates.
- Treat canonicalization, signing, verification, expiry, revocation, and constraint checks as security boundaries.
- Fail closed on malformed, missing, expired, revoked, or unverifiable data.
- Avoid logging secrets, private keys, raw credentials, or sensitive action payloads.
- Tests must include rejection cases, not only happy paths.
- Documentation must distinguish implemented guarantees from roadmap ideas.

## Learning Mode

The user is learning authentication while building Rava. Explain important auth concepts in plain language as they appear in the code: identity, signatures, capabilities, delegation, revocation, action verification, and auditability.
