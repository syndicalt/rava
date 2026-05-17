# Rava WASM V0 Wrapper

The `rava-wasm` crate exposes a small WASM boundary around the Rust verifier. It calls the Rust verifier and does not reimplement signatures, canonicalization, or attenuation.

The wrapper is intended for browser, Node, and agent-tooling environments that need to run V0 verification without porting trusted protocol logic to another language.

## Exported Function

`verify_action_json(request_json)` accepts a JSON string and returns a JSON string with:

- `accepted`;
- `rejection`, with stable `code` and optional `subject` when rejected.

The request JSON contains:

- `action`;
- `capability_chain`;
- `actor_public_key_hex`;
- `issuer_public_keys`;
- `now_unix`;
- `revoked_ids`, optional caller-supplied revoked signer or capability IDs.

The wrapper does not perform key discovery, DID resolution, replay persistence, distributed revocation freshness, request authentication, or trust-policy decisions. Callers must provide authentic keys and sufficiently fresh revocation input.

## Verification

Native wrapper regression tests run with:

```bash
cargo test -p rava-wasm
```

The actual WASM target is checked with:

```bash
cargo check -p rava-wasm --target wasm32-unknown-unknown
```

The WASM compile uses target-scoped `getrandom` and `uuid` JS features because shared Rust dependencies include signing and nonce-generation code. The exported wrapper still verifies only; it does not generate keys, signatures, capabilities, actions, receipts, or attestations.
