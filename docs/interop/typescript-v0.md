# Rava TypeScript V0 Package

The `packages/rava-wasm-js` package is a TypeScript wrapper for the Rava WASM verifier. It calls the generated WASM wrapper and does not reimplement Rava verification.

The package builds `crates/rava-wasm` for `wasm32-unknown-unknown`, runs `wasm-bindgen` for Node.js glue, compiles TypeScript, and runs Node tests against the V0 flight-booking vector.

## API

`verifyAction(request)` accepts:

- `action`;
- `capability_chain`;
- `actor_public_key_hex`;
- `issuer_public_keys`;
- `now_unix`;
- `revoked_ids`, optional caller-supplied revoked signer or capability IDs.

It returns:

- `accepted`;
- `rejection`, with stable `code` and optional `subject` when rejected.

## Verification

Run the package test with:

```bash
npm --prefix packages/rava-wasm-js test
```

The test builds the WASM artifact first, then verifies that the TypeScript API accepts the committed V0 flight-booking vector and preserves Rust verifier rejection codes for malformed signed input.

The package is an interop wrapper only. It does not perform key discovery, replay persistence, distributed revocation freshness, request authentication, or trust-policy decisions.
