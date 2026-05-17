use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;

use rava_core::action::{sign_action, ActionEnvelope, ActionInput};
use rava_core::attestation::{
    attestation_signing_payload, expected_attestation_id, sign_attestation, Attestation,
    AttestationInput, AttestationOutcome,
};
use rava_core::audit::{
    expected_verification_receipt_id, sign_verification_receipt,
    verification_receipt_signing_payload, verify_verification_receipt, VerificationReceipt,
    VerificationReceiptInput,
};
use rava_core::capability::{
    capability_signing_payload, delegate_capability, expected_capability_id, mint_capability,
    Capability, CapabilityInput, ConstraintValue, DelegationInput,
};
use rava_core::identity::{Signer, SignerKind};
use rava_core::replay::InMemoryReplayRegistry;
use rava_core::revocation::InMemoryRevocationRegistry;
use rava_core::verifier::{
    verify_action_once, VerificationError, VerificationResult, VerifyActionOnceInput,
};

use crate::cli::FlightBookingDemoArgs;
use crate::timestamp;

const HUMAN_FIXTURE_KEY: &str = "0101010101010101010101010101010101010101010101010101010101010101";
const PERSONAL_AGENT_FIXTURE_KEY: &str =
    "0202020202020202020202020202020202020202020202020202020202020202";
const BOOKING_AGENT_FIXTURE_KEY: &str =
    "0303030303030303030303030303030303030303030303030303030303030303";
const VERIFIER_FIXTURE_KEY: &str =
    "0404040404040404040404040404040404040404040404040404040404040404";
const EVALUATOR_FIXTURE_KEY: &str =
    "0505050505050505050505050505050505050505050505050505050505050505";

pub fn run_flight_booking_demo(args: FlightBookingDemoArgs) -> Result<(), Box<dyn Error>> {
    if args.deterministic_fixtures && args.write_fixtures.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--deterministic-fixtures requires --write-fixtures",
        )
        .into());
    }
    if args.deterministic_fixtures {
        let directory = args
            .write_fixtures
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing fixtures path"))?;
        let fixture_set = deterministic_flight_booking_fixture_set()?;
        write_flight_booking_fixture_set(&directory, &fixture_set)?;
        println!(
            "Rava deterministic fixtures written: {}",
            directory.display()
        );
        return Ok(());
    }

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

struct FlightBookingFixtureSet {
    action: ActionEnvelope,
    capability_chain: Vec<Capability>,
    actor_public_key_hex: String,
    issuer_keys: BTreeMap<String, String>,
    verifier: Signer,
    evaluator: Signer,
    receipt: VerificationReceipt,
    attestation: Attestation,
    tampered_receipt: VerificationReceipt,
    tampered_attestation: Attestation,
}

