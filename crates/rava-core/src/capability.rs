use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::canonical::{canonical_json, CanonicalError};
use crate::constraints::value_is_no_broader_than;
use crate::error::RavaError;
use crate::identity::Signer;
use crate::protocol::{CAPABILITY_ID_PREFIX, CAPABILITY_PENDING_ID, CAPABILITY_VERSION};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintValue {
    Integer(u64),
    Text(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityProof {
    pub signature_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub version: String,
    pub id: String,
    pub nonce: String,
    pub issuer: String,
    pub subject: String,
    pub resource: String,
    pub operations: Vec<String>,
    pub constraints: BTreeMap<String, ConstraintValue>,
    pub expires_at: OffsetDateTime,
    pub delegable: bool,
    pub parent_id: Option<String>,
    pub proof: CapabilityProof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityInput {
    pub subject: String,
    pub resource: String,
    pub operations: Vec<String>,
    pub constraints: BTreeMap<String, ConstraintValue>,
    pub expires_at: OffsetDateTime,
    pub delegable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegationInput {
    pub subject: String,
    pub operations: Vec<String>,
    pub constraints: BTreeMap<String, ConstraintValue>,
    pub expires_at: OffsetDateTime,
    pub delegable: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum CapabilityError {
    #[error(transparent)]
    Canonical(#[from] CanonicalError),

    #[error(transparent)]
    Rava(#[from] RavaError),

    #[error("capability serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("parent capability is not delegable")]
    ParentNotDelegable,

    #[error("issuer is not the parent capability subject")]
    IssuerNotParentSubject,

    #[error("capability operations cannot be empty")]
    OperationsEmpty,

    #[error("operation is not allowed by parent capability: {operation}")]
    OperationNotAllowed { operation: String },

    #[error("delegated capability cannot outlive parent capability")]
    ExpiryOutlivesParent,

    #[error("delegation expands constraint {constraint}")]
    ConstraintExpansion { constraint: String },

    #[error("delegation removes constraint {constraint}")]
    ConstraintRemoval { constraint: String },
}

#[derive(Debug, Serialize)]
struct UnsignedCapability<'a> {
    version: &'a str,
    id: &'a str,
    nonce: &'a str,
    issuer: &'a str,
    subject: &'a str,
    resource: &'a str,
    operations: &'a [String],
    constraints: &'a BTreeMap<String, ConstraintValue>,
    expires_at: OffsetDateTime,
    delegable: bool,
    parent_id: &'a Option<String>,
}

pub fn mint_capability(
    issuer: &Signer,
    input: CapabilityInput,
) -> Result<Capability, CapabilityError> {
    let nonce = Uuid::new_v4().to_string();
    let mut operations = input.operations;
    operations.sort();
    operations.dedup();
    if operations.is_empty() {
        return Err(CapabilityError::OperationsEmpty);
    }
    let parent_id = None;
    let pending_id = CAPABILITY_PENDING_ID;
    let unsigned = unsigned_capability_value(UnsignedCapability {
        version: CAPABILITY_VERSION,
        id: pending_id,
        nonce: &nonce,
        issuer: &issuer.id,
        subject: &input.subject,
        resource: &input.resource,
        operations: &operations,
        constraints: &input.constraints,
        expires_at: input.expires_at,
        delegable: input.delegable,
        parent_id: &parent_id,
    });
    let id = capability_id(&unsigned)?;

    let signing_payload = unsigned_capability_value(UnsignedCapability {
        version: CAPABILITY_VERSION,
        id: &id,
        nonce: &nonce,
        issuer: &issuer.id,
        subject: &input.subject,
        resource: &input.resource,
        operations: &operations,
        constraints: &input.constraints,
        expires_at: input.expires_at,
        delegable: input.delegable,
        parent_id: &parent_id,
    });
    let signature_hex = issuer.sign_json(&signing_payload)?;

    Ok(Capability {
        version: CAPABILITY_VERSION.to_owned(),
        id,
        nonce,
        issuer: issuer.id.clone(),
        subject: input.subject,
        resource: input.resource,
        operations,
        constraints: input.constraints,
        expires_at: input.expires_at,
        delegable: input.delegable,
        parent_id,
        proof: CapabilityProof { signature_hex },
    })
}

pub fn delegate_capability(
    issuer: &Signer,
    parent: &Capability,
    input: DelegationInput,
) -> Result<Capability, CapabilityError> {
    if !parent.delegable {
        return Err(CapabilityError::ParentNotDelegable);
    }
    if parent.subject != issuer.id {
        return Err(CapabilityError::IssuerNotParentSubject);
    }
    for operation in &input.operations {
        if !parent.operations.contains(operation) {
            return Err(CapabilityError::OperationNotAllowed {
                operation: operation.clone(),
            });
        }
    }
    if input.expires_at > parent.expires_at {
        return Err(CapabilityError::ExpiryOutlivesParent);
    }
    ensure_constraints_are_not_removed(&parent.constraints, &input.constraints)?;
    ensure_constraints_do_not_expand(&parent.constraints, &input.constraints)?;

    let nonce = Uuid::new_v4().to_string();
    let mut operations = input.operations;
    operations.sort();
    operations.dedup();
    if operations.is_empty() {
        return Err(CapabilityError::OperationsEmpty);
    }
    let parent_id = Some(parent.id.clone());
    let pending_id = CAPABILITY_PENDING_ID;
    let unsigned = unsigned_capability_value(UnsignedCapability {
        version: CAPABILITY_VERSION,
        id: pending_id,
        nonce: &nonce,
        issuer: &issuer.id,
        subject: &input.subject,
        resource: &parent.resource,
        operations: &operations,
        constraints: &input.constraints,
        expires_at: input.expires_at,
        delegable: input.delegable,
        parent_id: &parent_id,
    });
    let id = capability_id(&unsigned)?;

    let signing_payload = unsigned_capability_value(UnsignedCapability {
        version: CAPABILITY_VERSION,
        id: &id,
        nonce: &nonce,
        issuer: &issuer.id,
        subject: &input.subject,
        resource: &parent.resource,
        operations: &operations,
        constraints: &input.constraints,
        expires_at: input.expires_at,
        delegable: input.delegable,
        parent_id: &parent_id,
    });
    let signature_hex = issuer.sign_json(&signing_payload)?;

    Ok(Capability {
        version: CAPABILITY_VERSION.to_owned(),
        id,
        nonce,
        issuer: issuer.id.clone(),
        subject: input.subject,
        resource: parent.resource.clone(),
        operations,
        constraints: input.constraints,
        expires_at: input.expires_at,
        delegable: input.delegable,
        parent_id,
        proof: CapabilityProof { signature_hex },
    })
}

fn unsigned_capability_value(capability: UnsignedCapability<'_>) -> Value {
    json!({
        "version": capability.version,
        "id": capability.id,
        "nonce": capability.nonce,
        "issuer": capability.issuer,
        "subject": capability.subject,
        "resource": capability.resource,
        "operations": capability.operations,
        "constraints": capability.constraints,
        "expires_at": capability.expires_at,
        "delegable": capability.delegable,
        "parent_id": capability.parent_id,
    })
}

pub fn capability_signing_payload(capability: &Capability) -> Value {
    json!({
        "version": capability.version,
        "id": capability.id,
        "nonce": capability.nonce,
        "issuer": capability.issuer,
        "subject": capability.subject,
        "resource": capability.resource,
        "operations": capability.operations,
        "constraints": capability.constraints,
        "expires_at": capability.expires_at,
        "delegable": capability.delegable,
        "parent_id": capability.parent_id,
    })
}

pub fn expected_capability_id(capability: &Capability) -> Result<String, CanonicalError> {
    let unsigned = unsigned_capability_value(UnsignedCapability {
        version: &capability.version,
        id: CAPABILITY_PENDING_ID,
        nonce: &capability.nonce,
        issuer: &capability.issuer,
        subject: &capability.subject,
        resource: &capability.resource,
        operations: &capability.operations,
        constraints: &capability.constraints,
        expires_at: capability.expires_at,
        delegable: capability.delegable,
        parent_id: &capability.parent_id,
    });

    capability_id(&unsigned)
}

fn capability_id(unsigned: &Value) -> Result<String, CanonicalError> {
    let canonical = canonical_json(unsigned)?;
    let digest = Sha256::digest(canonical.as_bytes());
    Ok(format!(
        "{}{}",
        CAPABILITY_ID_PREFIX,
        hex::encode(&digest[..16])
    ))
}

fn ensure_constraints_do_not_expand(
    parent: &BTreeMap<String, ConstraintValue>,
    child: &BTreeMap<String, ConstraintValue>,
) -> Result<(), CapabilityError> {
    for (key, child_value) in child {
        let Some(parent_value) = parent.get(key) else {
            return Err(CapabilityError::ConstraintExpansion {
                constraint: key.clone(),
            });
        };

        if !value_is_no_broader_than(child_value, parent_value) {
            return Err(CapabilityError::ConstraintExpansion {
                constraint: key.clone(),
            });
        }
    }

    Ok(())
}

fn ensure_constraints_are_not_removed(
    parent: &BTreeMap<String, ConstraintValue>,
    child: &BTreeMap<String, ConstraintValue>,
) -> Result<(), CapabilityError> {
    for key in parent.keys() {
        if !child.contains_key(key) {
            return Err(CapabilityError::ConstraintRemoval {
                constraint: key.clone(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{Signer, SignerKind};
    use std::collections::BTreeMap;
    use std::error::Error;
    use time::OffsetDateTime;

    fn expires_at(seconds: i64) -> Result<OffsetDateTime, Box<dyn Error>> {
        Ok(OffsetDateTime::from_unix_timestamp(seconds)?)
    }

    fn max_amount(amount: u64) -> BTreeMap<String, ConstraintValue> {
        BTreeMap::from([(
            "max_amount_usd".to_owned(),
            ConstraintValue::Integer(amount),
        )])
    }

    #[test]
    fn human_mints_signed_root_capability_for_agent() -> Result<(), Box<dyn Error>> {
        let human = Signer::generate(SignerKind::Human);
        let agent = Signer::generate(SignerKind::Agent);

        let capability = mint_capability(
            &human,
            CapabilityInput {
                subject: agent.id.clone(),
                resource: "travel.booking".to_owned(),
                operations: vec!["purchase".to_owned()],
                constraints: max_amount(1_200),
                expires_at: expires_at(1_800_000_000)?,
                delegable: true,
            },
        )?;

        assert_eq!(capability.issuer, human.id);
        assert_eq!(capability.subject, agent.id);
        assert_eq!(capability.operations, vec!["purchase"]);
        assert!(capability.parent_id.is_none());
        assert_eq!(capability.proof.signature_hex.len(), 128);
        Ok(())
    }

    #[test]
    fn agent_delegates_narrower_capability_to_subagent() -> Result<(), Box<dyn Error>> {
        let human = Signer::generate(SignerKind::Human);
        let agent = Signer::generate(SignerKind::Agent);
        let subagent = Signer::generate(SignerKind::Agent);
        let root = mint_capability(
            &human,
            CapabilityInput {
                subject: agent.id.clone(),
                resource: "travel.booking".to_owned(),
                operations: vec!["search".to_owned(), "purchase".to_owned()],
                constraints: max_amount(1_200),
                expires_at: expires_at(1_800_000_000)?,
                delegable: true,
            },
        )?;

        let delegated = delegate_capability(
            &agent,
            &root,
            DelegationInput {
                subject: subagent.id.clone(),
                operations: vec!["purchase".to_owned()],
                constraints: max_amount(800),
                expires_at: expires_at(1_700_000_000)?,
                delegable: false,
            },
        )?;

        assert_eq!(delegated.parent_id, Some(root.id));
        assert_eq!(delegated.issuer, agent.id);
        assert_eq!(delegated.subject, subagent.id);
        assert_eq!(delegated.constraints, max_amount(800));
        Ok(())
    }

    #[test]
    fn delegation_rejects_operation_not_granted_by_parent() -> Result<(), Box<dyn Error>> {
        let human = Signer::generate(SignerKind::Human);
        let agent = Signer::generate(SignerKind::Agent);
        let subagent = Signer::generate(SignerKind::Agent);
        let root = mint_capability(
            &human,
            CapabilityInput {
                subject: agent.id.clone(),
                resource: "travel.booking".to_owned(),
                operations: vec!["search".to_owned()],
                constraints: max_amount(1_200),
                expires_at: expires_at(1_800_000_000)?,
                delegable: true,
            },
        )?;

        let result = delegate_capability(
            &agent,
            &root,
            DelegationInput {
                subject: subagent.id,
                operations: vec!["purchase".to_owned()],
                constraints: max_amount(800),
                expires_at: expires_at(1_700_000_000)?,
                delegable: false,
            },
        );

        assert!(matches!(
            result,
            Err(CapabilityError::OperationNotAllowed { .. })
        ));
        Ok(())
    }

    #[test]
    fn delegation_rejects_expiry_after_parent() -> Result<(), Box<dyn Error>> {
        let human = Signer::generate(SignerKind::Human);
        let agent = Signer::generate(SignerKind::Agent);
        let subagent = Signer::generate(SignerKind::Agent);
        let root = mint_capability(
            &human,
            CapabilityInput {
                subject: agent.id.clone(),
                resource: "travel.booking".to_owned(),
                operations: vec!["purchase".to_owned()],
                constraints: max_amount(1_200),
                expires_at: expires_at(1_700_000_000)?,
                delegable: true,
            },
        )?;

        let result = delegate_capability(
            &agent,
            &root,
            DelegationInput {
                subject: subagent.id,
                operations: vec!["purchase".to_owned()],
                constraints: max_amount(800),
                expires_at: expires_at(1_800_000_000)?,
                delegable: false,
            },
        );

        assert!(matches!(result, Err(CapabilityError::ExpiryOutlivesParent)));
        Ok(())
    }

    #[test]
    fn delegation_rejects_constraint_expansion() -> Result<(), Box<dyn Error>> {
        let human = Signer::generate(SignerKind::Human);
        let agent = Signer::generate(SignerKind::Agent);
        let subagent = Signer::generate(SignerKind::Agent);
        let root = mint_capability(
            &human,
            CapabilityInput {
                subject: agent.id.clone(),
                resource: "travel.booking".to_owned(),
                operations: vec!["purchase".to_owned()],
                constraints: max_amount(1_200),
                expires_at: expires_at(1_800_000_000)?,
                delegable: true,
            },
        )?;

        let result = delegate_capability(
            &agent,
            &root,
            DelegationInput {
                subject: subagent.id,
                operations: vec!["purchase".to_owned()],
                constraints: max_amount(1_500),
                expires_at: expires_at(1_700_000_000)?,
                delegable: false,
            },
        );

        assert!(matches!(
            result,
            Err(CapabilityError::ConstraintExpansion { .. })
        ));
        Ok(())
    }

    #[test]
    fn delegation_rejects_constraint_removal() -> Result<(), Box<dyn Error>> {
        let human = Signer::generate(SignerKind::Human);
        let agent = Signer::generate(SignerKind::Agent);
        let subagent = Signer::generate(SignerKind::Agent);
        let root = mint_capability(
            &human,
            CapabilityInput {
                subject: agent.id.clone(),
                resource: "travel.booking".to_owned(),
                operations: vec!["purchase".to_owned()],
                constraints: max_amount(1_200),
                expires_at: expires_at(1_800_000_000)?,
                delegable: true,
            },
        )?;

        let result = delegate_capability(
            &agent,
            &root,
            DelegationInput {
                subject: subagent.id,
                operations: vec!["purchase".to_owned()],
                constraints: BTreeMap::new(),
                expires_at: expires_at(1_700_000_000)?,
                delegable: false,
            },
        );

        assert!(matches!(
            result,
            Err(CapabilityError::ConstraintRemoval { .. })
        ));
        Ok(())
    }

    #[test]
    fn mint_rejects_empty_operations() -> Result<(), Box<dyn Error>> {
        let human = Signer::generate(SignerKind::Human);
        let agent = Signer::generate(SignerKind::Agent);

        let result = mint_capability(
            &human,
            CapabilityInput {
                subject: agent.id,
                resource: "travel.booking".to_owned(),
                operations: vec![],
                constraints: max_amount(1_200),
                expires_at: expires_at(1_800_000_000)?,
                delegable: true,
            },
        );

        assert!(matches!(result, Err(CapabilityError::OperationsEmpty)));
        Ok(())
    }

    #[test]
    fn delegation_rejects_empty_operations() -> Result<(), Box<dyn Error>> {
        let human = Signer::generate(SignerKind::Human);
        let agent = Signer::generate(SignerKind::Agent);
        let subagent = Signer::generate(SignerKind::Agent);
        let root = mint_capability(
            &human,
            CapabilityInput {
                subject: agent.id.clone(),
                resource: "travel.booking".to_owned(),
                operations: vec!["purchase".to_owned()],
                constraints: max_amount(1_200),
                expires_at: expires_at(1_800_000_000)?,
                delegable: true,
            },
        )?;

        let result = delegate_capability(
            &agent,
            &root,
            DelegationInput {
                subject: subagent.id,
                operations: vec![],
                constraints: max_amount(800),
                expires_at: expires_at(1_700_000_000)?,
                delegable: false,
            },
        );

        assert!(matches!(result, Err(CapabilityError::OperationsEmpty)));
        Ok(())
    }
}
