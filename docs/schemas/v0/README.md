# Rava V0 Wire Schemas

Schemas describe wire shape only. They do not verify signatures, derived IDs, replay state, revocation state, expiry, delegation attenuation, key authenticity, or trust policy.

Use the Rust verifier for authorization decisions. These schemas exist to help implementers parse and preflight Rava V0 JSON objects before passing them to verifier logic.

The timestamp representation matches the current Rust `time::OffsetDateTime` serde encoding used by the V0 wire examples.
