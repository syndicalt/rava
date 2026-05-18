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

## Rotation

Rotation should define:

- when the old key stops signing new protocol objects;
- how verifiers discover the new public key;
- how long old signed actions, receipts, and attestations remain verifiable;
- how emergency rotation revokes compromised signer IDs or capability IDs;
- how operators test rollback and recovery.

## Compromise Response

On suspected key compromise, production operators should:

- stop using the compromised private key;
- publish signer or capability revocations according to local policy;
- rotate affected keys;
- review accepted actions during the exposure window;
- preserve audit evidence for investigation;
- document residual risk.
