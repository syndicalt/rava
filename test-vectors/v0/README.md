# Rava V0 Test Vectors

This directory contains language-neutral wire vectors for independent Rava V0 implementations.

The vectors are JSON files only. They include signed actions, capability chains, receipts, attestations, public keys, empty replay and revocation stores, and tampered objects that must fail verification.

They intentionally do not include private keys. A verifier implementation should use `manifest.json` to find the relevant files and expected outcomes.

`rava inspect` may summarize these files, but inspect output is not authorization. `rava verify` or an equivalent verifier implementation is required for signature, replay, revocation, expiry, and attenuation checks.
