# Rava Production Monitoring V0

This runbook defines production monitoring requirements around Rava deployments. Production monitoring is not implemented by the V0 preview service.

The preview service can be observed by ordinary process and log tooling, but managed metrics, alerting, dashboards, and incident response are deployment responsibilities.

## Required Signals

Production monitoring should track:

- accepted and rejected decision rates;
- rejection code distribution;
- missing public keys;
- stale or failed revocation reads;
- replay attempts;
- audit-write failures;
- verifier latency and saturation;
- request body limit rejections;
- authentication and rate-limit rejections at the ingress layer;
- dependency outages.

## Sensitive Data

Monitoring must not leak private keys, credentials, raw action payloads, access tokens, capability envelopes, or downstream secrets.

Metrics should prefer counts, rates, stable rejection codes, and non-sensitive identifiers.

## Alerts

Operators should define alerts for:

- sudden acceptance or rejection-rate changes;
- repeated replay attempts;
- increased missing-key failures;
- revocation or replay store failures;
- audit-write failures;
- verifier saturation or dependency outage.

## Incident Use

Monitoring evidence should correlate with audit records, verification receipts, attestations, resolver evidence, and downstream tool or API use without requiring raw sensitive payload collection by default.
