# Rava V0 Fuzz Campaign: v0_wire_entrypoints 2026-05-18 1800s

This bounded longer campaign is review evidence only. It is not a proof of security, external audit, or production readiness certification.

This campaign ran after the frozen external-review candidate was recorded. It does not change the frozen external-review target.

## Campaign Metadata

- Commit SHA: `becbff9e2326f5304822decf636aadcd0e37bb48`
- Command: `cargo +nightly fuzz run v0_wire_entrypoints -- -max_total_time=1800`
- Duration: 1801 seconds
- Start time: `2026-05-18T03:25:30Z`
- End time: approximately `2026-05-18T03:55:31Z`
- Host: `NB-SLIM7-9I`
- OS and kernel: `Linux NB-SLIM7-9I 6.17.0-23-generic #23~24.04.1-Ubuntu SMP PREEMPT_DYNAMIC Tue Apr 14 16:11:48 UTC 2 x86_64 x86_64 x86_64 GNU/Linux`
- Rust toolchain: `rustc 1.97.0-nightly (507271bc1 2026-05-17)`
- Cargo toolchain: `cargo 1.97.0-nightly (4d1f98451 2026-05-15)`
- cargo-fuzz version: `0.13.1`
- Corpus path: `fuzz/corpus/v0_wire_entrypoints` (generated locally; not committed)
- Artifact path: `fuzz/artifacts/v0_wire_entrypoints`
- Seed: `2752387297`

## Coverage Intent

This campaign targets:

- JSON parsing;
- canonicalization;
- action verification;
- receipt verification;
- attestation verification.

The fuzz target is `fuzz/fuzz_targets/v0_wire_entrypoints.rs`.

## Results

- Total executions: 58,358,035
- Final coverage: `cov: 2657`
- Final feature count: `ft: 10540`
- Final corpus: `2548/1089Kb`
- Input limit: `4096`
- Final exec/s: `32403`
- Final RSS: `736Mb`
- Crash count: 0
- Timeout count: 0
- OOM count: 0
- Minimized crashing inputs: none
- Sanitizer or panic output: none observed
- Final line: `Done 58358035 runs in 1801 second(s)`

The run started from an empty local corpus and generated local corpus entries under `fuzz/corpus/v0_wire_entrypoints`. Those generated entries are not committed release artifacts.

## Remediation

No crash or bug was found in this bounded campaign, so no remediation PR or new regression test was required.

- Finding ID: not applicable
- Root cause: not applicable
- Regression tests: not applicable
- Pull requests: this campaign log
- Verification command: `cargo +nightly fuzz run v0_wire_entrypoints -- -max_total_time=1800` exited 0

## Final Rerun

No final rerun was required because no remediation was performed.
