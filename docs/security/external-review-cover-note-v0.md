# Rava V0 External Review Cover Note

Use this note when sending the Rava V0 draft reference implementation to an external reviewer.

Rava V0 is not production-ready security software. This cover note is not evidence that Rava has been externally reviewed, certified, or approved for production use.

## Review Target

- Review candidate tag: `v0-review-candidate-2026-05-18-r2`
- Frozen target commit: `d611c6d1c2fd00d7a3d46a4031bdea65820fe78b`
- Tracking issue: https://github.com/syndicalt/rava/issues/87
- Review candidate notes: `docs/release/v0-review-candidate-2026-05-18-r2.md`

Do not treat later commits as part of the frozen review target unless the reviewer explicitly agrees to review the new commit or tag.

## Start Here

Please start with:

- `docs/security/external-review-packet-v0.md`
- `docs/security/threat-model-v0.md`
- `docs/protocol/rava-v0.md`
- `docs/security/review-guide-v0.md`
- `docs/security/review-register-v0.md`

Use `.github/ISSUE_TEMPLATE/security-review-finding.yml` for structured finding intake when opening GitHub issues.

## Requested Review Focus

Please prioritize:

- canonicalization;
- signature binding;
- delegation attenuation;
- replay semantics;
- revocation semantics;
- receipt and attestation verification;
- fail-closed behavior for malformed, tampered, expired, revoked, replayed, or over-scoped inputs.

## Out of Scope for V0 Guarantees

These are production requirements, not implemented V0 guarantees:

- production key custody;
- public-key discovery and DID resolution;
- distributed replay;
- distributed revocation;
- caller identity;
- distributed rate limiting;
- managed audit storage;
- hosted verifier operations;
- production monitoring.

Findings about these areas are still useful, but they should be classified as production work, accepted risk, or out-of-scope rather than implied V0 guarantees.

## Local Verification Commands

The expected full local gate is:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p rava -- demo flight-booking
cargo package --workspace
cargo check --manifest-path fuzz/Cargo.toml --locked
cargo check -p rava-wasm --target wasm32-unknown-unknown
npm --prefix packages/rava-wasm-js test
(cd packages/rava-wasm-js && npm pack --dry-run)
```

Passing this gate is useful baseline evidence. It does not prove protocol security and does not replace external review.
