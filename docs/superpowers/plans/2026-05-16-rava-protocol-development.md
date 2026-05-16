# Rava Protocol Development Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build Rava V0 as a Rust reference protocol for signed, constrained, delegable agent actions.

**Architecture:** Put all trusted protocol logic in a small Rust core crate. The core owns canonicalization, signing, capability attenuation, revocation checks, action verification, and attestations. A Rust CLI demonstrates the delegated flight-booking scenario; TypeScript/WebAssembly bindings come later as wrappers around the Rust verifier.

**Tech Stack:** Rust workspace, `serde`, `serde_json`, `ed25519-dalek`, `rand_core`, `sha2`, `hex`, `thiserror`, `time`, `uuid`, `clap`, `assert_cmd`, `predicates`, `wasm-bindgen` later.

---

## Security Posture

Rava's core is Rust because the protocol is security-sensitive. We still need humility: Rust does not make cryptographic protocols correct by itself. The implementation must keep the trusted surface small, use audited crates, avoid custom cryptography, test failure cases first, and treat deterministic encoding as part of the security boundary.

Development rule: no hacks ever. Do not use temporary bypasses, fake verifiers, permissive defaults, test-only security shortcuts, or partial protocol behavior that accepts invalid data. If a correct solution is too large, cut scope and still build the smaller piece to production quality.

Rava also follows the Karpathy-inspired rules captured in `AGENTS.md`: think before coding, simplicity first, surgical changes, and goal-driven execution with concrete verification.

Plain-language model:

- **Identity:** who can sign.
- **Capability:** what that signer is allowed to do.
- **Delegation:** how authority is narrowed from one signer to another.
- **Action:** what an agent is trying to do now.
- **Verifier:** the judge that checks signatures, constraints, expiry, revocation, and delegation path.

## File Structure

- `Cargo.toml`: Rust workspace definition.
- `crates/rava-core/Cargo.toml`: Core protocol crate.
- `crates/rava-core/src/lib.rs`: Public exports.
- `crates/rava-core/src/error.rs`: Typed errors.
- `crates/rava-core/src/canonical.rs`: Deterministic canonical JSON.
- `crates/rava-core/src/identity.rs`: Signer IDs, keypairs, signatures, verification.
- `crates/rava-core/src/capability.rs`: Capability schema, minting, delegation, attenuation checks.
- `crates/rava-core/src/action.rs`: Action envelope schema and signing.
- `crates/rava-core/src/revocation.rs`: Revocation registry trait and in-memory implementation.
- `crates/rava-core/src/verifier.rs`: End-to-end action verification.
- `crates/rava-core/src/attestation.rs`: Post-action outcome attestations.
- `crates/rava-cli/Cargo.toml`: CLI package.
- `crates/rava-cli/src/main.rs`: CLI entrypoint.
- `crates/rava-cli/tests/flight_booking.rs`: CLI integration test.
- `docs/protocol/rava-v0.md`: Protocol draft.
- `README.md`: Product overview and quickstart.

## Milestone 1: Rust Workspace Foundation

### Task 1: Create Workspace and Empty Crates

**Files:**
- Create: `Cargo.toml`
- Create: `crates/rava-core/Cargo.toml`
- Create: `crates/rava-core/src/lib.rs`
- Create: `crates/rava-cli/Cargo.toml`
- Create: `crates/rava-cli/src/main.rs`

- [ ] **Step 1: Create workspace manifest**

Create `Cargo.toml`:

```toml
[workspace]
members = ["crates/rava-core", "crates/rava-cli"]
resolver = "2"

[workspace.package]
edition = "2021"
license = "Apache-2.0"
repository = "https://example.invalid/rava"

[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
```

- [ ] **Step 2: Create core crate manifest**

Create `crates/rava-core/Cargo.toml`:

```toml
[package]
name = "rava-core"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
ed25519-dalek = { version = "2.1", features = ["rand_core", "serde"] }
hex = "0.4"
rand_core = { version = "0.6", features = ["getrandom"] }
serde = { version = "1", features = ["derive"] }
serde_json = { version = "1", features = ["preserve_order"] }
sha2 = "0.10"
thiserror = "1"
time = { version = "0.3", features = ["formatting", "parsing", "serde"] }
uuid = { version = "1", features = ["v4", "serde"] }

[dev-dependencies]
pretty_assertions = "1"

[lints]
workspace = true
```

- [ ] **Step 3: Create CLI crate manifest**

Create `crates/rava-cli/Cargo.toml`:

```toml
[package]
name = "rava"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true

[[bin]]
name = "rava"
path = "src/main.rs"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
rava-core = { path = "../rava-core" }

[dev-dependencies]
assert_cmd = "2"
predicates = "3"

[lints]
workspace = true
```

- [ ] **Step 4: Create empty core export**

