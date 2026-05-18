# Rava V0 Fuzz Campaign Log Template

This template records optional longer fuzz campaigns for Rava V0. A fuzz campaign is not a proof of security, an external audit, or production readiness evidence by itself.

Copy this file to a dated campaign file such as `docs/security/fuzz-campaigns/2026-05-18-v0-wire-entrypoints.md` before filling it in.

## Campaign Metadata

- Commit SHA:
- Command: `cargo fuzz run v0_wire_entrypoints -- -max_total_time=86400`
- Duration:
- Host:
- OS and kernel:
- Rust toolchain:
- cargo-fuzz version:
- Corpus path:
- Artifact path:

## Coverage Intent

This campaign targets:

- JSON parsing;
- canonicalization;
- action verification;
- receipt verification;
- attestation verification.

The fuzz target is `fuzz/fuzz_targets/v0_wire_entrypoints.rs`.

## Results

- Start time:
- End time:
- Total executions:
- Crash count:
- Timeout count:
- OOM count:
- Minimized crashing inputs:
- Sanitizer or panic output:

## Remediation

For each crash or bug:

- finding ID:
- root cause:
- Regression tests:
- Pull requests:
- verification command:

## Final Rerun

After remediation, rerun the same command or a reviewer-approved reduced command and record:

- command:
- duration:
- result:
- remaining crashes:
- reviewer or maintainer who verified the rerun:
