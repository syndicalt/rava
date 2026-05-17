#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use rava_core::action::ActionEnvelope;
use rava_core::capability::Capability;
use rava_core::revocation::InMemoryRevocationRegistry;
use rava_core::verifier::{verify_action, VerificationResult, VerifyActionInput};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use wasm_bindgen::prelude::*;

#[derive(Debug, Deserialize)]
struct VerifyActionJsonRequest {
    action: ActionEnvelope,
    capability_chain: Vec<Capability>,
    actor_public_key_hex: String,
    issuer_public_keys: BTreeMap<String, String>,
    now_unix: i64,
    #[serde(default)]
    revoked_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
struct VerifyActionJsonResponse {
    accepted: bool,
    rejection: Option<VerifyActionJsonRejection>,
}

#[derive(Debug, Serialize)]
struct VerifyActionJsonRejection {
    code: String,
    subject: Option<String>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn verify_action_json(request_json: &str) -> Result<String, JsValue> {
    verify_action_json_inner(request_json).map_err(|error| JsValue::from_str(&error))
}

fn verify_action_json_inner(request_json: &str) -> Result<String, String> {
    let request: VerifyActionJsonRequest =
        serde_json::from_str(request_json).map_err(|error| error.to_string())?;
    let now =
        OffsetDateTime::from_unix_timestamp(request.now_unix).map_err(|error| error.to_string())?;
    let mut revocations = InMemoryRevocationRegistry::default();
    for revoked_id in &request.revoked_ids {
        revocations.revoke(revoked_id.clone());
    }

    let result = verify_action(VerifyActionInput {
        action: &request.action,
        capability_chain: &request.capability_chain,
        actor_public_key_hex: &request.actor_public_key_hex,
        capability_issuer_public_keys: &request.issuer_public_keys,
        revocations: &revocations,
        now,
    })
    .map_err(|error| error.to_string())?;

    let response = match result {
        VerificationResult::Accepted => VerifyActionJsonResponse {
            accepted: true,
            rejection: None,
        },
        VerificationResult::Rejected(error) => VerifyActionJsonResponse {
            accepted: false,
            rejection: Some(VerifyActionJsonRejection {
                code: error.code().to_owned(),
                subject: error.subject(),
            }),
        },
    };

    serde_json::to_string(&response).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::error::Error;
    use std::path::{Path, PathBuf};

    use serde_json::Value;

    use super::verify_action_json;

    #[test]
    fn verify_action_json_accepts_v0_test_vector() -> Result<(), Box<dyn Error>> {
        let request = accepted_request()?;

        let response = call_verify_action_json(&request)?;
        let response: Value = serde_json::from_str(&response)?;

        assert_eq!(
            response.get("accepted").and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(response.get("rejection"), Some(&Value::Null));
        Ok(())
    }

    #[test]
    fn verify_action_json_preserves_rejection_codes() -> Result<(), Box<dyn Error>> {
        let mut request = accepted_request()?;
        request["action"]["constraints"]["amount_usd"]["integer"] = serde_json::json!(900);

        let response = call_verify_action_json(&request)?;
        let response: Value = serde_json::from_str(&response)?;

        assert_eq!(
            response.get("accepted").and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            response.pointer("/rejection/code").and_then(Value::as_str),
            Some("action_signature_invalid")
        );
        Ok(())
    }

    #[test]
    fn verify_action_json_uses_caller_supplied_revocations() -> Result<(), Box<dyn Error>> {
        let mut request = accepted_request()?;
        let capability_id = request["capability_chain"]
            .as_array()
            .and_then(|capabilities| capabilities.last())
            .and_then(|capability| capability.get("id"))
            .and_then(Value::as_str)
            .ok_or("missing final capability id")?
            .to_owned();
        request["revoked_ids"] = serde_json::json!([capability_id]);

        let response = call_verify_action_json(&request)?;
        let response: Value = serde_json::from_str(&response)?;

        assert_eq!(
            response.pointer("/rejection/code").and_then(Value::as_str),
            Some("capability_revoked")
        );
        Ok(())
    }

    fn accepted_request() -> Result<Value, Box<dyn Error>> {
        let vector = repository_root().join("test-vectors/v0/flight-booking");
        let action: Value = serde_json::from_slice(&std::fs::read(vector.join("action.json"))?)?;
        let capability_chain: Value =
            serde_json::from_slice(&std::fs::read(vector.join("capability-chain.json"))?)?;
        let keys: Value = serde_json::from_slice(&std::fs::read(vector.join("keys.json"))?)?;
        let actor_key = keys
            .get("actor_public_key_hex")
            .and_then(Value::as_str)
            .ok_or("missing actor_public_key_hex")?;
        let issuer_keys = keys
            .get("issuer_public_keys")
            .and_then(Value::as_object)
            .ok_or("missing issuer_public_keys")?
            .iter()
            .map(|(issuer, key)| {
                key.as_str()
                    .map(|public_key| (issuer.clone(), public_key.to_owned()))
                    .ok_or_else(|| format!("issuer key for {issuer} is not a string"))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;

        Ok(serde_json::json!({
            "action": action,
            "capability_chain": capability_chain,
            "actor_public_key_hex": actor_key,
            "issuer_public_keys": issuer_keys,
            "now_unix": 1650000000
        }))
    }

    fn call_verify_action_json(request: &Value) -> Result<String, Box<dyn Error>> {
        match verify_action_json(&serde_json::to_string(request)?) {
            Ok(response) => Ok(response),
            Err(error) => Err(format!("{error:?}").into()),
        }
    }

    fn repository_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }
}
