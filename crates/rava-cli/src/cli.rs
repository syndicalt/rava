use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "rava")]
#[command(about = "Action-native authorization for autonomous agents")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Version,
    Attest {
        #[command(subcommand)]
        command: AttestCommand,
    },
    Demo {
        #[command(subcommand)]
        command: DemoCommand,
    },
    Key {
        #[command(subcommand)]
        command: KeyCommand,
    },
    Serve {
        #[command(subcommand)]
        command: ServeCommand,
    },
    Inspect {
        #[command(subcommand)]
        command: InspectCommand,
    },
    Verify {
        #[command(subcommand)]
        command: VerifyCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum AttestCommand {
    Sign(SignAttestationArgs),
}

#[derive(Debug, Parser)]
pub struct SignAttestationArgs {
    #[arg(long)]
    pub key: PathBuf,

    #[arg(long)]
    pub out: PathBuf,

    #[arg(long = "action-id")]
    pub action_id: String,

    #[arg(long)]
    pub outcome: String,

    #[arg(long)]
    pub subject: String,

    #[arg(long = "occurred-at-unix")]
    pub occurred_at_unix: i64,

    #[arg(long = "evidence-hash")]
    pub evidence_hash: String,
}

#[derive(Debug, Subcommand)]
pub enum DemoCommand {
    FlightBooking(FlightBookingDemoArgs),
}

#[derive(Debug, Parser)]
pub struct FlightBookingDemoArgs {
    #[arg(long = "write-fixtures")]
    pub write_fixtures: Option<PathBuf>,

    #[arg(long = "deterministic-fixtures")]
    pub deterministic_fixtures: bool,
}

#[derive(Debug, Subcommand)]
pub enum KeyCommand {
    Generate(GenerateKeyArgs),
}

#[derive(Debug, Parser)]
pub struct GenerateKeyArgs {
    #[arg(long)]
    pub kind: String,

    #[arg(long)]
    pub out: PathBuf,

    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Subcommand)]
pub enum ServeCommand {
    Verify(ServeVerifyArgs),
}

#[derive(Debug, Parser)]
pub struct ServeVerifyArgs {
    #[arg(long, default_value = "127.0.0.1:8787")]
    pub addr: String,

    #[arg(long = "max-request-bytes", default_value_t = 1_048_576)]
    pub max_request_bytes: usize,

    #[arg(long = "replay-store")]
    pub replay_store: Option<PathBuf>,

    #[arg(long = "revocation-store")]
    pub revocation_store: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum InspectCommand {
    Action(InspectActionArgs),
    CapabilityChain(InspectCapabilityChainArgs),
}

#[derive(Debug, Parser)]
pub struct InspectActionArgs {
    #[arg(long)]
    pub action: PathBuf,
}

#[derive(Debug, Parser)]
pub struct InspectCapabilityChainArgs {
    #[arg(long = "capability-chain")]
    pub capability_chain: PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum VerifyCommand {
    Action(VerifyActionArgs),
    Attestation(VerifyAttestationArgs),
    Receipt(VerifyReceiptArgs),
}

#[derive(Debug, Parser)]
pub struct VerifyActionArgs {
    #[arg(long)]
    pub action: PathBuf,

    #[arg(long = "capability-chain")]
    pub capability_chain: PathBuf,

    #[arg(long = "actor-key")]
    pub actor_key: String,

    #[arg(long = "issuer-key")]
    pub issuer_keys: Vec<String>,

    #[arg(long = "now-unix")]
    pub now_unix: Option<i64>,

    #[arg(long = "replay-store")]
    pub replay_store: Option<PathBuf>,

    #[arg(long = "revocation-store")]
    pub revocation_store: Option<PathBuf>,

    #[arg(long = "receipt-out")]
    pub receipt_out: Option<PathBuf>,

    #[arg(long = "receipt-key")]
    pub receipt_key: Option<PathBuf>,
}

#[derive(Debug, Parser)]
pub struct VerifyReceiptArgs {
    #[arg(long)]
    pub receipt: PathBuf,

    #[arg(long = "verifier-key")]
    pub verifier_key: String,
}

#[derive(Debug, Parser)]
pub struct VerifyAttestationArgs {
    #[arg(long)]
    pub attestation: PathBuf,

    #[arg(long = "evaluator-key")]
    pub evaluator_key: String,
}
