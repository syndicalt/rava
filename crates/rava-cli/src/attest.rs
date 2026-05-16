use std::error::Error;
use std::fs;
use std::io;

use rava_core::attestation::{sign_attestation, AttestationInput, AttestationOutcome};

use crate::cli::SignAttestationArgs;
use crate::key_file::read_signer_key_file;
use crate::timestamp;

pub fn run_attest_sign(args: SignAttestationArgs) -> Result<(), Box<dyn Error>> {
    let evaluator = read_signer_key_file(&args.key)?;
    let attestation = sign_attestation(
        &evaluator,
        AttestationInput {
            action_id: args.action_id,
            outcome: parse_attestation_outcome(&args.outcome)?,
            subject: args.subject,
            occurred_at: timestamp(args.occurred_at_unix)?,
            evidence_hash: args.evidence_hash,
        },
    )?;
    fs::write(&args.out, serde_json::to_vec_pretty(&attestation)?)?;
    println!("Rava attestation written: {}", args.out.display());
    println!("Rava attestation evaluator: {}", evaluator.id);
    println!(
        "Rava attestation evaluator public key: {}",
        evaluator.public_key_hex
    );
    Ok(())
}

fn parse_attestation_outcome(outcome: &str) -> Result<AttestationOutcome, Box<dyn Error>> {
    match outcome {
        "accepted" => Ok(AttestationOutcome::Accepted),
        "rejected" => Ok(AttestationOutcome::Rejected),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "attestation outcome must be accepted or rejected",
        )
        .into()),
    }
}