Create `crates/rava-core/src/lib.rs`:

```rust
#![forbid(unsafe_code)]

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

- [ ] **Step 5: Create minimal CLI**

Create `crates/rava-cli/src/main.rs`:

```rust
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "rava")]
#[command(about = "Action-native authorization for autonomous agents")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Version,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Version => println!("{}", rava_core::version()),
    }
}
```

- [ ] **Step 6: Verify workspace**

Run: `cargo test --workspace`

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml crates
git commit -m "chore: create rust workspace"
```

## Milestone 2: Canonicalization

### Task 2: Deterministic JSON Encoding

**Why this matters:** A signature is over bytes, not over vague meaning. If two machines serialize the same action differently, the signature breaks. Canonicalization gives every signer and verifier the same byte string.

**Files:**
- Create: `crates/rava-core/src/canonical.rs`
- Modify: `crates/rava-core/src/lib.rs`

- [ ] **Step 1: Write failing canonicalization test**

Create `crates/rava-core/src/canonical.rs`:

```rust
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_json_sorts_object_keys() {
        let value = json!({ "b": 2, "a": 1 });

        let encoded = canonical_json(&value).unwrap();

        assert_eq!(encoded, r#"{"a":1,"b":2}"#);
    }
}
```

Modify `crates/rava-core/src/lib.rs`:

```rust
#![forbid(unsafe_code)]

pub mod canonical;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

- [ ] **Step 2: Run failing test**

Run: `cargo test -p rava-core canonical_json_sorts_object_keys`

Expected: FAIL because `canonical_json` is not defined.

- [ ] **Step 3: Implement canonicalization**

Replace `crates/rava-core/src/canonical.rs` with:

```rust
use serde_json::{Map, Value};

#[derive(Debug, thiserror::Error)]
pub enum CanonicalError {
    #[error("JSON serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub fn canonical_json(value: &Value) -> Result<String, CanonicalError> {
    let sorted = sort_json(value);
    Ok(serde_json::to_string(&sorted)?)
}

