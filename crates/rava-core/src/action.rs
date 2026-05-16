use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::canonical::{canonical_json, CanonicalError};
use crate::capability::ConstraintValue;
use crate::error::RavaError;
use crate::hash::is_sha256_hash;
use crate::identity::Signer;
use crate::protocol::{ACTION_ID_PREFIX, ACTION_PENDING_ID, ACTION_VERSION};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionProof {
    pub signature_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionEnvelope {
    pub version: String,
    pub id: String,
    pub nonce: String,
    pub actor: String,
    pub controller: String,
    pub intent: String,
    pub resource: String,
    pub operation: String,
    pub constraints: BTreeMap<String, ConstraintValue>,
    pub capability_id: String,
    pub context_hash: String,
    pub proof: ActionProof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionInput {
    pub controller: String,
    pub intent: String,
    pub resource: String,
    pub operation: String,
    pub constraints: BTreeMap<String, ConstraintValue>,
    pub capability_id: String,
    pub context_hash: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ActionError {
    #[error(transparent)]
    Canonical(#[from] CanonicalError),

    #[error(transparent)]
    Rava(#[from] RavaError),

    #[error("action context_hash must be sha256:<64 lowercase hex chars>")]
    InvalidContextHash,
}

pub fn sign_action(actor: &Signer, input: ActionInput) -> Result<ActionEnvelope, ActionError> {
    if !is_sha256_hash(&input.context_hash) {
        return Err(ActionError::InvalidContextHash);
    }

    let nonce = Uuid::new_v4().to_string();
    let pending_id = ACTION_PENDING_ID;
    let unsigned = unsigned_action_value(pending_id, &nonce, &actor.id, &input);
    let id = action_id(&unsigned)?;
    let signing_payload = unsigned_action_value(&id, &nonce, &actor.id, &input);
    let signature_hex = actor.sign_json(&signing_payload)?;

    Ok(ActionEnvelope {
        version: ACTION_VERSION.to_owned(),
        id,
        nonce,
        actor: actor.id.clone(),
        controller: input.controller,
        intent: input.intent,
        resource: input.resource,
        operation: input.operation,
        constraints: input.constraints,
        capability_id: input.capability_id,
        context_hash: input.context_hash,
        proof: ActionProof { signature_hex },
    })
}

pub fn action_signing_payload(action: &ActionEnvelope) -> Value {
    json!({
        "version": action.version,
        "id": action.id,
        "nonce": action.nonce,
        "actor": action.actor,
        "controller": action.controller,
        "intent": action.intent,
        "resource": action.resource,
        "operation": action.operation,
        "constraints": action.constraints,
        "capability_id": action.capability_id,
        "context_hash": action.context_hash,
    })
}

pub fn expected_action_id(action: &ActionEnvelope) -> Result<String, CanonicalError> {
    let unsigned = json!({
        "version": action.version,
        "id": ACTION_PENDING_ID,
        "nonce": action.nonce,
        "actor": action.actor,
        "controller": action.controller,
        "intent": action.intent,
        "resource": action.resource,
        "operation": action.operation,
        "constraints": action.constraints,
        "capability_id": action.capability_id,
        "context_hash": action.context_hash,
    });

    action_id(&unsigned)
}

fn unsigned_action_value(id: &str, nonce: &str, actor: &str, input: &ActionInput) -> Value {
    json!({
        "version": ACTION_VERSION,
        "id": id,
        "nonce": nonce,
        "actor": actor,
        "controller": input.controller,
        "intent": input.intent,
        "resource": input.resource,
        "operation": input.operation,
        "constraints": input.constraints,
        "capability_id": input.capability_id,
        "context_hash": input.context_hash,
    })
}

fn action_id(unsigned: &Value) -> Result<String, CanonicalError> {
    let canonical = canonical_json(unsigned)?;
    let digest = Sha256::digest(canonical.as_bytes());
    Ok(format!(
        "{}{}",
        ACTION_ID_PREFIX,
        hex::encode(&digest[..16])
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{Signer, SignerKind};
    use std::collections::BTreeMap;

    #[test]
    fn sign_action_rejects_invalid_context_hash() {
        let actor = Signer::generate(SignerKind::Agent);

        let result = sign_action(
            &actor,
            ActionInput {
                controller: "rava:human:controller".to_owned(),
                intent: "book_flight".to_owned(),
                resource: "travel.booking".to_owned(),
                operation: "purchase".to_owned(),
                constraints: BTreeMap::new(),
                capability_id: "cap_demo".to_owned(),
                context_hash: "sha256:not-hex".to_owned(),
            },
        );

        assert!(matches!(result, Err(ActionError::InvalidContextHash)));
    }
}
