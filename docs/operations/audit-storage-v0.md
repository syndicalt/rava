# Rava Production Audit Storage V0

This runbook defines audit-storage requirements for production deployments around Rava. Managed audit storage is not implemented by the V0 preview service.

The preview service can append local decision metadata for development and integration testing. That local file is not durable, replicated, access-controlled, or retention-managed by Rava.

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
