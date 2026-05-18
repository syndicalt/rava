# Rava Production Caller Identity V0

This runbook defines caller-identity requirements for production verifier deployments. Caller identity is not implemented by the V0 preview service.

Rava verifies whether a signed action is authorized by a signed delegation chain. That action actor is not necessarily the same thing as the network caller, tenant, customer, API client, or service principal submitting the request.

## Required Properties

A production ingress layer should define:

- ingress authentication for clients submitting verification requests;
- caller-to-policy mapping;
- tenant or customer boundary;
- authorization to use a key source, replay store, revocation source, and audit destination;
- audit evidence for caller identity;
- behavior for missing or ambiguous callers.

## Separation From Action Actors

Caller identity must not be inferred from the action actor.

The action actor is the signer attempting to perform the action. The caller is the deployment principal invoking the verifier. A hosted verifier may need both: the action actor for protocol verification and the caller identity for tenant policy, quotas, resolver selection, audit routing, and abuse controls.

## Local Preview Guardrails

When `rava serve verify --caller-id <label>` is configured, the preview service records that explicit deployment label in local audit entries as `caller_id`. The value is not derived from the signed action actor or from unauthenticated request headers.

`--caller-id` requires `--auth-token-env`, so the preview service fails closed at startup rather than recording a caller label without the local bearer-token ingress guard. `GET /healthz` reports whether a caller label is configured.

This is local audit correlation evidence for a controlled deployment. It is not caller-to-policy mapping, tenant isolation, authorization to use trust stores or audit destinations, multi-caller identity, or a production ingress identity system.

## Fail-Closed Rule

If production policy requires caller identity and the ingress layer cannot authenticate or map the caller, the request should fail closed before verifier execution.

Accepting unauthenticated caller identity headers as authoritative is a deployment vulnerability, not a Rava core guarantee.

## Audit Use

Caller identity should be recorded as deployment audit metadata and correlated with verification receipts, resolver evidence, replay consumption, revocation checks, and downstream tool or API use.
