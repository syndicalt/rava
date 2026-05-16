use ed25519_dalek::{Signature, Signer as DalekSigner, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::canonical::canonical_json;
use crate::error::RavaError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignerKind {
    Human,
    Agent,
    Service,
    Runtime,
}

#[derive(Debug, Clone)]
pub struct Signer {
    signing_key: SigningKey,
    pub id: String,
    pub kind: SignerKind,
    pub public_key_hex: String,
}

impl Signer {
    pub fn generate(kind: SignerKind) -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let public_key = signing_key.verifying_key();
        let public_key_hex = hex::encode(public_key.as_bytes());
        let kind_label = match kind {
            SignerKind::Human => "human",
            SignerKind::Agent => "agent",
            SignerKind::Service => "service",
            SignerKind::Runtime => "runtime",
        };
        let id = format!("rava:{kind_label}:{}", &public_key_hex[..32]);

        Self {
            signing_key,
            id,
            kind,
            public_key_hex,
        }
    }

    pub fn from_signing_key_hex(
        kind: SignerKind,
        signing_key_hex: &str,
    ) -> Result<Self, RavaError> {
        let signing_key_bytes: [u8; 32] = hex::decode(signing_key_hex)?
            .try_into()
            .map_err(|_| RavaError::InvalidSigningKey)?;
        let signing_key = SigningKey::from_bytes(&signing_key_bytes);
        Ok(Self::from_signing_key(kind, signing_key))
    }

    pub fn signing_key_hex(&self) -> String {
        hex::encode(self.signing_key.to_bytes())
    }

    pub fn sign_json(&self, payload: &Value) -> Result<String, RavaError> {
        let canonical = canonical_json(payload)?;
        let signature = self.signing_key.sign(canonical.as_bytes());
        Ok(hex::encode(signature.to_bytes()))
    }

    fn from_signing_key(kind: SignerKind, signing_key: SigningKey) -> Self {
        let public_key = signing_key.verifying_key();
        let public_key_hex = hex::encode(public_key.as_bytes());
        let kind_label = match kind {
            SignerKind::Human => "human",
            SignerKind::Agent => "agent",
            SignerKind::Service => "service",
            SignerKind::Runtime => "runtime",
        };
        let id = format!("rava:{kind_label}:{}", &public_key_hex[..32]);

        Self {
            signing_key,
            id,
            kind,
            public_key_hex,
        }
    }
}

pub fn verify_json_signature(
    signer_id: &str,
    public_key_hex: &str,
    payload: &Value,
    signature_hex: &str,
) -> Result<bool, RavaError> {
    if !signer_id_matches_public_key(signer_id, public_key_hex) {
        return Ok(false);
    }

    let public_key_bytes: [u8; 32] = hex::decode(public_key_hex)?
        .try_into()
        .map_err(|_| RavaError::InvalidPublicKey)?;
    let signature_bytes: [u8; 64] = hex::decode(signature_hex)?
        .try_into()
        .map_err(|_| RavaError::InvalidSignature)?;

    let verifying_key =
        VerifyingKey::from_bytes(&public_key_bytes).map_err(|_| RavaError::InvalidPublicKey)?;
    let signature = Signature::from_bytes(&signature_bytes);
    let canonical = canonical_json(payload)?;

    Ok(verifying_key
        .verify(canonical.as_bytes(), &signature)
        .is_ok())
}

fn signer_id_matches_public_key(signer_id: &str, public_key_hex: &str) -> bool {
    if public_key_hex.len() < 32 {
        return false;
    }

    let Some(rest) = signer_id.strip_prefix("rava:") else {
        return false;
    };
    let Some((kind, key_prefix)) = rest.split_once(':') else {
        return false;
    };
    if !matches!(kind, "human" | "agent" | "service" | "runtime") {
        return false;
    }

    key_prefix == &public_key_hex[..32]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn signer_signs_and_verifies_payload() -> Result<(), RavaError> {
        let signer = Signer::generate(SignerKind::Human);
        let payload = json!({ "intent": "book_flight", "amount_usd": 500 });

        let signature = signer.sign_json(&payload)?;

        assert!(verify_json_signature(
            &signer.id,
            &signer.public_key_hex,
            &payload,
            &signature
        )?);
        Ok(())
    }

    #[test]
    fn verifier_rejects_modified_payload() -> Result<(), RavaError> {
        let signer = Signer::generate(SignerKind::Agent);
        let signed_payload = json!({ "intent": "book_flight", "amount_usd": 500 });
        let modified_payload = json!({ "intent": "book_flight", "amount_usd": 900 });
        let signature = signer.sign_json(&signed_payload)?;

        assert!(!verify_json_signature(
            &signer.id,
            &signer.public_key_hex,
            &modified_payload,
            &signature
        )?);
        Ok(())
    }

    #[test]
    fn verifier_rejects_public_key_that_does_not_match_signer_id() -> Result<(), RavaError> {
        let claimed = Signer::generate(SignerKind::Agent);
        let attacker = Signer::generate(SignerKind::Agent);
        let payload = json!({ "intent": "book_flight", "amount_usd": 500 });
        let signature = attacker.sign_json(&payload)?;

        assert!(!verify_json_signature(
            &claimed.id,
            &attacker.public_key_hex,
            &payload,
            &signature
        )?);
        Ok(())
    }

    #[test]
    fn verifier_rejects_unknown_signer_kind() -> Result<(), RavaError> {
        let signer = Signer::generate(SignerKind::Agent);
        let signer_id = format!("rava:alien:{}", &signer.public_key_hex[..32]);
        let payload = json!({ "intent": "book_flight", "amount_usd": 500 });
        let signature = signer.sign_json(&payload)?;

        assert!(!verify_json_signature(
            &signer_id,
            &signer.public_key_hex,
            &payload,
            &signature
        )?);
        Ok(())
    }
}
