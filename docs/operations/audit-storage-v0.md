# Rava Production Audit Storage V0

This runbook defines audit-storage requirements for production deployments around Rava. Managed audit storage is not implemented by the V0 preview service.

The preview service can append local decision metadata for development and integration testing. That local file is not durable, replicated, access-controlled, or retention-managed by Rava.

## Local Preview Guardrails

When `rava serve verify --audit-log <path>` is configured, the preview service appends newline-delimited decision metadata only. It does not write raw action payloads, capability envelopes, signatures, keys, credentials, or downstream secrets.

On Unix, newly created local audit log files are owner-only. Existing audit log files that are readable, writable, or executable by group or others are rejected before a decision entry is written. The preview service also opens the final audit log path without following a symlink and flushes and syncs the appended entry before returning a verifier response.

These guardrails reduce accidental local disclosure during development and integration testing. They are not managed audit storage, retention, export, tamper evidence, access-control review, deletion policy, or legal-hold support.

## Local Export Helper

`rava audit export --audit-log <path>` reads local preview audit NDJSON and writes a JSON array to standard output. Optional `--since-unix` and `--until-unix` bounds are inclusive and filter on `verified_at_unix`; when a time filter is used, entries missing `verified_at_unix` fail closed. The export helper rejects entries that contain raw payload-style fields such as `action`, `capability_chain`, `intent`, `resource`, `constraints`, or `proof`.

This helper is useful for local review and issue evidence. It is not managed retention, access control, tamper evidence, deletion policy, legal hold, customer reporting, or production audit export.

## Required Properties

A production audit system should provide:

- retention policy for verification decisions, receipts, attestations, resolver evidence, and downstream execution evidence;
- privacy classification for fields that may identify users, agents, resources, vendors, or business intent;
- export for incident response, customer reporting, legal hold, and independent review;
- tamper evidence for appended or modified records;
- access control for operators, reviewers, auditors, and automated pipelines;
- deletion or legal hold behavior that is documented before deployment;
- operational monitoring for write failures, storage saturation, and export failures.

## Failure Policy

Each deployment must define what happens when audit storage is unavailable.

If the deployment requires audit evidence before action execution, the caller should fail closed when the audit write cannot be completed durably. If the deployment allows best-effort audit writes, that residual risk must be documented outside the Rava core and surfaced to operators.

The preview service's local audit log is useful evidence for development, but it is not a production durability guarantee.

## Correlation

Production audit records should support correlation across:

- Rava verification receipt ID and action ID;
- signed verification receipt;
- signed post-action attestation;
- public-key resolver evidence;
- replay and revocation snapshot evidence;
- downstream tool or API call;
- caller identity from the deployment's ingress layer;
- incident or support case references.

Correlation identifiers should be enough to investigate decisions without requiring raw sensitive payload storage by default.

## Data Minimization

Audit systems should avoid storing raw action payloads, capability envelopes, signatures, private keys, credentials, access tokens, or downstream secrets unless a deployment explicitly needs them and has a handling policy.

The default production posture should store decision metadata, stable IDs, rejection codes, timestamps, key-source evidence, and references to separately governed records.

## Review Checklist

Before representing a deployment as production-ready, reviewers should confirm:

- audit records are durable before an accepted action is executed, if policy requires that guarantee;
- export has been tested with representative accepted and rejected decisions;
- retention and deletion policies are documented;
- access controls are reviewed;
- tamper-evidence controls are documented and tested;
- sensitive payload storage is either avoided or covered by a data-handling policy.
