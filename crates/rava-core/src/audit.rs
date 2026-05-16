use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::action::ActionEnvelope;
use crate::canonical::{canonical_json, CanonicalError};
use crate::capability::Capability;
use crate::error::RavaError;
use crate::hash::is_sha256_hash;
use crate::identity::{verify_json_signature, Signer};
use crate::nonce::is_canonical_uuid_v4;
use crate::protocol::{
    is_supported_verification_receipt_version, SHA256_PREFIX, VERIFICATION_RECEIPT_ID_PREFIX,
    VERIFICATION_RECEIPT_PENDING_ID, VERIFICATION_RECEIPT_VERSION,
};
use crate::verifier::{VerificationError, VerificationResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationReceiptProof {
    pub signature_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationDecision {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationRejectionReason {
    pub code: String,
    pub subject: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationReceipt {
    pub version: String,
    pub id: String,
    pub nonce: String,
    pub verifier: String,
    pub action_id: String,
    pub actor: String,
    pub capability_id: String,
    pub capability_chain_hash: String,
    pub context_hash: String,
    pub decision: VerificationDecision,
    pub reason: Option<VerificationRejectionReason>,
    pub verified_at: OffsetDateTime,
    pub proof: VerificationReceiptProof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationReceiptInput<'a> {
    pub action: &'a ActionEnvelope,
    pub capability_chain: &'a [Capability],
    pub result: &'a VerificationResult,
    pub verified_at: OffsetDateTime,
}

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error(transparent)]
    Canonical(#[from] CanonicalError),

    #[error(transparent)]
    Rava(#[from] RavaError),
}

pub fn sign_verification_receipt(
    verifier: &Signer,
    input: VerificationReceiptInput<'_>,
) -> Result<VerificationReceipt, AuditError> {
    let nonce = Uuid::new_v4().to_string();
    let capability_chain_hash = capability_chain_hash(input.capability_chain)?;
    let pending_id = VERIFICATION_RECEIPT_PENDING_ID;
    let unsigned = unsigned_receipt_value(
        pending_id,
        &nonce,
        &verifier.id,
        &capability_chain_hash,
        &input,
    );
    let id = receipt_id(&unsigned)?;
    let signing_payload =
        unsigned_receipt_value(&id, &nonce, &verifier.id, &capability_chain_hash, &input);
    let signature_hex = verifier.sign_json(&signing_payload)?;
    let (decision, reason) = decision_parts(input.result);

    Ok(VerificationReceipt {
        version: VERIFICATION_RECEIPT_VERSION.to_owned(),
        id,
        nonce,
        verifier: verifier.id.clone(),
        action_id: input.action.id.clone(),
        actor: input.action.actor.clone(),
        capability_id: input.action.capability_id.clone(),
        capability_chain_hash,
        context_hash: input.action.context_hash.clone(),
        decision,
        reason,
        verified_at: input.verified_at,
        proof: VerificationReceiptProof { signature_hex },
    })
}

pub fn verify_verification_receipt(
    receipt: &VerificationReceipt,
    verifier_public_key_hex: &str,
) -> Result<bool, RavaError> {
    if !is_supported_verification_receipt_version(&receipt.version) {
        return Ok(false);
    }
    if !is_canonical_uuid_v4(&receipt.nonce) {
        return Ok(false);
    }
    if !is_sha256_hash(&receipt.capability_chain_hash) {
        return Ok(false);
    }

    if expected_verification_receipt_id(receipt)? != receipt.id {
        return Ok(false);
    }

    verify_json_signature(
        &receipt.verifier,
        verifier_public_key_hex,
        &verification_receipt_signing_payload(receipt),
        &receipt.proof.signature_hex,
    )
}

pub fn verification_receipt_signing_payload(receipt: &VerificationReceipt) -> Value {
    json!({
        "version": receipt.version,
        "id": receipt.id,
        "nonce": receipt.nonce,
        "verifier": receipt.verifier,
        "action_id": receipt.action_id,
        "actor": receipt.actor,
        "capability_id": receipt.capability_id,
        "capability_chain_hash": receipt.capability_chain_hash,
        "context_hash": receipt.context_hash,
        "decision": receipt.decision,
        "reason": receipt.reason,
        "verified_at": receipt.verified_at,
    })
}

pub fn expected_verification_receipt_id(
    receipt: &VerificationReceipt,
) -> Result<String, CanonicalError> {
    let unsigned = json!({
        "version": receipt.version,
        "id": VERIFICATION_RECEIPT_PENDING_ID,
        "nonce": receipt.nonce,
        "verifier": receipt.verifier,
        "action_id": receipt.action_id,
        "actor": receipt.actor,
        "capability_id": receipt.capability_id,
        "capability_chain_hash": receipt.capability_chain_hash,
        "context_hash": receipt.context_hash,
        "decision": receipt.decision,
        "reason": receipt.reason,
        "verified_at": receipt.verified_at,
    });

    receipt_id(&unsigned)
}

fn unsigned_receipt_value(
    id: &str,
    nonce: &str,
    verifier: &str,
    capability_chain_hash: &str,
    input: &VerificationReceiptInput<'_>,
) -> Value {
    let (decision, reason) = decision_parts(input.result);
    json!({
        "version": VERIFICATION_RECEIPT_VERSION,
        "id": id,
        "nonce": nonce,
        "verifier": verifier,
        "action_id": input.action.id,
        "actor": input.action.actor,
        "capability_id": input.action.capability_id,
        "capability_chain_hash": capability_chain_hash,
        "context_hash": input.action.context_hash,
        "decision": decision,
        "reason": reason,
        "verified_at": input.verified_at,
    })
}

pub fn capability_chain_hash(capability_chain: &[Capability]) -> Result<String, CanonicalError> {
    let canonical = canonical_json(&serde_json::to_value(capability_chain)?)?;
    let digest = Sha256::digest(canonical.as_bytes());
    Ok(format!("{}{}", SHA256_PREFIX, hex::encode(digest)))
}

fn receipt_id(unsigned: &Value) -> Result<String, CanonicalError> {
    let canonical = canonical_json(unsigned)?;
    let digest = Sha256::digest(canonical.as_bytes());
    Ok(format!(
        "{}{}",
        VERIFICATION_RECEIPT_ID_PREFIX,
        hex::encode(&digest[..16])
    ))
}

fn decision_parts(
    result: &VerificationResult,
) -> (VerificationDecision, Option<VerificationRejectionReason>) {
    match result {
        VerificationResult::Accepted => (VerificationDecision::Accepted, None),
        VerificationResult::Rejected(error) => (
            VerificationDecision::Rejected,
            Some(reason_from_error(error)),
        ),
    }
}

fn reason_from_error(error: &VerificationError) -> VerificationRejectionReason {
    VerificationRejectionReason {
        code: error.code().to_owned(),
        subject: error.subject(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{sign_action, ActionInput};
    use crate::capability::{
        delegate_capability, mint_capability, Capability, CapabilityInput, ConstraintValue,
        DelegationInput,
    };
    use crate::identity::{Signer, SignerKind};
    use crate::verifier::{VerificationError, VerificationResult};
    use std::collections::BTreeMap;
    use std::error::Error;
    use time::OffsetDateTime;

    fn at(seconds: i64) -> Result<OffsetDateTime, Box<dyn Error>> {
        Ok(OffsetDateTime::from_unix_timestamp(seconds)?)
    }

    fn action(actor: &Signer) -> Result<crate::action::ActionEnvelope, Box<dyn Error>> {
        Ok(sign_action(
            actor,
            ActionInput {
                controller: "did:rava:human".to_owned(),
                intent: "book_flight".to_owned(),
                resource: "travel.booking".to_owned(),
                operation: "purchase".to_owned(),
                constraints: BTreeMap::from([(
                    "amount_usd".to_owned(),
                    ConstraintValue::Integer(750),
                )]),
                capability_id: "cap_demo".to_owned(),
                context_hash:
                    "sha256:1111111111111111111111111111111111111111111111111111111111111111"
                        .to_owned(),
            },
        )?)
    }

    fn capability_chain(
        controller: &Signer,
        agent: &Signer,
    ) -> Result<Vec<Capability>, Box<dyn Error>> {
        let root = mint_capability(
            controller,
            CapabilityInput {
                subject: agent.id.clone(),
                resource: "travel.booking".to_owned(),
                operations: vec!["purchase".to_owned()],
                constraints: BTreeMap::from([(
                    "max_amount_usd".to_owned(),
                    ConstraintValue::Integer(1_200),
                )]),
                expires_at: at(1_700_000_000)?,
                delegable: true,
            },
        )?;
        let delegated = delegate_capability(
            agent,
            &root,
            DelegationInput {
                subject: agent.id.clone(),
                operations: vec!["purchase".to_owned()],
                constraints: BTreeMap::from([(
                    "max_amount_usd".to_owned(),
                    ConstraintValue::Integer(800),
                )]),
                expires_at: at(1_690_000_000)?,
                delegable: false,
            },
        )?;

        Ok(vec![root, delegated])
    }

    #[test]
    fn verifier_signs_minimal_verification_receipt() -> Result<(), Box<dyn Error>> {
        let verifier = Signer::generate(SignerKind::Service);
        let actor = Signer::generate(SignerKind::Agent);
        let controller = Signer::generate(SignerKind::Human);
        let action = action(&actor)?;
        let chain = capability_chain(&controller, &actor)?;

        let receipt = sign_verification_receipt(
            &verifier,
            VerificationReceiptInput {
                action: &action,
                capability_chain: &chain,
                result: &VerificationResult::Accepted,
                verified_at: at(1_650_000_000)?,
            },
        )?;

        assert_eq!(receipt.verifier, verifier.id);
        assert_eq!(receipt.action_id, action.id);
        assert_eq!(receipt.actor, actor.id);
        assert_eq!(receipt.capability_id, "cap_demo");
        assert_eq!(
            receipt.capability_chain_hash,
            capability_chain_hash(&chain)?
        );
        assert_eq!(receipt.decision, VerificationDecision::Accepted);
        assert!(receipt.reason.is_none());
        assert!(verify_verification_receipt(
            &receipt,
            &verifier.public_key_hex
        )?);
        Ok(())
    }

    #[test]
    fn verification_receipt_omits_raw_action_payload() -> Result<(), Box<dyn Error>> {
        let verifier = Signer::generate(SignerKind::Service);
        let actor = Signer::generate(SignerKind::Agent);
        let controller = Signer::generate(SignerKind::Human);
        let action = action(&actor)?;
        let chain = capability_chain(&controller, &actor)?;

        let receipt = sign_verification_receipt(
            &verifier,
            VerificationReceiptInput {
                action: &action,
                capability_chain: &chain,
                result: &VerificationResult::Rejected(VerificationError::ConstraintExceeded {
                    constraint: "max_amount_usd".to_owned(),
                }),
                verified_at: at(1_650_000_000)?,
            },
        )?;
        let json = serde_json::to_value(&receipt)?;

        assert!(json.get("intent").is_none());
        assert!(json.get("resource").is_none());
        assert!(json.get("operation").is_none());
        assert!(json.get("constraints").is_none());
        assert_eq!(receipt.decision, VerificationDecision::Rejected);
        assert_eq!(
            receipt.reason,
            Some(VerificationRejectionReason {
                code: "constraint_exceeded".to_owned(),
                subject: Some("max_amount_usd".to_owned()),
            })
        );
        Ok(())
    }

    #[test]
    fn verifier_rejects_tampered_verification_receipt() -> Result<(), Box<dyn Error>> {
        let verifier = Signer::generate(SignerKind::Service);
        let actor = Signer::generate(SignerKind::Agent);
        let controller = Signer::generate(SignerKind::Human);
        let action = action(&actor)?;
        let chain = capability_chain(&controller, &actor)?;
        let mut receipt = sign_verification_receipt(
            &verifier,
            VerificationReceiptInput {
                action: &action,
                capability_chain: &chain,
                result: &VerificationResult::Accepted,
                verified_at: at(1_650_000_000)?,
            },
        )?;

        receipt.decision = VerificationDecision::Rejected;

        assert!(!verify_verification_receipt(
            &receipt,
            &verifier.public_key_hex
        )?);
        Ok(())
    }

    #[test]
    fn verifier_rejects_receipt_with_tampered_capability_chain_hash() -> Result<(), Box<dyn Error>>
    {
        let verifier = Signer::generate(SignerKind::Service);
        let actor = Signer::generate(SignerKind::Agent);
        let controller = Signer::generate(SignerKind::Human);
        let action = action(&actor)?;
        let chain = capability_chain(&controller, &actor)?;
        let mut receipt = sign_verification_receipt(
            &verifier,
            VerificationReceiptInput {
                action: &action,
                capability_chain: &chain,
                result: &VerificationResult::Accepted,
                verified_at: at(1_650_000_000)?,
            },
        )?;

        receipt.capability_chain_hash =
            "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_owned();

        assert!(!verify_verification_receipt(
            &receipt,
            &verifier.public_key_hex
        )?);
        Ok(())
    }

    #[test]
    fn attacker_story_rejects_recomputed_and_resigned_receipt_with_malformed_nonce(
    ) -> Result<(), Box<dyn Error>> {
        let verifier = Signer::generate(SignerKind::Service);
        let actor = Signer::generate(SignerKind::Agent);
        let controller = Signer::generate(SignerKind::Human);
        let action = action(&actor)?;
        let chain = capability_chain(&controller, &actor)?;
        let mut receipt = sign_verification_receipt(
            &verifier,
            VerificationReceiptInput {
                action: &action,
                capability_chain: &chain,
                result: &VerificationResult::Accepted,
                verified_at: at(1_650_000_000)?,
            },
        )?;
        receipt.nonce = "not-a-uuid".to_owned();
        receipt.id = expected_verification_receipt_id(&receipt)?;
        receipt.proof.signature_hex =
            verifier.sign_json(&verification_receipt_signing_payload(&receipt))?;

        assert!(!verify_verification_receipt(
            &receipt,
            &verifier.public_key_hex
        )?);
        Ok(())
    }

    #[test]
    fn attacker_story_rejects_recomputed_and_resigned_receipt_with_malformed_capability_chain_hash(
    ) -> Result<(), Box<dyn Error>> {
        let verifier = Signer::generate(SignerKind::Service);
        let actor = Signer::generate(SignerKind::Agent);
        let controller = Signer::generate(SignerKind::Human);
        let action = action(&actor)?;
        let chain = capability_chain(&controller, &actor)?;
        let mut receipt = sign_verification_receipt(
            &verifier,
            VerificationReceiptInput {
                action: &action,
                capability_chain: &chain,
                result: &VerificationResult::Accepted,
                verified_at: at(1_650_000_000)?,
            },
        )?;
        receipt.capability_chain_hash = "sha256:not-hex".to_owned();
        receipt.id = expected_verification_receipt_id(&receipt)?;
        receipt.proof.signature_hex =
            verifier.sign_json(&verification_receipt_signing_payload(&receipt))?;

        assert!(!verify_verification_receipt(
            &receipt,
            &verifier.public_key_hex
        )?);
        Ok(())
    }

    #[test]
    fn verifier_rejects_unsupported_verification_receipt_version() -> Result<(), Box<dyn Error>> {
        let verifier = Signer::generate(SignerKind::Service);
        let actor = Signer::generate(SignerKind::Agent);
        let controller = Signer::generate(SignerKind::Human);
        let action = action(&actor)?;
        let chain = capability_chain(&controller, &actor)?;
        let mut receipt = sign_verification_receipt(
            &verifier,
            VerificationReceiptInput {
                action: &action,
                capability_chain: &chain,
                result: &VerificationResult::Accepted,
                verified_at: at(1_650_000_000)?,
            },
        )?;
        receipt.version = "rava-verification-receipt-v999".to_owned();
        receipt.id = expected_verification_receipt_id(&receipt)?;
        receipt.proof.signature_hex =
            verifier.sign_json(&verification_receipt_signing_payload(&receipt))?;

        assert!(!verify_verification_receipt(
            &receipt,
            &verifier.public_key_hex
        )?);
        Ok(())
    }
}
