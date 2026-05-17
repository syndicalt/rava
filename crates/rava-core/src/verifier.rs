use std::collections::BTreeMap;

use time::OffsetDateTime;

use crate::action::{action_signing_payload, expected_action_id, ActionEnvelope};
use crate::capability::{capability_signing_payload, expected_capability_id, Capability};
use crate::constraints::{action_constraint_violation, value_is_no_broader_than};
use crate::error::RavaError;
use crate::hash::is_sha256_hash;
use crate::identity::verify_json_signature;
use crate::nonce::is_canonical_uuid_v4;
use crate::protocol::{is_supported_action_version, is_supported_capability_version};
use crate::replay::{ReplayRegistry, ReplayStoreError};
use crate::revocation::RevocationRegistry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationResult {
    Accepted,
    Rejected(VerificationError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationError {
    UnsupportedActionVersion {
        version: String,
    },
    UnsupportedCapabilityVersion {
        version: String,
    },
    ActionNonceInvalid,
    CapabilityNonceInvalid {
        capability_id: String,
    },
    ActionContextHashInvalid,
    ActionSignatureInvalid,
    ActionIdMismatch,
    ActionReplayed {
        action_id: String,
    },
    CapabilityChainEmpty,
    ActionCapabilityNotFinal,
    RootIssuerNotController,
    CapabilityIdMismatch {
        capability_id: String,
    },
    CapabilityOperationsEmpty {
        capability_id: String,
    },
    CapabilityOperationsNotCanonical {
        capability_id: String,
    },
    CapabilitySignatureInvalid {
        capability_id: String,
    },
    CapabilityRevoked {
        capability_id: String,
    },
    SignerRevoked {
        signer_id: String,
    },
    CapabilityExpired {
        capability_id: String,
    },
    CapabilityParentMismatch {
        capability_id: String,
    },
    CapabilityIssuerNotParentSubject {
        capability_id: String,
    },
    CapabilityResourceMismatch {
        capability_id: String,
    },
    CapabilityOperationNotGranted {
        capability_id: String,
        operation: String,
    },
    CapabilityExpiryOutlivesParent {
        capability_id: String,
    },
    CapabilityConstraintRemoved {
        capability_id: String,
        constraint: String,
    },
    CapabilityConstraintExpanded {
        capability_id: String,
        constraint: String,
    },
    ParentCapabilityNotDelegable {
        capability_id: String,
    },
    FinalSubjectNotActor,
    ResourceMismatch,
    OperationNotAllowed,
    ConstraintExceeded {
        constraint: String,
    },
    MissingIssuerPublicKey {
        issuer: String,
    },
}

impl VerificationError {
    pub fn code(&self) -> &'static str {
        match self {
            VerificationError::UnsupportedActionVersion { .. } => "unsupported_action_version",
            VerificationError::UnsupportedCapabilityVersion { .. } => {
                "unsupported_capability_version"
            }
            VerificationError::ActionNonceInvalid => "action_nonce_invalid",
            VerificationError::CapabilityNonceInvalid { .. } => "capability_nonce_invalid",
            VerificationError::ActionContextHashInvalid => "action_context_hash_invalid",
            VerificationError::ActionSignatureInvalid => "action_signature_invalid",
            VerificationError::ActionIdMismatch => "action_id_mismatch",
            VerificationError::ActionReplayed { .. } => "action_replayed",
            VerificationError::CapabilityChainEmpty => "capability_chain_empty",
            VerificationError::ActionCapabilityNotFinal => "action_capability_not_final",
            VerificationError::RootIssuerNotController => "root_issuer_not_controller",
            VerificationError::CapabilityIdMismatch { .. } => "capability_id_mismatch",
            VerificationError::CapabilityOperationsEmpty { .. } => "capability_operations_empty",
            VerificationError::CapabilityOperationsNotCanonical { .. } => {
                "capability_operations_not_canonical"
            }
            VerificationError::CapabilitySignatureInvalid { .. } => "capability_signature_invalid",
            VerificationError::CapabilityRevoked { .. } => "capability_revoked",
            VerificationError::SignerRevoked { .. } => "signer_revoked",
            VerificationError::CapabilityExpired { .. } => "capability_expired",
            VerificationError::CapabilityParentMismatch { .. } => "capability_parent_mismatch",
            VerificationError::CapabilityIssuerNotParentSubject { .. } => {
                "capability_issuer_not_parent_subject"
            }
            VerificationError::CapabilityResourceMismatch { .. } => "capability_resource_mismatch",
            VerificationError::CapabilityOperationNotGranted { .. } => {
                "capability_operation_not_granted"
            }
            VerificationError::CapabilityExpiryOutlivesParent { .. } => {
                "capability_expiry_outlives_parent"
            }
            VerificationError::CapabilityConstraintRemoved { .. } => {
                "capability_constraint_removed"
            }
            VerificationError::CapabilityConstraintExpanded { .. } => {
                "capability_constraint_expanded"
            }
            VerificationError::ParentCapabilityNotDelegable { .. } => {
                "parent_capability_not_delegable"
            }
            VerificationError::FinalSubjectNotActor => "final_subject_not_actor",
            VerificationError::ResourceMismatch => "resource_mismatch",
            VerificationError::OperationNotAllowed => "operation_not_allowed",
            VerificationError::ConstraintExceeded { .. } => "constraint_exceeded",
            VerificationError::MissingIssuerPublicKey { .. } => "missing_issuer_public_key",
        }
    }

    pub fn subject(&self) -> Option<String> {
        match self {
            VerificationError::UnsupportedActionVersion { version }
            | VerificationError::UnsupportedCapabilityVersion { version } => Some(version.clone()),
            VerificationError::CapabilityNonceInvalid { capability_id }
            | VerificationError::CapabilityIdMismatch { capability_id }
            | VerificationError::CapabilityOperationsEmpty { capability_id }
            | VerificationError::CapabilityOperationsNotCanonical { capability_id }
            | VerificationError::CapabilitySignatureInvalid { capability_id }
            | VerificationError::CapabilityRevoked { capability_id }
            | VerificationError::CapabilityExpired { capability_id }
            | VerificationError::CapabilityParentMismatch { capability_id }
            | VerificationError::CapabilityIssuerNotParentSubject { capability_id }
            | VerificationError::CapabilityResourceMismatch { capability_id }
            | VerificationError::CapabilityExpiryOutlivesParent { capability_id }
            | VerificationError::ParentCapabilityNotDelegable { capability_id } => {
                Some(capability_id.clone())
            }
            VerificationError::ActionReplayed { action_id } => Some(action_id.clone()),
            VerificationError::SignerRevoked { signer_id } => Some(signer_id.clone()),
            VerificationError::CapabilityOperationNotGranted {
                capability_id,
                operation,
            } => Some(format!("{capability_id}:{operation}")),
            VerificationError::CapabilityConstraintRemoved {
                capability_id,
                constraint,
            }
            | VerificationError::CapabilityConstraintExpanded {
                capability_id,
                constraint,
            } => Some(format!("{capability_id}:{constraint}")),
            VerificationError::ConstraintExceeded { constraint } => Some(constraint.clone()),
            VerificationError::MissingIssuerPublicKey { issuer } => Some(issuer.clone()),
            VerificationError::ActionNonceInvalid
            | VerificationError::ActionContextHashInvalid
            | VerificationError::ActionSignatureInvalid
            | VerificationError::ActionIdMismatch
            | VerificationError::CapabilityChainEmpty
            | VerificationError::ActionCapabilityNotFinal
            | VerificationError::RootIssuerNotController
            | VerificationError::FinalSubjectNotActor
            | VerificationError::ResourceMismatch
            | VerificationError::OperationNotAllowed => None,
        }
    }
}

#[derive(Debug)]
pub struct VerifyActionInput<'a, R: RevocationRegistry> {
    pub action: &'a ActionEnvelope,
    pub capability_chain: &'a [Capability],
    pub actor_public_key_hex: &'a str,
    pub capability_issuer_public_keys: &'a BTreeMap<String, String>,
    pub revocations: &'a R,
    pub now: OffsetDateTime,
}

