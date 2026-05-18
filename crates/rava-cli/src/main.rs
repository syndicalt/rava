use std::process::ExitCode;

use clap::Parser;
use time::OffsetDateTime;

mod attest;
mod audit;
mod cli;
mod demo;
mod inspect;
mod key;
mod key_file;
mod serve;
mod verify;

use attest::run_attest_sign;
use audit::run_audit_export;
use cli::{
    AttestCommand, AuditCommand, Cli, Command, DemoCommand, InspectCommand, KeyCommand,
    ServeCommand, VerifyCommand,
};
use demo::run_flight_booking_demo;
use inspect::{run_inspect_action, run_inspect_capability_chain};
use key::run_key_generate;
use serve::run_serve_verify;
use verify::{run_verify_action, run_verify_attestation, run_verify_receipt};

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Version => {
            println!("{}", rava_core::version());
            Ok(())
        }
        Command::Audit {
            command: AuditCommand::Export(args),
        } => run_audit_export(args),
        Command::Attest {
            command: AttestCommand::Sign(args),
        } => run_attest_sign(args),
        Command::Demo {
            command: DemoCommand::FlightBooking(args),
        } => run_flight_booking_demo(args),
        Command::Key {
            command: KeyCommand::Generate(args),
        } => run_key_generate(args),
        Command::Serve {
            command: ServeCommand::Verify(args),
        } => run_serve_verify(args),
        Command::Inspect {
            command: InspectCommand::Action(args),
        } => run_inspect_action(args),
        Command::Inspect {
            command: InspectCommand::CapabilityChain(args),
        } => run_inspect_capability_chain(args),
        Command::Verify {
            command: VerifyCommand::Action(args),
        } => run_verify_action(args),
        Command::Verify {
            command: VerifyCommand::Attestation(args),
        } => run_verify_attestation(args),
        Command::Verify {
            command: VerifyCommand::Receipt(args),
        } => run_verify_receipt(args),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rava: {error}");
            ExitCode::FAILURE
        }
    }
}

fn timestamp(seconds: i64) -> Result<OffsetDateTime, time::error::ComponentRange> {
    OffsetDateTime::from_unix_timestamp(seconds)
}
