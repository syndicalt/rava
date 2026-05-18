use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;

use rava_core::action::ActionEnvelope;
use rava_core::attestation::{verify_attestation, Attestation};
use rava_core::audit::{
    sign_verification_receipt, verify_verification_receipt, VerificationReceipt,
    VerificationReceiptInput,
};
use rava_core::capability::Capability;
use rava_core::identity::{Signer, SignerKind};
use rava_core::replay::FileReplayRegistry;
use rava_core::revocation::{
    FileRevocationRegistry, InMemoryRevocationRegistry, RevocationRegistry,
};
use rava_core::verifier::{
    verify_action, verify_action_once, VerificationResult, VerifyActionInput, VerifyActionOnceInput,
};
use serde::Deserialize;
use time::OffsetDateTime;

use crate::cli::{VerifyActionArgs, VerifyAttestationArgs, VerifyReceiptArgs};
use crate::key_file::read_signer_key_file;

pub fn run_verify_action(args: VerifyActionArgs) -> Result<(), Box<dyn Error>> {
    let action: ActionEnvelope = serde_json::from_slice(&fs::read(&args.action)?)?;
    let capability_chain: Vec<Capability> =
        serde_json::from_slice(&fs::read(&args.capability_chain)?)?;
    let trust_bundle = match &args.trust_bundle {
        Some(path) => Some(read_static_trust_bundle(path)?),
        None => None,
    };
    let actor_key = select_actor_public_key(
        &action.actor,
        args.actor_key.as_deref(),
        trust_bundle.as_ref(),
    )?;
    let issuer_keys = resolve_issuer_keys(&args.issuer_keys, trust_bundle.as_ref())?;
    let now = match args.now_unix {
        Some(seconds) => OffsetDateTime::from_unix_timestamp(seconds)?,
        None => OffsetDateTime::now_utc(),
    };

    let result = if let Some(revocation_store) = &args.revocation_store {
        let revocations = FileRevocationRegistry::open(revocation_store)?;
        verify_action_with_registries(
            &action,
            &capability_chain,
            &actor_key,
            &issuer_keys,
            &revocations,
            args.replay_store.as_ref(),
            now,
        )?
    } else {
        let revocations = InMemoryRevocationRegistry::default();
        verify_action_with_registries(
            &action,
            &capability_chain,
            &actor_key,
            &issuer_keys,
            &revocations,
            args.replay_store.as_ref(),
            now,
        )?
    };

    println!(
        "Rava verification accepted: {}",
        result == VerificationResult::Accepted
    );
    if let VerificationResult::Rejected(error) = &result {
        println!("Rava rejection: {error:?}");
    }
    if args.receipt_key.is_some() && args.receipt_out.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "receipt-key requires receipt-out",
        )
        .into());
    }
    if let Some(receipt_out) = args.receipt_out {
        let verifier = match &args.receipt_key {
            Some(path) => read_signer_key_file(path)?,
            None => Signer::generate(SignerKind::Service),
        };
        let receipt = sign_verification_receipt(
            &verifier,
            VerificationReceiptInput {
                action: &action,
                capability_chain: &capability_chain,
                result: &result,
                verified_at: now,
            },
        )?;
        fs::write(receipt_out, serde_json::to_vec_pretty(&receipt)?)?;
        println!("Rava receipt verifier: {}", verifier.id);
        println!(
            "Rava receipt verifier public key: {}",
            verifier.public_key_hex
        );
    }

    Ok(())
}

pub fn run_verify_attestation(args: VerifyAttestationArgs) -> Result<(), Box<dyn Error>> {
    let attestation: Attestation = serde_json::from_slice(&fs::read(&args.attestation)?)?;
    let verified = verify_attestation(&attestation, &args.evaluator_key)?;
    println!("Rava attestation verified: {verified}");
    Ok(())
}

pub fn run_verify_receipt(args: VerifyReceiptArgs) -> Result<(), Box<dyn Error>> {
    let receipt: VerificationReceipt = serde_json::from_slice(&fs::read(&args.receipt)?)?;
    let verified = verify_verification_receipt(&receipt, &args.verifier_key)?;
    println!("Rava receipt verified: {verified}");
    Ok(())
}

fn verify_action_with_registries<R: RevocationRegistry>(
    action: &ActionEnvelope,
    capability_chain: &[Capability],
    actor_public_key_hex: &str,
    issuer_keys: &BTreeMap<String, String>,
    revocations: &R,
    replay_store: Option<&PathBuf>,
    now: OffsetDateTime,
) -> Result<VerificationResult, Box<dyn Error>> {
    let result = if let Some(replay_store) = replay_store {
        let mut replay = FileReplayRegistry::open(replay_store)?;
        verify_action_once(VerifyActionOnceInput {
            action,
            capability_chain,
            actor_public_key_hex,
            capability_issuer_public_keys: issuer_keys,
            revocations,
            replay: &mut replay,
            now,
        })?
    } else {
        verify_action(VerifyActionInput {
            action,
            capability_chain,
            actor_public_key_hex,
            capability_issuer_public_keys: issuer_keys,
            revocations,
            now,
        })?
    };

    Ok(result)
}

fn parse_issuer_keys(entries: &[String]) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
    let mut keys = BTreeMap::new();

    for entry in entries {
        let (issuer, public_key) = entry.split_once('=').ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "issuer-key must use issuer=public_key_hex",
            )
        })?;
        if issuer.is_empty() || public_key.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "issuer-key issuer and public key must be non-empty",
            )
            .into());
        }
        keys.insert(issuer.to_owned(), public_key.to_owned());
    }

    Ok(keys)
}

#[derive(Debug, Deserialize)]
struct StaticTrustBundle {
    version: String,
    keys: BTreeMap<String, String>,
}

fn read_static_trust_bundle(path: &PathBuf) -> Result<StaticTrustBundle, Box<dyn Error>> {
    let bundle: StaticTrustBundle = serde_json::from_slice(&fs::read(path)?)?;
    if bundle.version != "rava-static-trust-bundle-v0" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported trust bundle version",
        )
        .into());
    }
    for (signer_id, public_key) in &bundle.keys {
        if signer_id.is_empty() || public_key.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "trust bundle signer IDs and public keys must be non-empty",
            )
            .into());
        }
    }
    Ok(bundle)
}

fn select_actor_public_key(
    actor: &str,
    actor_key: Option<&str>,
    trust_bundle: Option<&StaticTrustBundle>,
) -> Result<String, Box<dyn Error>> {
    let bundled_key = trust_bundle.and_then(|bundle| bundle.keys.get(actor));
    match (actor_key, bundled_key) {
        (Some(explicit), Some(bundled)) if explicit != bundled => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "actor-key conflicts with trust-bundle actor key",
        )
        .into()),
        (Some(explicit), _) => Ok(explicit.to_owned()),
        (None, Some(bundled)) => Ok(bundled.clone()),
        (None, None) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "actor public key requires --actor-key or a trust-bundle entry for the action actor",
        )
        .into()),
    }
}

fn resolve_issuer_keys(
    entries: &[String],
    trust_bundle: Option<&StaticTrustBundle>,
) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
    let mut keys = parse_issuer_keys(entries)?;
    if let Some(bundle) = trust_bundle {
        for (signer_id, public_key) in &bundle.keys {
            match keys.get(signer_id) {
                Some(explicit) if explicit != public_key => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "issuer-key conflicts with trust-bundle signer key",
                    )
                    .into());
                }
                Some(_) => {}
                None => {
                    keys.insert(signer_id.clone(), public_key.clone());
                }
            }
        }
    }
    Ok(keys)
}