#[derive(Debug)]
pub struct VerifyActionOnceInput<'a, R: RevocationRegistry, P: ReplayRegistry> {
    pub action: &'a ActionEnvelope,
    pub capability_chain: &'a [Capability],
    pub actor_public_key_hex: &'a str,
    pub capability_issuer_public_keys: &'a BTreeMap<String, String>,
    pub revocations: &'a R,
    pub replay: &'a mut P,
    pub now: OffsetDateTime,
}

#[derive(Debug, thiserror::Error)]
pub enum VerifyActionOnceError {
    #[error(transparent)]
    Rava(#[from] RavaError),

    #[error(transparent)]
    Replay(#[from] ReplayStoreError),
}

pub fn verify_action<R: RevocationRegistry>(
    input: VerifyActionInput<'_, R>,
) -> Result<VerificationResult, RavaError> {
    if !is_supported_action_version(&input.action.version) {
        return Ok(VerificationResult::Rejected(
            VerificationError::UnsupportedActionVersion {
                version: input.action.version.clone(),
            },
        ));
    }
    if !is_sha256_hash(&input.action.context_hash) {
        return Ok(VerificationResult::Rejected(
            VerificationError::ActionContextHashInvalid,
        ));
    }
    if !is_canonical_uuid_v4(&input.action.nonce) {
        return Ok(VerificationResult::Rejected(
            VerificationError::ActionNonceInvalid,
        ));
    }

    let action_signature_valid = verify_json_signature(
        &input.action.actor,
        input.actor_public_key_hex,
        &action_signing_payload(input.action),
        &input.action.proof.signature_hex,
    )?;
    if !action_signature_valid {
        return Ok(VerificationResult::Rejected(
            VerificationError::ActionSignatureInvalid,
        ));
    }
    if input.revocations.is_revoked(&input.action.actor) {
        return Ok(VerificationResult::Rejected(
            VerificationError::SignerRevoked {
                signer_id: input.action.actor.clone(),
            },
        ));
    }
    if expected_action_id(input.action)? != input.action.id {
        return Ok(VerificationResult::Rejected(
            VerificationError::ActionIdMismatch,
        ));
    }

    let Some(final_capability) = input.capability_chain.last() else {
        return Ok(VerificationResult::Rejected(
            VerificationError::CapabilityChainEmpty,
        ));
    };
    if input.capability_chain[0].issuer != input.action.controller {
        return Ok(VerificationResult::Rejected(
            VerificationError::RootIssuerNotController,
        ));
    }
    if final_capability.id != input.action.capability_id {
        return Ok(VerificationResult::Rejected(
            VerificationError::ActionCapabilityNotFinal,
        ));
    }

    for (index, capability) in input.capability_chain.iter().enumerate() {
        if !is_supported_capability_version(&capability.version) {
            return Ok(VerificationResult::Rejected(
                VerificationError::UnsupportedCapabilityVersion {
                    version: capability.version.clone(),
                },
            ));
        }
        if !is_canonical_uuid_v4(&capability.nonce) {
            return Ok(VerificationResult::Rejected(
                VerificationError::CapabilityNonceInvalid {
                    capability_id: capability.id.clone(),
                },
            ));
        }
        if capability.operations.is_empty() {
            return Ok(VerificationResult::Rejected(
                VerificationError::CapabilityOperationsEmpty {
                    capability_id: capability.id.clone(),
                },
            ));
        }
        if !operations_are_canonical(&capability.operations) {
            return Ok(VerificationResult::Rejected(
                VerificationError::CapabilityOperationsNotCanonical {
                    capability_id: capability.id.clone(),
                },
            ));
        }

        if expected_capability_id(capability)? != capability.id {
            return Ok(VerificationResult::Rejected(
                VerificationError::CapabilityIdMismatch {
                    capability_id: capability.id.clone(),
                },
            ));
        }

        let Some(issuer_public_key) = input.capability_issuer_public_keys.get(&capability.issuer)
        else {
            return Ok(VerificationResult::Rejected(
                VerificationError::MissingIssuerPublicKey {
                    issuer: capability.issuer.clone(),
                },
            ));
        };

        let capability_signature_valid = verify_json_signature(
            &capability.issuer,
            issuer_public_key,
            &capability_signing_payload(capability),
            &capability.proof.signature_hex,
        )?;
        if !capability_signature_valid {
            return Ok(VerificationResult::Rejected(
                VerificationError::CapabilitySignatureInvalid {
                    capability_id: capability.id.clone(),
                },
            ));
        }

        if input.revocations.is_revoked(&capability.id) {
            return Ok(VerificationResult::Rejected(
                VerificationError::CapabilityRevoked {
                    capability_id: capability.id.clone(),
                },
            ));
        }
        if input.revocations.is_revoked(&capability.issuer) {
            return Ok(VerificationResult::Rejected(
                VerificationError::SignerRevoked {
                    signer_id: capability.issuer.clone(),
                },
            ));
        }

        if capability.expires_at <= input.now {
            return Ok(VerificationResult::Rejected(
                VerificationError::CapabilityExpired {
                    capability_id: capability.id.clone(),
                },
            ));
        }

        if index == 0 {
            if capability.parent_id.is_some() {
                return Ok(VerificationResult::Rejected(
                    VerificationError::CapabilityParentMismatch {
                        capability_id: capability.id.clone(),
                    },
                ));
            }
            continue;
        }

        let parent = &input.capability_chain[index - 1];
        if capability.parent_id.as_deref() != Some(parent.id.as_str()) {
            return Ok(VerificationResult::Rejected(
                VerificationError::CapabilityParentMismatch {
                    capability_id: capability.id.clone(),
                },
            ));
        }
        if capability.issuer != parent.subject {
            return Ok(VerificationResult::Rejected(
                VerificationError::CapabilityIssuerNotParentSubject {
                    capability_id: capability.id.clone(),
                },
            ));
        }
        if !parent.delegable {
            return Ok(VerificationResult::Rejected(
                VerificationError::ParentCapabilityNotDelegable {
                    capability_id: parent.id.clone(),
                },
            ));
        }
        if capability.resource != parent.resource {
            return Ok(VerificationResult::Rejected(
                VerificationError::CapabilityResourceMismatch {
                    capability_id: capability.id.clone(),
                },
            ));
        }
        if let Some(operation) = capability
            .operations
            .iter()
            .find(|operation| !parent.operations.contains(operation))
        {
            return Ok(VerificationResult::Rejected(
                VerificationError::CapabilityOperationNotGranted {
                    capability_id: capability.id.clone(),
                    operation: operation.clone(),
                },
            ));
        }
        if capability.expires_at > parent.expires_at {
            return Ok(VerificationResult::Rejected(
                VerificationError::CapabilityExpiryOutlivesParent {
                    capability_id: capability.id.clone(),
                },
            ));
        }
        if let Some(constraint) = first_removed_constraint(parent, capability) {
            return Ok(VerificationResult::Rejected(
                VerificationError::CapabilityConstraintRemoved {
                    capability_id: capability.id.clone(),
                    constraint,
                },
            ));
        }
        if let Some(constraint) = first_expanded_constraint(parent, capability) {
            return Ok(VerificationResult::Rejected(
                VerificationError::CapabilityConstraintExpanded {
                    capability_id: capability.id.clone(),
                    constraint,
                },
            ));
        }
    }

    if final_capability.subject != input.action.actor {
        return Ok(VerificationResult::Rejected(
            VerificationError::FinalSubjectNotActor,
        ));
    }
    if final_capability.resource != input.action.resource {
        return Ok(VerificationResult::Rejected(
            VerificationError::ResourceMismatch,
        ));
    }
    if !final_capability
        .operations
        .contains(&input.action.operation)
    {
        return Ok(VerificationResult::Rejected(
            VerificationError::OperationNotAllowed,
        ));
    }
    if let Some(constraint) = first_exceeded_constraint(input.action, final_capability) {
        return Ok(VerificationResult::Rejected(
            VerificationError::ConstraintExceeded { constraint },
        ));
    }

    Ok(VerificationResult::Accepted)
}

pub fn verify_action_once<R: RevocationRegistry, P: ReplayRegistry>(
    input: VerifyActionOnceInput<'_, R, P>,
) -> Result<VerificationResult, VerifyActionOnceError> {
    if input.replay.has_seen(&input.action.id) {
        return Ok(VerificationResult::Rejected(
            VerificationError::ActionReplayed {
                action_id: input.action.id.clone(),
            },
        ));
    }

    let result = verify_action(VerifyActionInput {
        action: input.action,
        capability_chain: input.capability_chain,
        actor_public_key_hex: input.actor_public_key_hex,
        capability_issuer_public_keys: input.capability_issuer_public_keys,
        revocations: input.revocations,
        now: input.now,
    })?;

    if result == VerificationResult::Accepted {
        input.replay.record(input.action.id.clone())?;
    }

    Ok(result)
}

fn first_exceeded_constraint(action: &ActionEnvelope, capability: &Capability) -> Option<String> {
    for (key, action_value) in &action.constraints {
        if let Some(constraint) =
            action_constraint_violation(key, action_value, &capability.constraints)
        {
            return Some(constraint);
        }
    }

    None
}

fn first_removed_constraint(parent: &Capability, child: &Capability) -> Option<String> {
    parent
        .constraints
        .keys()
        .find(|key| !child.constraints.contains_key(*key))
        .cloned()
}

fn first_expanded_constraint(parent: &Capability, child: &Capability) -> Option<String> {
    for (key, child_value) in &child.constraints {
        let Some(parent_value) = parent.constraints.get(key) else {
            return Some(key.clone());
        };

        if !value_is_no_broader_than(child_value, parent_value) {
            return Some(key.clone());
        }
    }

    None
}

fn operations_are_canonical(operations: &[String]) -> bool {
    operations.windows(2).all(|pair| pair[0] < pair[1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{action_signing_payload, expected_action_id, sign_action, ActionInput};
    use crate::capability::{
        capability_signing_payload, delegate_capability, expected_capability_id, mint_capability,
        CapabilityInput, ConstraintValue, DelegationInput,
    };
    use crate::identity::{Signer, SignerKind};
    use crate::revocation::InMemoryRevocationRegistry;
    use std::collections::BTreeMap;
    use std::error::Error;
    use time::OffsetDateTime;

    struct Scenario {
        human: Signer,
        personal_agent: Signer,
        booking_agent: Signer,
        root: crate::capability::Capability,
        purchase: crate::capability::Capability,
    }

    fn at(seconds: i64) -> Result<OffsetDateTime, Box<dyn Error>> {
        Ok(OffsetDateTime::from_unix_timestamp(seconds)?)
    }

    fn max_amount(amount: u64) -> BTreeMap<String, ConstraintValue> {
        BTreeMap::from([(
            "max_amount_usd".to_owned(),
            ConstraintValue::Integer(amount),
        )])
    }

    fn amount(amount: u64) -> BTreeMap<String, ConstraintValue> {
        BTreeMap::from([("amount_usd".to_owned(), ConstraintValue::Integer(amount))])
    }

    fn scenario() -> Result<Scenario, Box<dyn Error>> {
        let human = Signer::generate(SignerKind::Human);
        let personal_agent = Signer::generate(SignerKind::Agent);
        let booking_agent = Signer::generate(SignerKind::Agent);
        let root = mint_capability(
            &human,
            CapabilityInput {
                subject: personal_agent.id.clone(),
                resource: "travel.booking".to_owned(),
                operations: vec!["purchase".to_owned()],
                constraints: max_amount(1_200),
                expires_at: at(1_800_000_000)?,
                delegable: true,
            },
        )?;
        let purchase = delegate_capability(
            &personal_agent,
            &root,
            DelegationInput {
                subject: booking_agent.id.clone(),
                operations: vec!["purchase".to_owned()],
                constraints: max_amount(800),
                expires_at: at(1_700_000_000)?,
                delegable: false,
            },
        )?;

        Ok(Scenario {
            human,
            personal_agent,
            booking_agent,
            root,
            purchase,
        })
    }

    fn purchase_action(
        scenario: &Scenario,
        amount_usd: u64,
    ) -> Result<crate::action::ActionEnvelope, Box<dyn Error>> {
        Ok(sign_action(
            &scenario.booking_agent,
            ActionInput {
                controller: scenario.human.id.clone(),
                intent: "book_flight".to_owned(),
                resource: "travel.booking".to_owned(),
                operation: "purchase".to_owned(),
                constraints: amount(amount_usd),
                capability_id: scenario.purchase.id.clone(),
                context_hash:
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_owned(),
            },
        )?)
    }

    fn action_for_capability(
        scenario: &Scenario,
        capability: &Capability,
        operation: &str,
        amount_usd: u64,
    ) -> Result<crate::action::ActionEnvelope, Box<dyn Error>> {
        Ok(sign_action(
            &scenario.booking_agent,
            ActionInput {
                controller: scenario.human.id.clone(),
                intent: "book_flight".to_owned(),
                resource: capability.resource.clone(),
                operation: operation.to_owned(),
                constraints: amount(amount_usd),
                capability_id: capability.id.clone(),
                context_hash:
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_owned(),
            },
        )?)
    }

    fn resign_capability(
        signer: &Signer,
        capability: &mut Capability,
    ) -> Result<(), Box<dyn Error>> {
        capability.id = expected_capability_id(capability)?;
        capability.proof.signature_hex =
            signer.sign_json(&capability_signing_payload(capability))?;
        Ok(())
    }

    #[test]
    fn accepts_valid_delegated_purchase_action() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let action = purchase_action(&scenario, 750)?;
        let issuer_keys = BTreeMap::from([
            (
                scenario.root.issuer.clone(),
                scenario.human.public_key_hex.clone(),
            ),
            (
                scenario.purchase.issuer.clone(),
                scenario.personal_agent.public_key_hex.clone(),
            ),
        ]);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, scenario.purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(result, VerificationResult::Accepted);
        Ok(())
    }

    #[test]
    fn verification_errors_expose_stable_codes_and_subjects() {
        let capability_error = VerificationError::CapabilityConstraintExpanded {
            capability_id: "cap_demo".to_owned(),
            constraint: "max_amount_usd".to_owned(),
        };
        assert_eq!(capability_error.code(), "capability_constraint_expanded");
        assert_eq!(
            capability_error.subject(),
            Some("cap_demo:max_amount_usd".to_owned())
        );

        let action_error = VerificationError::ConstraintExceeded {
            constraint: "max_amount_usd".to_owned(),
        };
        assert_eq!(action_error.code(), "constraint_exceeded");
        assert_eq!(action_error.subject(), Some("max_amount_usd".to_owned()));

        let context_error = VerificationError::ActionContextHashInvalid;
        assert_eq!(context_error.code(), "action_context_hash_invalid");
        assert_eq!(context_error.subject(), None);
    }

    #[test]
    fn rejects_signed_action_with_invalid_nonce() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let mut action = purchase_action(&scenario, 750)?;
        action.nonce = "not-a-uuid".to_owned();
        action.id = expected_action_id(&action)?;
        action.proof.signature_hex = scenario
            .booking_agent
            .sign_json(&action_signing_payload(&action))?;
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, scenario.purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::ActionNonceInvalid)
        );
        Ok(())
    }