fn deterministic_flight_booking_fixture_set() -> Result<FlightBookingFixtureSet, Box<dyn Error>> {
    let human = Signer::from_signing_key_hex(SignerKind::Human, HUMAN_FIXTURE_KEY)?;
    let personal_agent =
        Signer::from_signing_key_hex(SignerKind::Agent, PERSONAL_AGENT_FIXTURE_KEY)?;
    let booking_agent = Signer::from_signing_key_hex(SignerKind::Agent, BOOKING_AGENT_FIXTURE_KEY)?;
    let verifier = Signer::from_signing_key_hex(SignerKind::Service, VERIFIER_FIXTURE_KEY)?;
    let evaluator = Signer::from_signing_key_hex(SignerKind::Service, EVALUATOR_FIXTURE_KEY)?;

    let mut root = mint_capability(
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
    resign_capability(&human, &mut root, "11111111-1111-4111-8111-111111111111")?;

    let mut purchase = delegate_capability(
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
    resign_capability(
        &personal_agent,
        &mut purchase,
        "22222222-2222-4222-8222-222222222222",
    )?;

    let mut action = sign_action(
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
    resign_action(
        &booking_agent,
        &mut action,
        "33333333-3333-4333-8333-333333333333",
    )?;

    let capability_chain = vec![root, purchase];
    let result = VerificationResult::Accepted;
    let mut receipt = sign_verification_receipt(
        &verifier,
        VerificationReceiptInput {
            action: &action,
            capability_chain: &capability_chain,
            result: &result,
            verified_at: timestamp(1_650_000_000)?,
        },
    )?;
    resign_receipt(
        &verifier,
        &mut receipt,
        "44444444-4444-4444-8444-444444444444",
    )?;

    let mut attestation = sign_attestation(
        &evaluator,
        AttestationInput {
            action_id: action.id.clone(),
            outcome: AttestationOutcome::Accepted,
            subject: "travel.booking".to_owned(),
            occurred_at: timestamp(1_650_000_030)?,
            evidence_hash:
                "sha256:3333333333333333333333333333333333333333333333333333333333333333".to_owned(),
        },
    )?;
    resign_attestation(
        &evaluator,
        &mut attestation,
        "55555555-5555-4555-8555-555555555555",
    )?;

    let mut tampered_receipt = receipt.clone();
    tampered_receipt.capability_chain_hash =
        "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_owned();

    let mut tampered_attestation = attestation.clone();
    tampered_attestation.evidence_hash =
        "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_owned();

    let issuer_keys = BTreeMap::from([
        (
            capability_chain[0].issuer.clone(),
            human.public_key_hex.clone(),
        ),
        (
            capability_chain[1].issuer.clone(),
            personal_agent.public_key_hex.clone(),
        ),
    ]);

    Ok(FlightBookingFixtureSet {
        action,
        capability_chain,
        actor_public_key_hex: booking_agent.public_key_hex,
        issuer_keys,
        verifier,
        evaluator,
        receipt,
        attestation,
        tampered_receipt,
        tampered_attestation,
    })
}

fn resign_capability(
    signer: &Signer,
    capability: &mut Capability,
    nonce: &str,
) -> Result<(), Box<dyn Error>> {
    capability.nonce = nonce.to_owned();
    capability.id = expected_capability_id(capability)?;
    capability.proof.signature_hex = signer.sign_json(&capability_signing_payload(capability))?;
    Ok(())
}

fn resign_action(
    signer: &Signer,
    action: &mut ActionEnvelope,
    nonce: &str,
) -> Result<(), Box<dyn Error>> {
    action.nonce = nonce.to_owned();
    action.id = rava_core::action::expected_action_id(action)?;
    action.proof.signature_hex =
        signer.sign_json(&rava_core::action::action_signing_payload(action))?;
    Ok(())
}

fn resign_receipt(
    signer: &Signer,
    receipt: &mut VerificationReceipt,
    nonce: &str,
) -> Result<(), Box<dyn Error>> {
    receipt.nonce = nonce.to_owned();
    receipt.id = expected_verification_receipt_id(receipt)?;
    receipt.proof.signature_hex =
        signer.sign_json(&verification_receipt_signing_payload(receipt))?;
    Ok(())
}

fn resign_attestation(
    signer: &Signer,
    attestation: &mut Attestation,
    nonce: &str,
) -> Result<(), Box<dyn Error>> {
    attestation.nonce = nonce.to_owned();
    attestation.id = expected_attestation_id(attestation)?;
    attestation.proof.signature_hex =
        signer.sign_json(&attestation_signing_payload(attestation))?;
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

fn write_flight_booking_fixture_set(
    directory: &PathBuf,
    fixtures: &FlightBookingFixtureSet,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(directory)?;
    fs::write(
        directory.join("action.json"),
        serde_json::to_vec_pretty(&fixtures.action)?,
    )?;
    fs::write(
        directory.join("capability-chain.json"),
        serde_json::to_vec_pretty(&fixtures.capability_chain)?,
    )?;
    fs::write(
        directory.join("keys.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "actor": fixtures.action.actor,
            "actor_public_key_hex": fixtures.actor_public_key_hex,
            "issuer_public_keys": fixtures.issuer_keys,
            "verifier": fixtures.verifier.id,
            "verifier_public_key_hex": fixtures.verifier.public_key_hex,
            "evaluator": fixtures.evaluator.id,
            "evaluator_public_key_hex": fixtures.evaluator.public_key_hex,
        }))?,
    )?;
    fs::write(
        directory.join("receipt.json"),
        serde_json::to_vec_pretty(&fixtures.receipt)?,
    )?;
    fs::write(
        directory.join("attestation.json"),
        serde_json::to_vec_pretty(&fixtures.attestation)?,
    )?;
    fs::write(
        directory.join("tampered-receipt.json"),
        serde_json::to_vec_pretty(&fixtures.tampered_receipt)?,
    )?;
    fs::write(
        directory.join("tampered-attestation.json"),
        serde_json::to_vec_pretty(&fixtures.tampered_attestation)?,
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
