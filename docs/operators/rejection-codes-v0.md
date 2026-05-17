# Rava V0 Rejection Codes

This document describes stable V0 verifier rejection codes for operators, wrapper authors, and audit tooling. It is documentation for the current Rust verifier, not a new protocol feature.

## Operator Contract

Every rejected V0 action verification has:

- a stable machine-readable `code`;
- an optional `subject` that points at the relevant action ID, capability ID, signer ID, version string, operation, or constraint;
- fail-closed semantics.

Operators should treat every rejection as authorization failure. Logging and dashboards may group by `code`, but must not turn a rejection into acceptance. Subjects may include identifiers and constraint names, so handle them as audit data.

## Stable Codes

| Code | Subject | Meaning | Operator action |
| --- | --- | --- | --- |
| `unsupported_action_version` | version | Action version is not supported by this verifier. | Reject; use a supported V0 action or upgrade verifier intentionally. |
| `unsupported_capability_version` | version | Capability version is not supported by this verifier. | Reject; use a supported V0 capability or upgrade verifier intentionally. |
| `action_nonce_invalid` | none | Action nonce is not a canonical UUID v4 string. | Reject; require a newly signed action. |
| `capability_nonce_invalid` | capability ID | Capability nonce is not a canonical UUID v4 string. | Reject; require a newly signed capability chain. |
| `action_context_hash_invalid` | none | Action context hash is not `sha256:<64 lowercase hex>`. | Reject; fix the signed context reference and re-sign. |
| `action_signature_invalid` | none | Action signature does not verify against the supplied actor key. | Reject; investigate tampering or wrong actor key input. |
| `action_id_mismatch` | none | Action ID is not derived from the signed action fields. | Reject; require canonical ID derivation and re-signing. |
| `action_replayed` | action ID | One-time verifier has already accepted this action ID. | Reject; require a fresh signed action. |
| `capability_chain_empty` | none | Verifier received no capability chain. | Reject; supply the signed chain. |
| `action_capability_not_final` | none | Action capability ID does not match the final chain capability. | Reject; use the chain that grants the signed action. |
| `root_issuer_not_controller` | none | Root capability issuer does not match action controller. | Reject; controller and root issuer must align. |
| `capability_id_mismatch` | capability ID | Capability ID is not derived from the signed capability fields. | Reject; require canonical ID derivation and re-signing. |
| `capability_operations_empty` | capability ID | Capability has no operations. | Reject; capabilities must grant at least one operation. |
| `capability_operations_not_canonical` | capability ID | Capability operations are unsorted or duplicated. | Reject; normalize and re-sign. |
| `capability_signature_invalid` | capability ID | Capability signature does not verify against the supplied issuer key. | Reject; investigate tampering or wrong issuer key input. |
| `capability_revoked` | capability ID | Revocation registry marks the capability revoked. | Reject; require a non-revoked capability. |
| `signer_revoked` | signer ID | Revocation registry marks the actor or capability issuer revoked. | Reject; rotate or replace the signer according to caller policy. |
| `capability_expired` | capability ID | Capability expiry is at or before verifier time. | Reject; require a fresh capability. |
| `capability_parent_mismatch` | capability ID | Child capability does not link to the previous chain capability, or root has a parent. | Reject; supply a valid ordered chain. |
| `capability_issuer_not_parent_subject` | capability ID | Child issuer is not the parent subject. | Reject; delegation signer is not authorized by the parent. |
| `capability_resource_mismatch` | capability ID | Child resource differs from parent resource. | Reject; delegation broadened or changed resource. |
| `capability_operation_not_granted` | `capability_id:operation` | Child operation is not granted by parent. | Reject; delegation broadened operation scope. |
| `capability_expiry_outlives_parent` | capability ID | Child expiry is later than parent expiry. | Reject; delegation broadened time scope. |
| `capability_constraint_removed` | `capability_id:constraint` | Child omitted a parent constraint key. | Reject; delegation removed a required constraint. |
| `capability_constraint_expanded` | `capability_id:constraint` | Child constraint is broader than parent constraint. | Reject; delegation expanded constraint scope. |
| `parent_capability_not_delegable` | capability ID | Parent capability does not permit delegation. | Reject; parent authority cannot be delegated. |
| `final_subject_not_actor` | none | Final capability subject is not the action actor. | Reject; actor lacks final delegated authority. |
| `resource_mismatch` | none | Action resource differs from final capability resource. | Reject; signed action is outside resource scope. |
| `operation_not_allowed` | none | Action operation is not in the final capability operations. | Reject; signed action is outside operation scope. |
| `constraint_exceeded` | constraint | Action constraint is not covered by final capability constraints. | Reject; signed action is outside constraint scope. |
| `missing_issuer_public_key` | issuer ID | Caller did not provide the public key for a capability issuer. | Reject; supply an authentic issuer key from caller trust policy. |

## Stability

V0 code strings are intended to be stable for wrappers and logs. New codes may be added when new verifier checks are added. Existing code meaning should not be weakened.

## Receipts

Verification receipts can carry rejected decisions. A signed rejected receipt is audit evidence that the verifier denied an action; it is not authority to execute the action.
