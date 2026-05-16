use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::canonical::{canonical_json, CanonicalError};
use crate::error::RavaError;
use crate::hash::is_sha256_hash;
use crate::identity::{verify_json_signature, Signer};
use crate::nonce::is_canonical_uuid_v4;
use crate::protocol::{
    is_supported_attestation_version, ATTESTATION_ID_PREFIX, ATTESTATION_PENDING_ID,
    ATTESTATION_VERSION,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttestationOutcome {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationProof {
    pub signature_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attestation {
    pub version: String,
    pub id: String,
    pub nonce: String,
    pub action_id: String,
    pub evaluator: String,
    pub outcome: AttestationOutcome,
    pub subject: String,
    pub occurred_at: OffsetDateTime,
    pub evidence_hash: String,
    pub proof: AttestationProof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttestationInput {
    pub action_id: String,
    pub outcome: AttestationOutcome,
    pub subject: String,
    pub occurred_at: OffsetDateTime,
    pub evidence_hash: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AttestationError {
    #[error(transparent)]
    Canonical(#[from] CanonicalError),

    #[error(transparent)]
    Rava(#[from] RavaError),

    #[error("attestation evidence_hash must be sha256:<64 lowercase hex chars>")]
    InvalidEvidenceHash,
}

pub fn sign_attestation(
    evaluator: &Signer,
    input: AttestationInput,
) -> Result<Attestation, AttestationError> {
    if !is_sha256_hash(&input.evidence_hash) {
        return Err(AttestationError::InvalidEvidenceHash);
    }

    let nonce = Uuid::new_v4().to_string();
    let pending_id = ATTESTATION_PENDING_ID;
    let unsigned = unsigned_attestation_value(pending_id, &nonce, &evaluator.id, &input);
    let id = attestation_id(&unsigned)?;
    let signing_payload = unsigned_attestation_value(&id, &nonce, &evaluator.id, &input);
    let signature_hex = evaluator.sign_json(&signing_payload)?;

    Ok(Attestation {
        version: ATTESTATION_VERSION.to_owned(),
        id,
        nonce,
        action_id: input.action_id,
        evaluator: evaluator.id.clone(),
        outcome: input.outcome,
        subject: input.subject,
        occurred_at: input.occurred_at,
        evidence_hash: input.evidence_hash,
        proof: AttestationProof { signature_hex },
    })
}

pub fn verify_attestation(
    attestation: &Attestation,
    evaluator_public_key_hex: &str,
) -> Result<bool, RavaError> {
    if !is_supported_attestation_version(&attestation.version) {
        return Ok(false);
    }
    if !is_canonical_uuid_v4(&attestation.nonce) {
        return Ok(false);
    }
    if !is_sha256_hash(&attestation.evidence_hash) {
        return Ok(false);
    }

    if expected_attestation_id(attestation)? != attestation.id {
        return Ok(false);
    }

    verify_json_signature(
        &attestation.evaluator,
        evaluator_public_key_hex,
        &attestation_signing_payload(attestation),
        &attestation.proof.signature_hex,
    )
}

pub fn attestation_signing_payload(attestation: &Attestation) -> Value {
    json!({
        "version": attestation.version,
        "id": attestation.id,
        "nonce": attestation.nonce,
        "action_id": attestation.action_id,
        "evaluator": attestation.evaluator,
        "outcome": attestation.outcome,
        "subject": attestation.subject,
        "occurred_at": attestation.occurred_at,
        "evidence_hash": attestation.evidence_hash,
    })
}

pub fn expected_attestation_id(attestation: &Attestation) -> Result<String, CanonicalError> {
    let unsigned = json!({
        "version": attestation.version,
        "id": ATTESTATION_PENDING_ID,
        "nonce": attestation.nonce,
        "action_id": attestation.action_id,
        "evaluator": attestation.evaluator,
        "outcome": attestation.outcome,
        "subject": attestation.subject,
        "occurred_at": attestation.occurred_at,
        "evidence_hash": attestation.evidence_hash,
    });

    attestation_id(&unsigned)
}

fn unsigned_attestation_value(
    id: &str,
    nonce: &str,
    evaluator: &str,
    input: &AttestationInput,
) -> Value {
    json!({
        "version": ATTESTATION_VERSION,
        "id": id,
        "nonce": nonce,
        "action_id": input.action_id,
        "evaluator": evaluator,
        "outcome": input.outcome,
        "subject": input.subject,
        "occurred_at": input.occurred_at,
        "evidence_hash": input.evidence_hash,
    })
}

fn attestation_id(unsigned: &Value) -> Result<String, CanonicalError> {
    let canonical = canonical_json(unsigned)?;
    let digest = Sha256::digest(canonical.as_bytes());
    Ok(format!(
        "{}{}",
        ATTESTATION_ID_PREFIX,
        hex::encode(&digest[..16])
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{Signer, SignerKind};
    use std::error::Error;
    use time::OffsetDateTime;

    fn at(seconds: i64) -> Result<OffsetDateTime, Box<dyn Error>> {
        Ok(OffsetDateTime::from_unix_timestamp(seconds)?)
    }

    #[test]
    fn signer_attests_action_outcome() -> Result<(), Box<dyn Error>> {
        let service = Signer::generate(SignerKind::Service);

        let attestation = sign_attestation(
            &service,
            AttestationInput {
                action_id: "act_demo".to_owned(),
                outcome: AttestationOutcome::Accepted,
                subject: "travel.booking".to_owned(),
                occurred_at: at(1_650_000_000)?,
                evidence_hash:
                    "sha256:3333333333333333333333333333333333333333333333333333333333333333"
                        .to_owned(),
            },
        )?;

        assert_eq!(attestation.evaluator, service.id);
        assert!(verify_attestation(&attestation, &service.public_key_hex)?);
        Ok(())
    }

    #[test]
    fn verifier_rejects_tampered_attestation() -> Result<(), Box<dyn Error>> {
        let service = Signer::generate(SignerKind::Service);
        let mut attestation = sign_attestation(
            &service,
            AttestationInput {
                action_id: "act_demo".to_owned(),
                outcome: AttestationOutcome::Accepted,
                subject: "travel.booking".to_owned(),
                occurred_at: at(1_650_000_000)?,
                evidence_hash:
                    "sha256:3333333333333333333333333333333333333333333333333333333333333333"
                        .to_owned(),
            },
        )?;

        attestation.outcome = AttestationOutcome::Rejected;

        assert!(!verify_attestation(&attestation, &service.public_key_hex)?);
        Ok(())
    }

    #[test]
    fn verifier_rejects_attestation_with_non_derived_id() -> Result<(), Box<dyn Error>> {
        let service = Signer::generate(SignerKind::Service);
        let mut attestation = sign_attestation(
            &service,
            AttestationInput {
                action_id: "act_demo".to_owned(),
                outcome: AttestationOutcome::Accepted,
                subject: "travel.booking".to_owned(),
                occurred_at: at(1_650_000_000)?,
                evidence_hash:
                    "sha256:3333333333333333333333333333333333333333333333333333333333333333"
                        .to_owned(),
            },
        )?;
        attestation.id = "att_attacker_chosen".to_owned();
        attestation.proof.signature_hex =
            service.sign_json(&attestation_signing_payload(&attestation))?;

        assert!(!verify_attestation(&attestation, &service.public_key_hex)?);
        Ok(())
    }

    #[test]
    fn attacker_story_rejects_recomputed_and_resigned_attestation_with_malformed_nonce(
    ) -> Result<(), Box<dyn Error>> {
        let service = Signer::generate(SignerKind::Service);
        let mut attestation = sign_attestation(
            &service,
            AttestationInput {
                action_id: "act_demo".to_owned(),
                outcome: AttestationOutcome::Accepted,
                subject: "travel.booking".to_owned(),
                occurred_at: at(1_650_000_000)?,
                evidence_hash:
                    "sha256:3333333333333333333333333333333333333333333333333333333333333333"
                        .to_owned(),
            },
        )?;
        attestation.nonce = "not-a-uuid".to_owned();
        attestation.id = expected_attestation_id(&attestation)?;
        attestation.proof.signature_hex =
            service.sign_json(&attestation_signing_payload(&attestation))?;

        assert!(!verify_attestation(&attestation, &service.public_key_hex)?);
        Ok(())
    }

    #[test]
    fn verifier_rejects_unsupported_attestation_version() -> Result<(), Box<dyn Error>> {
        let service = Signer::generate(SignerKind::Service);
        let mut attestation = sign_attestation(
            &service,
            AttestationInput {
                action_id: "act_demo".to_owned(),
                outcome: AttestationOutcome::Accepted,
                subject: "travel.booking".to_owned(),
                occurred_at: at(1_650_000_000)?,
                evidence_hash:
                    "sha256:3333333333333333333333333333333333333333333333333333333333333333"
                        .to_owned(),
            },
        )?;
        attestation.version = "rava-attestation-v999".to_owned();
        attestation.id = expected_attestation_id(&attestation)?;
        attestation.proof.signature_hex =
            service.sign_json(&attestation_signing_payload(&attestation))?;

        assert!(!verify_attestation(&attestation, &service.public_key_hex)?);
        Ok(())
    }

    #[test]
    fn sign_attestation_rejects_invalid_evidence_hash() -> Result<(), Box<dyn Error>> {
        let service = Signer::generate(SignerKind::Service);

        let result = sign_attestation(
            &service,
            AttestationInput {
                action_id: "act_demo".to_owned(),
                outcome: AttestationOutcome::Accepted,
                subject: "travel.booking".to_owned(),
                occurred_at: at(1_650_000_000)?,
                evidence_hash: "sha256:not-hex".to_owned(),
            },
        );

        assert!(matches!(result, Err(AttestationError::InvalidEvidenceHash)));
        Ok(())
    }

    #[test]
    fn verifier_rejects_attestation_with_invalid_evidence_hash() -> Result<(), Box<dyn Error>> {
        let service = Signer::generate(SignerKind::Service);
        let mut attestation = sign_attestation(
            &service,
            AttestationInput {
                action_id: "act_demo".to_owned(),
                outcome: AttestationOutcome::Accepted,
                subject: "travel.booking".to_owned(),
                occurred_at: at(1_650_000_000)?,
                evidence_hash:
                    "sha256:3333333333333333333333333333333333333333333333333333333333333333"
                        .to_owned(),
            },
        )?;
        attestation.evidence_hash = "sha256:not-hex".to_owned();
        attestation.id = expected_attestation_id(&attestation)?;
        attestation.proof.signature_hex =
            service.sign_json(&attestation_signing_payload(&attestation))?;

        assert!(!verify_attestation(&attestation, &service.public_key_hex)?);
        Ok(())
    }
}