    #[test]
    fn rejects_signed_capability_with_invalid_nonce() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let mut purchase = scenario.purchase.clone();
        purchase.nonce = "not-a-uuid".to_owned();
        purchase.id = expected_capability_id(&purchase)?;
        purchase.proof.signature_hex = scenario
            .personal_agent
            .sign_json(&capability_signing_payload(&purchase))?;
        let action = sign_action(
            &scenario.booking_agent,
            ActionInput {
                controller: scenario.human.id.clone(),
                intent: "book_flight".to_owned(),
                resource: "travel.booking".to_owned(),
                operation: "purchase".to_owned(),
                constraints: amount(750),
                capability_id: purchase.id.clone(),
                context_hash:
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_owned(),
            },
        )?;
        let issuer_keys = BTreeMap::from([
            (
                scenario.root.issuer.clone(),
                scenario.human.public_key_hex.clone(),
            ),
            (
                purchase.issuer.clone(),
                scenario.personal_agent.public_key_hex.clone(),
            ),
        ]);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, purchase.clone()],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::CapabilityNonceInvalid {
                capability_id: purchase.id
            })
        );
        Ok(())
    }

    #[test]
    fn rejects_purchase_above_delegated_amount() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let action = purchase_action(&scenario, 900)?;
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, scenario.purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::ConstraintExceeded {
                constraint: "max_amount_usd".to_owned()
            })
        );
        Ok(())
    }

    #[test]
    fn rejects_action_constraint_not_granted_by_capability() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let mut constraints = amount(750);
        constraints.insert(
            "external_transfer_allowed".to_owned(),
            ConstraintValue::Bool(true),
        );
        let action = sign_action(
            &scenario.booking_agent,
            ActionInput {
                controller: scenario.human.id.clone(),
                intent: "book_flight".to_owned(),
                resource: "travel.booking".to_owned(),
                operation: "purchase".to_owned(),
                constraints,
                capability_id: scenario.purchase.id.clone(),
                context_hash:
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_owned(),
            },
        )?;
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, scenario.purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::ConstraintExceeded {
                constraint: "external_transfer_allowed".to_owned()
            })
        );
        Ok(())
    }

    #[test]
    fn rejects_amount_constraint_without_capability_limit() -> Result<(), Box<dyn Error>> {
        let human = Signer::generate(SignerKind::Human);
        let agent = Signer::generate(SignerKind::Agent);
        let booking_agent = Signer::generate(SignerKind::Agent);
        let root = mint_capability(
            &human,
            CapabilityInput {
                subject: agent.id.clone(),
                resource: "travel.booking".to_owned(),
                operations: vec!["purchase".to_owned()],
                constraints: BTreeMap::new(),
                expires_at: at(1_800_000_000)?,
                delegable: true,
            },
        )?;
        let purchase = delegate_capability(
            &agent,
            &root,
            DelegationInput {
                subject: booking_agent.id.clone(),
                operations: vec!["purchase".to_owned()],
                constraints: BTreeMap::new(),
                expires_at: at(1_700_000_000)?,
                delegable: false,
            },
        )?;
        let action = sign_action(
            &booking_agent,
            ActionInput {
                controller: human.id.clone(),
                intent: "book_flight".to_owned(),
                resource: "travel.booking".to_owned(),
                operation: "purchase".to_owned(),
                constraints: amount(750),
                capability_id: purchase.id.clone(),
                context_hash:
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_owned(),
            },
        )?;
        let issuer_keys = BTreeMap::from([
            (root.issuer.clone(), human.public_key_hex.clone()),
            (purchase.issuer.clone(), agent.public_key_hex.clone()),
        ]);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[root, purchase],
            actor_public_key_hex: &booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::ConstraintExceeded {
                constraint: "amount_usd".to_owned()
            })
        );
        Ok(())
    }

    #[test]
    fn rejects_revoked_capability() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let action = purchase_action(&scenario, 750)?;
        let issuer_keys = issuer_keys(&scenario);
        let mut revocations = InMemoryRevocationRegistry::default();
        revocations.revoke(scenario.purchase.id.clone());

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, scenario.purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert!(matches!(
            result,
            VerificationResult::Rejected(VerificationError::CapabilityRevoked { .. })
        ));
        Ok(())
    }

    #[test]
    fn rejects_revoked_action_actor() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let action = purchase_action(&scenario, 750)?;
        let issuer_keys = issuer_keys(&scenario);
        let mut revocations = InMemoryRevocationRegistry::default();
        revocations.revoke(scenario.booking_agent.id.clone());

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, scenario.purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::SignerRevoked {
                signer_id: scenario.booking_agent.id
            })
        );
        Ok(())
    }

    #[test]
    fn rejects_revoked_capability_issuer() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let action = purchase_action(&scenario, 750)?;
        let issuer_keys = issuer_keys(&scenario);
        let mut revocations = InMemoryRevocationRegistry::default();
        revocations.revoke(scenario.personal_agent.id.clone());

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, scenario.purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::SignerRevoked {
                signer_id: scenario.personal_agent.id
            })
        );
        Ok(())
    }

    #[test]
    fn rejects_expired_capability() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let action = purchase_action(&scenario, 750)?;
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, scenario.purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_750_000_000)?,
        })?;

        assert!(matches!(
            result,
            VerificationResult::Rejected(VerificationError::CapabilityExpired { .. })
        ));
        Ok(())
    }

    #[test]
    fn rejects_invalid_parent_chain() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();
        let mut tampered_purchase = scenario.purchase;
        tampered_purchase.parent_id = Some("cap_not_the_parent".to_owned());
        tampered_purchase.id = expected_capability_id(&tampered_purchase)?;
        tampered_purchase.proof.signature_hex = scenario
            .personal_agent
            .sign_json(&capability_signing_payload(&tampered_purchase))?;
        let action = sign_action(
            &scenario.booking_agent,
            ActionInput {
                controller: scenario.human.id.clone(),
                intent: "book_flight".to_owned(),
                resource: "travel.booking".to_owned(),
                operation: "purchase".to_owned(),
                constraints: amount(750),
                capability_id: tampered_purchase.id.clone(),
                context_hash:
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_owned(),
            },
        )?;

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, tampered_purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert!(matches!(
            result,
            VerificationResult::Rejected(VerificationError::CapabilitySignatureInvalid { .. })
                | VerificationResult::Rejected(VerificationError::CapabilityParentMismatch { .. })
        ));
        Ok(())
    }

    #[test]
    fn rejects_child_capability_resource_not_granted_by_parent() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();
        let mut purchase = scenario.purchase.clone();
        purchase.resource = "payments.transfer".to_owned();
        resign_capability(&scenario.personal_agent, &mut purchase)?;
        let action = action_for_capability(&scenario, &purchase, "purchase", 750)?;

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, purchase.clone()],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::CapabilityResourceMismatch {
                capability_id: purchase.id
            })
        );
        Ok(())
    }

    #[test]
    fn rejects_child_capability_operation_not_granted_by_parent() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();
        let mut purchase = scenario.purchase.clone();
        purchase.operations = vec!["refund".to_owned()];
        resign_capability(&scenario.personal_agent, &mut purchase)?;
        let action = action_for_capability(&scenario, &purchase, "refund", 750)?;

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, purchase.clone()],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::CapabilityOperationNotGranted {
                capability_id: purchase.id,
                operation: "refund".to_owned()
            })
        );
        Ok(())
    }

    #[test]
    fn rejects_child_capability_expiry_after_parent() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();
        let mut purchase = scenario.purchase.clone();
        purchase.expires_at = at(1_900_000_000)?;
        resign_capability(&scenario.personal_agent, &mut purchase)?;
        let action = action_for_capability(&scenario, &purchase, "purchase", 750)?;

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, purchase.clone()],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::CapabilityExpiryOutlivesParent {
                capability_id: purchase.id
            })
        );
        Ok(())
    }

    #[test]
    fn rejects_child_capability_constraint_expansion() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();
        let mut purchase = scenario.purchase.clone();
        purchase.constraints = max_amount(5_000);
        resign_capability(&scenario.personal_agent, &mut purchase)?;
        let action = action_for_capability(&scenario, &purchase, "purchase", 3_000)?;

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, purchase.clone()],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::CapabilityConstraintExpanded {
                capability_id: purchase.id,
                constraint: "max_amount_usd".to_owned()
            })
        );
        Ok(())
    }

    #[test]
    fn rejects_child_capability_constraint_removal() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();
        let mut purchase = scenario.purchase.clone();
        purchase.constraints = BTreeMap::new();
        resign_capability(&scenario.personal_agent, &mut purchase)?;
        let action = action_for_capability(&scenario, &purchase, "purchase", 750)?;

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, purchase.clone()],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::CapabilityConstraintRemoved {
                capability_id: purchase.id,
                constraint: "max_amount_usd".to_owned()
            })
        );
        Ok(())
    }

    #[test]
    fn rejects_action_actor_without_final_capability() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let impostor = Signer::generate(SignerKind::Agent);
        let action = sign_action(
            &impostor,
            ActionInput {
                controller: scenario.human.id.clone(),
                intent: "book_flight".to_owned(),
                resource: "travel.booking".to_owned(),
                operation: "purchase".to_owned(),
                constraints: amount(750),
                capability_id: scenario.purchase.id.clone(),
                context_hash:
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_owned(),
            },
        )?;
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, scenario.purchase],
            actor_public_key_hex: &impostor.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::FinalSubjectNotActor)
        );
        Ok(())
    }

    #[test]
    fn rejects_action_controller_that_is_not_root_issuer() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let other_controller = Signer::generate(SignerKind::Human);
        let action = sign_action(
            &scenario.booking_agent,
            ActionInput {
                controller: other_controller.id,
                intent: "book_flight".to_owned(),
                resource: "travel.booking".to_owned(),
                operation: "purchase".to_owned(),
                constraints: amount(750),
                capability_id: scenario.purchase.id.clone(),
                context_hash:
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_owned(),
            },
        )?;
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, scenario.purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::RootIssuerNotController)
        );
        Ok(())
    }

    #[test]
    fn rejects_tampered_action_signature() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let mut action = purchase_action(&scenario, 750)?;
        action.constraints = amount(700);
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, scenario.purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::ActionSignatureInvalid)
        );
        Ok(())
    }

    #[test]
    fn rejects_signed_action_with_non_derived_id() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let mut action = purchase_action(&scenario, 750)?;
        action.id = "act_attacker_chosen".to_owned();
        action.proof.signature_hex = scenario
            .booking_agent
            .sign_json(&action_signing_payload(&action))?;
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, scenario.purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::ActionIdMismatch)
        );
        Ok(())
    }

    #[test]
    fn rejects_signed_action_with_invalid_context_hash() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let mut action = purchase_action(&scenario, 750)?;
        action.context_hash = "sha256:not-hex".to_owned();
        action.id = expected_action_id(&action)?;
        action.proof.signature_hex = scenario
            .booking_agent
            .sign_json(&action_signing_payload(&action))?;
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, scenario.purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::ActionContextHashInvalid)
        );
        Ok(())
    }

    #[test]
    fn rejects_unsupported_action_version() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let mut action = purchase_action(&scenario, 750)?;
        action.version = "rava-action-v999".to_owned();
        action.id = expected_action_id(&action)?;
        action.proof.signature_hex = scenario
            .booking_agent
            .sign_json(&action_signing_payload(&action))?;
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, scenario.purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::UnsupportedActionVersion {
                version: "rava-action-v999".to_owned()
            })
        );
        Ok(())
    }

    #[test]
    fn rejects_signed_capability_with_non_derived_id() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let mut purchase = scenario.purchase;
        purchase.id = "cap_attacker_chosen".to_owned();
        purchase.proof.signature_hex = scenario
            .personal_agent
            .sign_json(&capability_signing_payload(&purchase))?;
        let action = sign_action(
            &scenario.booking_agent,
            ActionInput {
                controller: scenario.human.id.clone(),
                intent: "book_flight".to_owned(),
                resource: "travel.booking".to_owned(),
                operation: "purchase".to_owned(),
                constraints: amount(750),
                capability_id: purchase.id.clone(),
                context_hash:
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_owned(),
            },
        )?;
        let issuer_keys = BTreeMap::from([
            (
                scenario.root.issuer.clone(),
                scenario.human.public_key_hex.clone(),
            ),
            (
                purchase.issuer.clone(),
                scenario.personal_agent.public_key_hex.clone(),
            ),
        ]);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert!(matches!(
            result,
            VerificationResult::Rejected(VerificationError::CapabilityIdMismatch { .. })
        ));
        Ok(())
    }

    #[test]
    fn rejects_unsupported_capability_version() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let mut purchase = scenario.purchase;
        purchase.version = "rava-capability-v999".to_owned();
        purchase.id = expected_capability_id(&purchase)?;
        purchase.proof.signature_hex = scenario
            .personal_agent
            .sign_json(&capability_signing_payload(&purchase))?;
        let action = sign_action(
            &scenario.booking_agent,
            ActionInput {
                controller: scenario.human.id.clone(),
                intent: "book_flight".to_owned(),
                resource: "travel.booking".to_owned(),
                operation: "purchase".to_owned(),
                constraints: amount(750),
                capability_id: purchase.id.clone(),
                context_hash:
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_owned(),
            },
        )?;
        let issuer_keys = BTreeMap::from([
            (
                scenario.root.issuer.clone(),
                scenario.human.public_key_hex.clone(),
            ),
            (
                purchase.issuer.clone(),
                scenario.personal_agent.public_key_hex.clone(),
            ),
        ]);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(
            result,
            VerificationResult::Rejected(VerificationError::UnsupportedCapabilityVersion {
                version: "rava-capability-v999".to_owned()
            })
        );
        Ok(())
    }

    #[test]
    fn rejects_signed_capability_with_duplicate_operations() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let mut purchase = scenario.purchase;
        purchase.operations = vec!["purchase".to_owned(), "purchase".to_owned()];
        purchase.id = expected_capability_id(&purchase)?;
        purchase.proof.signature_hex = scenario
            .personal_agent
            .sign_json(&capability_signing_payload(&purchase))?;
        let action = sign_action(
            &scenario.booking_agent,
            ActionInput {
                controller: scenario.human.id.clone(),
                intent: "book_flight".to_owned(),
                resource: "travel.booking".to_owned(),
                operation: "purchase".to_owned(),
                constraints: amount(750),
                capability_id: purchase.id.clone(),
                context_hash:
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_owned(),
            },
        )?;
        let issuer_keys = BTreeMap::from([
            (
                scenario.root.issuer.clone(),
                scenario.human.public_key_hex.clone(),
            ),
            (
                purchase.issuer.clone(),
                scenario.personal_agent.public_key_hex.clone(),
            ),
        ]);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert!(matches!(
            result,
            VerificationResult::Rejected(
                VerificationError::CapabilityOperationsNotCanonical { .. }
            )
        ));
        Ok(())
    }

    #[test]
    fn rejects_signed_capability_with_unsorted_operations() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let mut root = scenario.root;
        root.operations = vec!["search".to_owned(), "purchase".to_owned()];
        root.id = expected_capability_id(&root)?;
        root.proof.signature_hex = scenario
            .human
            .sign_json(&capability_signing_payload(&root))?;
        let purchase = delegate_capability(
            &scenario.personal_agent,
            &root,
            DelegationInput {
                subject: scenario.booking_agent.id.clone(),
                operations: vec!["purchase".to_owned()],
                constraints: max_amount(800),
                expires_at: at(1_700_000_000)?,
                delegable: false,
            },
        )?;
        let action = sign_action(
            &scenario.booking_agent,
            ActionInput {
                controller: scenario.human.id.clone(),
                intent: "book_flight".to_owned(),
                resource: "travel.booking".to_owned(),
                operation: "purchase".to_owned(),
                constraints: amount(750),
                capability_id: purchase.id.clone(),
                context_hash:
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_owned(),
            },
        )?;
        let issuer_keys = BTreeMap::from([
            (root.issuer.clone(), scenario.human.public_key_hex.clone()),
            (
                purchase.issuer.clone(),
                scenario.personal_agent.public_key_hex.clone(),
            ),
        ]);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[root, purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert!(matches!(
            result,
            VerificationResult::Rejected(
                VerificationError::CapabilityOperationsNotCanonical { .. }
            )
        ));
        Ok(())
    }

    #[test]
    fn rejects_signed_capability_with_empty_operations() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let mut purchase = scenario.purchase;
        purchase.operations = vec![];
        purchase.id = expected_capability_id(&purchase)?;
        purchase.proof.signature_hex = scenario
            .personal_agent
            .sign_json(&capability_signing_payload(&purchase))?;
        let action = sign_action(
            &scenario.booking_agent,
            ActionInput {
                controller: scenario.human.id.clone(),
                intent: "book_flight".to_owned(),
                resource: "travel.booking".to_owned(),
                operation: "purchase".to_owned(),
                constraints: amount(750),
                capability_id: purchase.id.clone(),
                context_hash:
                    "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                        .to_owned(),
            },
        )?;
        let issuer_keys = BTreeMap::from([
            (
                scenario.root.issuer.clone(),
                scenario.human.public_key_hex.clone(),
            ),
            (
                purchase.issuer.clone(),
                scenario.personal_agent.public_key_hex.clone(),
            ),
        ]);
        let revocations = InMemoryRevocationRegistry::default();

        let result = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &[scenario.root, purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now: at(1_650_000_000)?,
        })?;

        assert!(matches!(
            result,
            VerificationResult::Rejected(VerificationError::CapabilityOperationsEmpty { .. })
        ));
        Ok(())
    }

    #[test]
    fn verify_action_once_records_accepted_action_and_rejects_replay() -> Result<(), Box<dyn Error>>
    {
        let scenario = scenario()?;
        let action = purchase_action(&scenario, 750)?;
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();
        let mut replay = crate::replay::InMemoryReplayRegistry::default();

        let first = verify_action_once(VerifyActionOnceInput {
            action: &action,
            capability_chain: &[scenario.root.clone(), scenario.purchase.clone()],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            replay: &mut replay,
            now: at(1_650_000_000)?,
        })?;
        let second = verify_action_once(VerifyActionOnceInput {
            action: &action,
            capability_chain: &[scenario.root, scenario.purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            replay: &mut replay,
            now: at(1_650_000_000)?,
        })?;

        assert_eq!(first, VerificationResult::Accepted);
        assert_eq!(
            second,
            VerificationResult::Rejected(VerificationError::ActionReplayed {
                action_id: action.id
            })
        );
        Ok(())
    }

    #[test]
    fn verify_action_once_does_not_record_rejected_action() -> Result<(), Box<dyn Error>> {
        let scenario = scenario()?;
        let over_budget = purchase_action(&scenario, 900)?;
        let allowed = purchase_action(&scenario, 750)?;
        let issuer_keys = issuer_keys(&scenario);
        let revocations = InMemoryRevocationRegistry::default();
        let mut replay = crate::replay::InMemoryReplayRegistry::default();

        let rejected = verify_action_once(VerifyActionOnceInput {
            action: &over_budget,
            capability_chain: &[scenario.root.clone(), scenario.purchase.clone()],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            replay: &mut replay,
            now: at(1_650_000_000)?,
        })?;
        let accepted = verify_action_once(VerifyActionOnceInput {
            action: &allowed,
            capability_chain: &[scenario.root, scenario.purchase],
            actor_public_key_hex: &scenario.booking_agent.public_key_hex,
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            replay: &mut replay,
            now: at(1_650_000_000)?,
        })?;

        assert!(matches!(
            rejected,
            VerificationResult::Rejected(VerificationError::ConstraintExceeded { .. })
        ));
        assert_eq!(accepted, VerificationResult::Accepted);
        Ok(())
    }

    #[test]
    fn delegated_amount_constraints_are_monotonic_at_boundary() -> Result<(), Box<dyn Error>> {
        for amount_usd in [0, 1, 799, 800] {
            let scenario = scenario()?;
            let action = purchase_action(&scenario, amount_usd)?;
            let issuer_keys = issuer_keys(&scenario);
            let revocations = InMemoryRevocationRegistry::default();

            let result = verify_action(VerifyActionInput {
                action: &action,
                capability_chain: &[scenario.root, scenario.purchase],
                actor_public_key_hex: &scenario.booking_agent.public_key_hex,
                capability_issuer_public_keys: &issuer_keys,
                revocations: &revocations,
                now: at(1_650_000_000)?,
            })?;

            assert_eq!(
                result,
                VerificationResult::Accepted,
                "amount {amount_usd} should be within delegated bound"
            );
        }

        for amount_usd in [801, 1_200, 1_201, 10_000] {
            let scenario = scenario()?;
            let action = purchase_action(&scenario, amount_usd)?;
            let issuer_keys = issuer_keys(&scenario);
            let revocations = InMemoryRevocationRegistry::default();

            let result = verify_action(VerifyActionInput {
                action: &action,
                capability_chain: &[scenario.root, scenario.purchase],
                actor_public_key_hex: &scenario.booking_agent.public_key_hex,
                capability_issuer_public_keys: &issuer_keys,
                revocations: &revocations,
                now: at(1_650_000_000)?,
            })?;

            assert_eq!(
                result,
                VerificationResult::Rejected(VerificationError::ConstraintExceeded {
                    constraint: "max_amount_usd".to_owned()
                }),
                "amount {amount_usd} should exceed delegated bound"
            );
        }

        Ok(())
    }

    #[test]
    fn malformed_signed_action_variants_fail_closed_after_resigning() -> Result<(), Box<dyn Error>>
    {
        let cases = [
            (
                "unsupported_version",
                "rava-action-v999",
                "a3f5b348-f28f-4a99-8cda-9a7e34bc5f01",
                "sha256:0000000000000000000000000000000000000000000000000000000000000000",
                VerificationResult::Rejected(VerificationError::UnsupportedActionVersion {
                    version: "rava-action-v999".to_owned(),
                }),
            ),
            (
                "bad_nonce",
                "rava-action-v0",
                "not-a-uuid",
                "sha256:0000000000000000000000000000000000000000000000000000000000000000",
                VerificationResult::Rejected(VerificationError::ActionNonceInvalid),
            ),
            (
                "bad_context_hash",
                "rava-action-v0",
                "a3f5b348-f28f-4a99-8cda-9a7e34bc5f01",
                "sha256:not-hex",
                VerificationResult::Rejected(VerificationError::ActionContextHashInvalid),
            ),
        ];

        for (label, version, nonce, context_hash, expected) in cases {
            let scenario = scenario()?;
            let mut action = purchase_action(&scenario, 750)?;
            action.version = version.to_owned();
            action.nonce = nonce.to_owned();
            action.context_hash = context_hash.to_owned();
            action.id = expected_action_id(&action)?;
            action.proof.signature_hex = scenario
                .booking_agent
                .sign_json(&action_signing_payload(&action))?;
            let issuer_keys = issuer_keys(&scenario);
            let revocations = InMemoryRevocationRegistry::default();

            let result = verify_action(VerifyActionInput {
                action: &action,
                capability_chain: &[scenario.root, scenario.purchase],
                actor_public_key_hex: &scenario.booking_agent.public_key_hex,
                capability_issuer_public_keys: &issuer_keys,
                revocations: &revocations,
                now: at(1_650_000_000)?,
            })?;

            assert_eq!(result, expected, "{label}");
        }

        Ok(())
    }

    mod attacker_stories {
        use super::*;

        #[test]
        fn forged_child_capability_cannot_broaden_any_parent_attenuation_dimension(
        ) -> Result<(), Box<dyn Error>> {
            let cases = [
                (
                    "resource",
                    "payments.transfer",
                    vec!["purchase".to_owned()],
                    max_amount(800),
                    at(1_700_000_000)?,
                    true,
                    VerificationError::CapabilityResourceMismatch {
                        capability_id: String::new(),
                    },
                    "purchase",
                    750,
                ),
                (
                    "operation",
                    "travel.booking",
                    vec!["refund".to_owned()],
                    max_amount(800),
                    at(1_700_000_000)?,
                    true,
                    VerificationError::CapabilityOperationNotGranted {
                        capability_id: String::new(),
                        operation: "refund".to_owned(),
                    },
                    "refund",
                    750,
                ),
                (
                    "expiry",
                    "travel.booking",
                    vec!["purchase".to_owned()],
                    max_amount(800),
                    at(1_900_000_000)?,
                    true,
                    VerificationError::CapabilityExpiryOutlivesParent {
                        capability_id: String::new(),
                    },
                    "purchase",
                    750,
                ),
                (
                    "constraint_expansion",
                    "travel.booking",
                    vec!["purchase".to_owned()],
                    max_amount(5_000),
                    at(1_700_000_000)?,
                    true,
                    VerificationError::CapabilityConstraintExpanded {
                        capability_id: String::new(),
                        constraint: "max_amount_usd".to_owned(),
                    },
                    "purchase",
                    3_000,
                ),
                (
                    "constraint_removal",
                    "travel.booking",
                    vec!["purchase".to_owned()],
                    BTreeMap::new(),
                    at(1_700_000_000)?,
                    true,
                    VerificationError::CapabilityConstraintRemoved {
                        capability_id: String::new(),
                        constraint: "max_amount_usd".to_owned(),
                    },
                    "purchase",
                    750,
                ),
                (
                    "non_delegable_parent",
                    "travel.booking",
                    vec!["purchase".to_owned()],
                    max_amount(800),
                    at(1_700_000_000)?,
                    false,
                    VerificationError::ParentCapabilityNotDelegable {
                        capability_id: String::new(),
                    },
                    "purchase",
                    750,
                ),
            ];

            for (
                label,
                resource,
                operations,
                constraints,
                expires_at,
                parent_delegable,
                expected_error,
                action_operation,
                action_amount,
            ) in cases
            {
                let scenario = scenario()?;
                let issuer_keys = issuer_keys(&scenario);
                let revocations = InMemoryRevocationRegistry::default();
                let mut root = scenario.root.clone();
                root.delegable = parent_delegable;
                resign_capability(&scenario.human, &mut root)?;

                let mut child = scenario.purchase.clone();
                child.parent_id = Some(root.id.clone());
                child.resource = resource.to_owned();
                child.operations = operations;
                child.constraints = constraints;
                child.expires_at = expires_at;
                resign_capability(&scenario.personal_agent, &mut child)?;
                let action =
                    action_for_capability(&scenario, &child, action_operation, action_amount)?;

                let result = verify_action(VerifyActionInput {
                    action: &action,
                    capability_chain: &[root.clone(), child.clone()],
                    actor_public_key_hex: &scenario.booking_agent.public_key_hex,
                    capability_issuer_public_keys: &issuer_keys,
                    revocations: &revocations,
                    now: at(1_650_000_000)?,
                })?;

                assert_eq!(
                    result,
                    VerificationResult::Rejected(error_with_capability_id(
                        expected_error,
                        if label == "non_delegable_parent" {
                            root.id.clone()
                        } else {
                            child.id.clone()
                        },
                    )),
                    "{label}"
                );
            }

            Ok(())
        }

        #[test]
        fn reordered_chain_root_issuer_must_match_action_controller() -> Result<(), Box<dyn Error>>
        {
            let scenario = scenario()?;
            let other_controller = Signer::generate(SignerKind::Human);
            let action = sign_action(
                &scenario.booking_agent,
                ActionInput {
                    controller: other_controller.id,
                    intent: "book_flight".to_owned(),
                    resource: "travel.booking".to_owned(),
                    operation: "purchase".to_owned(),
                    constraints: amount(750),
                    capability_id: scenario.purchase.id.clone(),
                    context_hash:
                        "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                            .to_owned(),
                },
            )?;
            let issuer_keys = issuer_keys(&scenario);
            let revocations = InMemoryRevocationRegistry::default();

            let result = verify_action(VerifyActionInput {
                action: &action,
                capability_chain: &[scenario.root, scenario.purchase],
                actor_public_key_hex: &scenario.booking_agent.public_key_hex,
                capability_issuer_public_keys: &issuer_keys,
                revocations: &revocations,
                now: at(1_650_000_000)?,
            })?;

            assert_eq!(
                result,
                VerificationResult::Rejected(VerificationError::RootIssuerNotController)
            );
            Ok(())
        }

        #[test]
        fn accepted_action_replay_is_rejected_but_rejected_action_is_not_consumed(
        ) -> Result<(), Box<dyn Error>> {
            let scenario = scenario()?;
            let over_budget = purchase_action(&scenario, 900)?;
            let allowed = purchase_action(&scenario, 750)?;
            let issuer_keys = issuer_keys(&scenario);
            let revocations = InMemoryRevocationRegistry::default();
            let mut replay = crate::replay::InMemoryReplayRegistry::default();

            let rejected = verify_action_once(VerifyActionOnceInput {
                action: &over_budget,
                capability_chain: &[scenario.root.clone(), scenario.purchase.clone()],
                actor_public_key_hex: &scenario.booking_agent.public_key_hex,
                capability_issuer_public_keys: &issuer_keys,
                revocations: &revocations,
                replay: &mut replay,
                now: at(1_650_000_000)?,
            })?;
            let accepted = verify_action_once(VerifyActionOnceInput {
                action: &allowed,
                capability_chain: &[scenario.root.clone(), scenario.purchase.clone()],
                actor_public_key_hex: &scenario.booking_agent.public_key_hex,
                capability_issuer_public_keys: &issuer_keys,
                revocations: &revocations,
                replay: &mut replay,
                now: at(1_650_000_000)?,
            })?;
            let replayed = verify_action_once(VerifyActionOnceInput {
                action: &allowed,
                capability_chain: &[scenario.root, scenario.purchase],
                actor_public_key_hex: &scenario.booking_agent.public_key_hex,
                capability_issuer_public_keys: &issuer_keys,
                revocations: &revocations,
                replay: &mut replay,
                now: at(1_650_000_000)?,
            })?;

            assert!(matches!(
                rejected,
                VerificationResult::Rejected(VerificationError::ConstraintExceeded { .. })
            ));
            assert_eq!(accepted, VerificationResult::Accepted);
            assert_eq!(
                replayed,
                VerificationResult::Rejected(VerificationError::ActionReplayed {
                    action_id: allowed.id
                })
            );
            Ok(())
        }

        fn error_with_capability_id(
            error: VerificationError,
            capability_id: String,
        ) -> VerificationError {
            match error {
                VerificationError::CapabilityResourceMismatch { .. } => {
                    VerificationError::CapabilityResourceMismatch { capability_id }
                }
                VerificationError::CapabilityOperationNotGranted { operation, .. } => {
                    VerificationError::CapabilityOperationNotGranted {
                        capability_id,
                        operation,
                    }
                }
                VerificationError::CapabilityExpiryOutlivesParent { .. } => {
                    VerificationError::CapabilityExpiryOutlivesParent { capability_id }
                }
                VerificationError::CapabilityConstraintExpanded { constraint, .. } => {
                    VerificationError::CapabilityConstraintExpanded {
                        capability_id,
                        constraint,
                    }
                }
                VerificationError::CapabilityConstraintRemoved { constraint, .. } => {
                    VerificationError::CapabilityConstraintRemoved {
                        capability_id,
                        constraint,
                    }
                }
                VerificationError::ParentCapabilityNotDelegable { .. } => {
                    VerificationError::ParentCapabilityNotDelegable { capability_id }
                }
                _ => error,
            }
        }
    }

    fn issuer_keys(scenario: &Scenario) -> BTreeMap<String, String> {
        BTreeMap::from([
            (
                scenario.root.issuer.clone(),
                scenario.human.public_key_hex.clone(),
            ),
            (
                scenario.purchase.issuer.clone(),
                scenario.personal_agent.public_key_hex.clone(),
            ),
        ])
    }
}
