#![no_main]

use std::collections::BTreeMap;

use libfuzzer_sys::fuzz_target;
use rava_core::action::ActionEnvelope;
use rava_core::attestation::{verify_attestation, Attestation};
use rava_core::audit::{verify_verification_receipt, VerificationReceipt};
use rava_core::canonical::canonical_json;
use rava_core::capability::Capability;
use rava_core::revocation::InMemoryRevocationRegistry;
use rava_core::verifier::{verify_action, VerifyActionInput};
use serde_json::Value;
use time::OffsetDateTime;

fuzz_target!(|data: &[u8]| {
    let Ok(value) = serde_json::from_slice::<Value>(data) else {
        return;
    };

    let _ = canonical_json(&value);

    if let Ok(action) = serde_json::from_value::<ActionEnvelope>(value.clone()) {
        let capability_chain = value
            .get("capability_chain")
            .cloned()
            .and_then(|chain| serde_json::from_value::<Vec<Capability>>(chain).ok())
            .unwrap_or_default();
        let issuer_keys = BTreeMap::new();
        let revocations = InMemoryRevocationRegistry::default();
        let now = OffsetDateTime::from_unix_timestamp(1_650_000_000)
            .unwrap_or(OffsetDateTime::UNIX_EPOCH);

        let _ = verify_action(VerifyActionInput {
            action: &action,
            capability_chain: &capability_chain,
            actor_public_key_hex: "",
            capability_issuer_public_keys: &issuer_keys,
            revocations: &revocations,
            now,
        });
    }

    if let Ok(receipt) = serde_json::from_value::<VerificationReceipt>(value.clone()) {
        let _ = verify_verification_receipt(&receipt, "");
    }

    if let Ok(attestation) = serde_json::from_value::<Attestation>(value) {
        let _ = verify_attestation(&attestation, "");
    }
});
