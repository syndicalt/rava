use std::error::Error;
use std::fs;
use std::io;

use rava_core::action::ActionEnvelope;
use rava_core::capability::Capability;

use crate::cli::{InspectActionArgs, InspectCapabilityChainArgs};

pub fn run_inspect_action(args: InspectActionArgs) -> Result<(), Box<dyn Error>> {
    let action: ActionEnvelope = serde_json::from_slice(&fs::read(&args.action)?)?;

    println!("Rava inspection only: true");
    println!("Rava object: action");
    println!("Rava action id: {}", action.id);
    println!("Rava actor: {}", action.actor);
    println!("Rava controller: {}", action.controller);
    println!("Rava intent: {}", action.intent);
    println!("Rava resource: {}", action.resource);
    println!("Rava operation: {}", action.operation);
    println!("Rava capability id: {}", action.capability_id);
    Ok(())
}

pub fn run_inspect_capability_chain(
    args: InspectCapabilityChainArgs,
) -> Result<(), Box<dyn Error>> {
    let capability_chain: Vec<Capability> =
        serde_json::from_slice(&fs::read(&args.capability_chain)?)?;
    let root = capability_chain.first().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "capability chain must contain at least one capability",
        )
    })?;
    let final_capability = capability_chain
        .last()
        .ok_or("capability chain must contain at least one capability")?;

    println!("Rava inspection only: true");
    println!("Rava object: capability-chain");
    println!("Rava capability count: {}", capability_chain.len());
    println!("Rava root issuer: {}", root.issuer);
    println!("Rava final subject: {}", final_capability.subject);
    println!("Rava final resource: {}", final_capability.resource);
    println!(
        "Rava final operations: {}",
        final_capability.operations.join(",")
    );
    Ok(())
}
