## Summary

- 

## Security Boundary

Does this change affect canonicalization, signing, verification, expiry, revocation, replay, receipts, attestations, wrappers, workflows, release docs, or security docs?

- [ ] Yes
- [ ] No

If yes, describe the affected boundary and the fail-closed behavior preserved by this change:

- 

Confirm:

- [ ] No new cryptographic primitives
- [ ] No verifier shortcuts
- [ ] No test-only bypasses
- [ ] No production-ready or externally audited claim

## Required Evidence

Record exact command results before requesting review:

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

## Review Artifacts

Reference any affected review artifacts:

- `docs/security/threat-model-v0.md`
- `docs/security/review-register-v0.md`
- `.github/CODEOWNERS`
- `.github/ISSUE_TEMPLATE/security-review-finding.yml`

If this PR remediates a review finding, link the issue, finding record, regression test, and verification evidence.
