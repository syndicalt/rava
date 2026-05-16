use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use rava_core::action::{sign_action, ActionEnvelope, ActionInput};
use rava_core::audit::{
    sign_verification_receipt, verify_verification_receipt, VerificationReceiptInput,
};
use rava_core::capability::{
    delegate_capability, mint_capability, Capability, CapabilityInput, ConstraintValue,
    DelegationInput,
};
use rava_core::identity::{Signer, SignerKind};
use rava_core::replay::InMemoryReplayRegistry;
use rava_core::revocation::InMemoryRevocationRegistry;
use rava_core::verifier::{
    verify_action_once, VerificationError, VerificationResult, VerifyActionOnceInput,
};

use crate::cli::FlightBookingDemoArgs;
use crate::timestamp;

pub fn run_flight_booking_demo(args: FlightBookingDemoArgs) -> Result<(), Box<dyn Error>> {
    let human = Signer::generate(SignerKind::Human);
    let personal_agent = Signer::generate(SignerKind::Agent);
    let booking_agent = Signer::generate(SignerKind::Agent);
    let verifier = Signer::generate(SignerKind::Service);

    let root = mint_capability(
        &human,
        CapabilityInput {
            subject: personal_agent.id.clone(),
            resource: "travel.booking".to_owned(),
            operations: vec!["purchase".to_owned()],
            constraints: max_amount(1_200),
            expires_at: timestamp(1_800_000_000)?,
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
            expires_at: timestamp(1_700_000_000)?,
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
            context_hash: "sha256:2222222222222222222222222222222222222222222222222222222222222222"
                .to_owned(),
        },
    )?;
    let issuer_keys = BTreeMap::from([
        (root.issuer.clone(), human.public_key_hex.clone()),
        (
            purchase.issuer.clone(),
            personal_agent.public_key_hex.clone(),
        ),
    ]);
    let capability_chain = vec![root.clone(), purchase.clone()];
    if let Some(directory) = args.write_fixtures {
        write_flight_booking_fixtures(
            &directory,
            &action,
            &capability_chain,
            &booking_agent.public_key_hex,
            &issuer_keys,
        )?;
        println!("Rava fixtures written: {}", directory.display());
        return Ok(());
    }

    let revocations = InMemoryRevocationRegistry::default();
    let mut replay = InMemoryReplayRegistry::default();
    let result = verify_action_once(VerifyActionOnceInput {
        action: &action,
        capability_chain: &capability_chain,
        actor_public_key_hex: &booking_agent.public_key_hex,
        capability_issuer_public_keys: &issuer_keys,
        revocations: &revocations,
        replay: &mut replay,
        now: timestamp(1_650_000_000)?,
    })?;
    let replay_result = verify_action_once(VerifyActionOnceInput {
        action: &action,
        capability_chain: &capability_chain,
        actor_public_key_hex: &booking_agent.public_key_hex,
        capability_issuer_public_keys: &issuer_keys,
        revocations: &revocations,
        replay: &mut replay,
        now: timestamp(1_650_000_000)?,
    })?;

    println!(
        "Rava verification accepted: {}",
        result == VerificationResult::Accepted
    );
    println!(
        "Rava replay rejected: {}",
        matches!(
            replay_result,
            VerificationResult::Rejected(VerificationError::ActionReplayed { .. })
        )
    );
    let receipt = sign_verification_receipt(
        &verifier,
        VerificationReceiptInput {
            action: &action,
            capability_chain: &capability_chain,
            result: &result,
            verified_at: timestamp(1_650_000_000)?,
        },
    )?;
    println!(
        "Rava receipt verified: {}",
        verify_verification_receipt(&receipt, &verifier.public_key_hex)?
    );
    Ok(())
}

fn write_flight_booking_fixtures(
    directory: &PathBuf,
    action: &ActionEnvelope,
    capability_chain: &[Capability],
    actor_public_key_hex: &str,
    issuer_keys: &BTreeMap<String, String>,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(directory)?;
    fs::write(
        directory.join("action.json"),
        serde_json::to_vec_pretty(action)?,
    )?;
    fs::write(
        directory.join("capability-chain.json"),
        serde_json::to_vec_pretty(capability_chain)?,
    )?;
    fs::write(
        directory.join("keys.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "actor": action.actor,
            "actor_public_key_hex": actor_public_key_hex,
            "issuer_public_keys": issuer_keys,
        }))?,
    )?;
    fs::write(
        directory.join("replay.json"),
        serde_json::to_vec_pretty(&serde_json::json!({ "action_ids": [] }))?,
    )?;
    fs::write(
        directory.join("revocations.json"),
        serde_json::to_vec_pretty(&serde_json::json!({ "revoked_ids": [] }))?,
    )?;
    Ok(())
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
