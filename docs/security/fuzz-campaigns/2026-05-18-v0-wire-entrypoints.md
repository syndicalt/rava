# Rava V0 Fuzz Campaign: v0_wire_entrypoints 2026-05-18

This bounded campaign is review evidence only. It is not a proof of security, external audit, or production readiness certification.

## Campaign Metadata

- Commit SHA: `83bf11da3078c643acabb35ccec409725b4f95a2`
- Command: `cargo +nightly fuzz run v0_wire_entrypoints -- -max_total_time=600`
- Duration: 601 seconds
- Start time: `2026-05-18T02:31:08Z`
- End time: approximately `2026-05-18T02:41:09Z`
- Host: `NB-SLIM7-9I`
- OS and kernel: `Linux NB-SLIM7-9I 6.17.0-23-generic #23~24.04.1-Ubuntu SMP PREEMPT_DYNAMIC Tue Apr 14 16:11:48 UTC 2 x86_64 x86_64 x86_64 GNU/Linux`
- Rust toolchain: `rustc 1.97.0-nightly (507271bc1 2026-05-17)`
- Cargo toolchain: `cargo 1.97.0-nightly (4d1f98451 2026-05-15)`
- cargo-fuzz version: `0.13.1`
- Corpus path: `fuzz/corpus/v0_wire_entrypoints`
- Artifact path: `fuzz/artifacts/v0_wire_entrypoints`
- Seed: `3852110329`

## Coverage Intent

This campaign targets:

- JSON parsing;
- canonicalization;
- action verification;
- receipt verification;
- attestation verification.

The fuzz target is `fuzz/fuzz_targets/v0_wire_entrypoints.rs`.

## Results

- Total executions: 20,742,375
- Final coverage: `cov: 2343`
- Final feature count: `ft: 9732`
- Final corpus: `2315/827Kb`
- Input limit: `4096`
- Final exec/s: `34513`
- Final RSS: `562Mb`
- Crash count: 0
- Timeout count: 0
- OOM count: 0
- Minimized crashing inputs: none
- Sanitizer or panic output: none observed
- Final line: `Done 20742375 runs in 601 second(s)`

## Remediation

No crash or bug was found in this bounded campaign, so no remediation PR or new regression test was required.

- Finding ID: not applicable
- Root cause: not applicable
- Regression tests: not applicable
- Pull requests: this campaign log
- Verification command: `cargo +nightly fuzz run v0_wire_entrypoints -- -max_total_time=600` exited 0

## Final Rerun

No final rerun was required because no remediation was performed.