fn sort_json(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(sort_json).collect()),
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();

            for key in keys {
                if let Some(item) = map.get(key) {
                    sorted.insert(key.clone(), sort_json(item));
                }
            }

            Value::Object(sorted)
        }
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonical_json_sorts_object_keys() {
        let value = json!({ "b": 2, "a": 1 });

        let encoded = canonical_json(&value).unwrap();

        assert_eq!(encoded, r#"{"a":1,"b":2}"#);
    }

    #[test]
    fn canonical_json_sorts_nested_object_keys() {
        let value = json!({
            "outer": { "z": true, "a": false },
            "list": [{ "b": 2, "a": 1 }]
        });

        let encoded = canonical_json(&value).unwrap();

        assert_eq!(
            encoded,
            r#"{"list":[{"a":1,"b":2}],"outer":{"a":false,"z":true}}"#
        );
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p rava-core canonical_json`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/rava-core/src/lib.rs crates/rava-core/src/canonical.rs
git commit -m "feat: add canonical json encoding"
```

## Milestone 3: Identity and Signatures

### Task 3: Sign and Verify Payloads

**Why this matters:** Authentication starts with proving control of a private key. In Rava, a signer can be a human, agent, service, or runtime. The verifier checks that the matching public key produced the signature over the exact canonical payload.

**Files:**
- Create: `crates/rava-core/src/error.rs`
- Create: `crates/rava-core/src/identity.rs`
- Modify: `crates/rava-core/src/lib.rs`

- [ ] **Step 1: Write failing signing test**

Create `crates/rava-core/src/identity.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn signer_signs_and_verifies_payload() {
        let signer = Signer::generate(SignerKind::Human);
        let payload = json!({ "intent": "book_flight", "amount_usd": 500 });

        let signature = signer.sign_json(&payload).unwrap();

        assert!(verify_json_signature(
            &signer.id,
            &signer.public_key_hex,
            &payload,
            &signature
        )
        .unwrap());
    }
}
```

Modify `crates/rava-core/src/lib.rs`:

```rust
#![forbid(unsafe_code)]

pub mod canonical;
pub mod error;
pub mod identity;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

- [ ] **Step 2: Run failing test**

Run: `cargo test -p rava-core signer_signs_and_verifies_payload`

Expected: FAIL because signer types and functions are missing.

- [ ] **Step 3: Add shared error type**

Create `crates/rava-core/src/error.rs`:

```rust
use crate::canonical::CanonicalError;

#[derive(Debug, thiserror::Error)]
pub enum RavaError {
    #[error(transparent)]
    Canonical(#[from] CanonicalError),

    #[error("invalid hex: {0}")]
    Hex(#[from] hex::FromHexError),

    #[error("invalid signature")]
    InvalidSignature,

    #[error("invalid public key")]
    InvalidPublicKey,
}
```

- [ ] **Step 4: Implement signer identity**

Replace `crates/rava-core/src/identity.rs` with:

```rust
use ed25519_dalek::{Signature, Signer as DalekSigner, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::canonical::canonical_json;
use crate::error::RavaError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignerKind {
    Human,
    Agent,
    Service,
    Runtime,
}

#[derive(Debug, Clone)]
pub struct Signer {
    signing_key: SigningKey,
    pub id: String,
    pub kind: SignerKind,
    pub public_key_hex: String,
}

impl Signer {
    pub fn generate(kind: SignerKind) -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let public_key = signing_key.verifying_key();
        let public_key_hex = hex::encode(public_key.as_bytes());
        let kind_label = match kind {
            SignerKind::Human => "human",
            SignerKind::Agent => "agent",
            SignerKind::Service => "service",
            SignerKind::Runtime => "runtime",
        };
        let id = format!("rava:{kind_label}:{}", &public_key_hex[..32]);

        Self {
            signing_key,
            id,
            kind,
            public_key_hex,
        }
    }

    pub fn sign_json(&self, payload: &Value) -> Result<String, RavaError> {
        let canonical = canonical_json(payload)?;
        let signature = self.signing_key.sign(canonical.as_bytes());
        Ok(hex::encode(signature.to_bytes()))
    }
}

pub fn verify_json_signature(
    signer_id: &str,
    public_key_hex: &str,
    payload: &Value,
    signature_hex: &str,
) -> Result<bool, RavaError> {
    if !signer_id.starts_with("rava:") {
        return Ok(false);
    }

    let public_key_bytes: [u8; 32] = hex::decode(public_key_hex)?
        .try_into()
        .map_err(|_| RavaError::InvalidPublicKey)?;
    let signature_bytes: [u8; 64] = hex::decode(signature_hex)?
        .try_into()
        .map_err(|_| RavaError::InvalidSignature)?;

    let verifying_key =
        VerifyingKey::from_bytes(&public_key_bytes).map_err(|_| RavaError::InvalidPublicKey)?;
    let signature = Signature::from_bytes(&signature_bytes);
    let canonical = canonical_json(payload)?;

    Ok(verifying_key
        .verify(canonical.as_bytes(), &signature)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn signer_signs_and_verifies_payload() {
        let signer = Signer::generate(SignerKind::Human);
        let payload = json!({ "intent": "book_flight", "amount_usd": 500 });

        let signature = signer.sign_json(&payload).unwrap();

        assert!(verify_json_signature(
            &signer.id,
            &signer.public_key_hex,
            &payload,
            &signature
        )
        .unwrap());
    }

    #[test]
    fn verifier_rejects_modified_payload() {
        let signer = Signer::generate(SignerKind::Agent);
        let signed_payload = json!({ "intent": "book_flight", "amount_usd": 500 });
        let modified_payload = json!({ "intent": "book_flight", "amount_usd": 900 });
        let signature = signer.sign_json(&signed_payload).unwrap();

        assert!(!verify_json_signature(
            &signer.id,
            &signer.public_key_hex,
            &modified_payload,
            &signature
        )
        .unwrap());
    }
}
```

- [ ] **Step 5: Run identity tests**

Run: `cargo test -p rava-core identity`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/rava-core/src
git commit -m "feat: add signer identity"
```

## Milestone 4: Capabilities and Delegation

### Task 4: Mint and Attenuate Capabilities

**Why this matters:** A capability is a narrow permission object. Delegation should only make it weaker, never stronger.

**Files:**
- Create: `crates/rava-core/src/capability.rs`
- Modify: `crates/rava-core/src/lib.rs`

- [ ] **Step 1: Write tests first**

Tests must cover:

- a human mints a root capability for an agent;
- a personal agent delegates a narrower capability to a sub-agent;
- delegation fails if the child operation is not in the parent operations;
- delegation fails if the child expiry outlives the parent;
- delegation fails if the parent is not delegable.

- [ ] **Step 2: Verify tests fail**

Run: `cargo test -p rava-core capability`

Expected: FAIL because capability APIs do not exist.

- [ ] **Step 3: Implement capability schema**

Implement:

- `Capability`
- `CapabilityInput`
- `DelegationInput`
- `mint_capability`
- `delegate_capability`
- capability IDs derived from SHA-256 of canonical unsigned payload;
- signed proof over the canonical unsigned payload.

- [ ] **Step 4: Run tests**

Run: `cargo test -p rava-core capability`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/rava-core/src
git commit -m "feat: add capability delegation"
```

## Milestone 5: Actions and Verification

### Task 5: Verify Delegated Actions

**Why this matters:** This is the heart of Rava. The verifier decides whether a concrete action is allowed right now.

**Files:**
- Create: `crates/rava-core/src/action.rs`
- Create: `crates/rava-core/src/revocation.rs`
- Create: `crates/rava-core/src/verifier.rs`
- Modify: `crates/rava-core/src/lib.rs`

- [ ] **Step 1: Write scenario tests first**

Tests must cover:

- accepts a delegated flight purchase under the delegated amount;
- rejects a purchase over the delegated amount;
- rejects an expired capability;
- rejects a revoked capability;
- rejects an invalid parent chain;
- rejects an action whose actor is not the final capability subject;
- rejects an action with a tampered signature.

- [ ] **Step 2: Verify tests fail**

Run: `cargo test -p rava-core verifier`

Expected: FAIL because action, revocation, and verifier APIs do not exist.

- [ ] **Step 3: Implement action envelopes**

Implement signed action envelopes with:

- version;
- action ID;
- actor;
- controller;
- intent;
- resource;
- operation;
- constraints;
- capability ID;
- context hash;
- proof.

- [ ] **Step 4: Implement revocation registry**

Implement:

- `RevocationRegistry` trait;
- `InMemoryRevocationRegistry`;
- revocation checks for capability IDs and signer IDs.

- [ ] **Step 5: Implement verifier**

Verifier must check:

- action signature;
- final capability ID matches action capability ID;
- capability signatures;
- capability expiry;
- capability revocation;
- chain parent links;
- issuer equals parent subject for each delegated step;
- parent capability is delegable;
- final capability subject equals action actor;
- resource and operation match;
- action constraints do not exceed final capability constraints.

- [ ] **Step 6: Run tests**

Run: `cargo test -p rava-core verifier`

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/rava-core/src
git commit -m "feat: verify delegated actions"
```

## Milestone 6: CLI Demo and Protocol Draft

### Task 6: Demonstrate Flight Booking

**Files:**
- Create: `crates/rava-cli/tests/flight_booking.rs`
- Modify: `crates/rava-cli/src/main.rs`
- Create: `README.md`
- Create: `docs/protocol/rava-v0.md`

- [ ] **Step 1: Write failing CLI integration test**

The test should run:

```bash
cargo run -p rava -- demo flight-booking
```

Expected stdout:

```text
Rava verification accepted: true
```

- [ ] **Step 2: Verify test fails**

Run: `cargo test -p rava --test flight_booking`

Expected: FAIL because the demo command does not exist.

- [ ] **Step 3: Implement demo command**

Create the delegated flight-booking scenario:

1. human controller creates a personal agent capability for `travel.booking`;
2. personal agent delegates purchase under 800 USD to booking agent;
3. booking agent signs action for 750 USD;
4. verifier accepts the action;
5. CLI prints `Rava verification accepted: true`.

- [ ] **Step 4: Write README**

README must explain:

- Rava is action-native authorization for agents;
- Rust is the trusted core;
- V0 is not a global identity provider or reputation market;
- how to run tests;
- how to run the demo;
- the learning model: identity, capability, delegation, action, verifier.

- [ ] **Step 5: Write protocol draft**

`docs/protocol/rava-v0.md` must define:

- identity model;
- canonicalization rule;
- signature rule;
- capability schema;
- delegation rules;
- action schema;
- verification algorithm;
- revocation model;
- attestation model as a post-action extension.

- [ ] **Step 6: Verify all**

Run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p rava -- demo flight-booking
```

Expected: all commands pass and the demo prints `Rava verification accepted: true`.

- [ ] **Step 7: Commit**

```bash
git add README.md docs/protocol crates
git commit -m "docs: describe rava v0 protocol"
```

## Milestone 7: Product Readiness Review

### Task 7: Check Claims Against Implementation

**Files:**
- Modify: `README.md`
- Modify: `docs/protocol/rava-v0.md`
- Modify: `docs/superpowers/specs/2026-05-16-rava-product-direction.md`

- [ ] **Step 1: Audit claims**

Confirm docs only claim what V0 proves:

- local signer identities;
- deterministic canonicalization;
- Ed25519 signing and verification;
- capability minting;
- attenuated delegation;
- revocation checks;
- signed action verification;
- CLI demo.

- [ ] **Step 2: Downgrade unsupported claims**

Move these to roadmap language if present:

- blockchain anchoring;
- zero-knowledge reputation;
- production custody;
- OAuth replacement;
- global identity registry;
- insurance or bonding.

- [ ] **Step 3: Final verification**

Run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add README.md docs
git commit -m "docs: align claims with v0 implementation"
```

## Execution Recommendation

Start with Milestones 1 through 3 in one focused build session. That gives Rava a secure Rust foundation: deterministic bytes, identities, and signatures. Do not build reputation, blockchain anchoring, OAuth interop, or WASM until the delegated action verifier is correct and well tested.
