# Rava Production Key Custody V0

This runbook defines key-custody requirements around Rava deployments. Key custody is not implemented by the Rava V0 core.

Rava verifies signatures against caller-supplied public keys. It does not decide how private keys are generated, stored, rotated, recovered, or retired.

## Required Properties

A production custody plan should define:

- generation ceremony and entropy source for human, agent, service, verifier, and evaluator keys;
- storage boundary for private keys;
- backup and recovery;
- rotation schedule;
- emergency rotation;
- compromise response;
- operator access review;
- audit evidence for key creation, activation, rotation, and retirement.

## Private-Key Handling

Private keys must not be logged, committed to examples, embedded in browser bundles, stored in public issue trackers, or exposed through debug output.

If a deployment intentionally places private keys in a browser, test harness, or low-assurance runtime, that custody boundary is a deployment risk and not a Rava core guarantee.

## Local CLI Guardrails

`rava key generate` is a local development and controlled-deployment helper, not a managed custody system. It provides these local guardrails:

- generated key output prints the signer ID and public key, not `private_key_hex`;
- generated private-key files are owner-only on Unix;
- loading a private-key file on Unix fails if the file is readable, writable, or executable by group or others;
- forced key generation replaces the destination path instead of writing private key material through an existing symlink.

These guardrails reduce accidental local exposure. They do not replace a production custody provider, hardware-backed key storage, cloud KMS, recovery process, or operator access review.

## Local Compromise Response Helper

`rava key revoke --id <signer-id> --revocation-store <path>` records a signer ID in the local revocation snapshot consumed by `rava verify action --revocation-store`. Local file-backed updates are lock-serialized, merge existing revoked IDs before persisting, and preserve existing `fresh_until_unix` metadata.

This helper gives controlled deployments a local break-glass path for suspected signer compromise. It is not a managed custody provider, rotation ceremony, emergency propagation system, key-discovery update, operator approval workflow, or production incident-response process.

## Rotation

Rotation should define:

- when the old key stops signing new protocol objects;
- how verifiers discover the new public key;
- how long old signed actions, receipts, and attestations remain verifiable;
- how emergency rotation revokes compromised signer IDs or capability IDs;
- how operators test rollback and recovery.

## Local Rotation Exercise

For a controlled local deployment, operators can rehearse the non-production rotation path with existing explicit-trust primitives:

1. Generate a replacement key with `rava key generate --kind <kind> --out <new-key.json>`.
2. Stop using the old private key for new protocol objects.
3. Update the explicit static trust bundle with the new signer ID and public key.
4. Verify newly signed objects with the updated trust bundle before relying on the replacement key.
5. For emergency rotation after suspected compromise, revoke the old signer ID with `rava key revoke --id <old-signer-id> --revocation-store <revocations.json>`.
6. If revocation freshness is required, publish or refresh the local revocation snapshot so `fresh_until_unix` remains greater than verifier `now_unix`.

This exercise is useful for local readiness evidence. It is not a production rotation ceremony, custody-system test, trust-bundle publication system, rollback test, or operator approval workflow.

## Compromise Response

On suspected key compromise, production operators should:

- stop using the compromised private key;
- publish signer or capability revocations according to local policy;
- rotate affected keys;
- review accepted actions during the exposure window;
- preserve audit evidence for investigation;
- document residual risk.
