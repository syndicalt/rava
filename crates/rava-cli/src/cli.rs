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
    Audit {
        #[command(subcommand)]
        command: AuditCommand,
    },
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
pub enum AuditCommand {
    Export(ExportAuditArgs),
}

#[derive(Debug, Parser)]
pub struct ExportAuditArgs {
    #[arg(long = "audit-log")]
    pub audit_log: PathBuf,

    #[arg(long = "since-unix")]
    pub since_unix: Option<i64>,

    #[arg(long = "until-unix")]
    pub until_unix: Option<i64>,

    #[arg(long)]
    pub output: Option<PathBuf>,
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
    Revoke(RevokeKeyArgs),
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

#[derive(Debug, Parser)]
pub struct RevokeKeyArgs {
    #[arg(long)]
    pub id: String,

    #[arg(long = "revocation-store")]
    pub revocation_store: PathBuf,
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

    #[arg(long = "require-replay-store")]
    pub require_replay_store: bool,

    #[arg(long = "revocation-store")]
    pub revocation_store: Option<PathBuf>,

    #[arg(long = "require-fresh-revocations")]
    pub require_fresh_revocations: bool,

    #[arg(long = "audit-log")]
    pub audit_log: Option<PathBuf>,

    #[arg(long = "require-audit-log")]
    pub require_audit_log: bool,

    #[arg(long = "auth-token-env")]
    pub auth_token_env: Option<String>,

    #[arg(long = "require-auth-token-env")]
    pub require_auth_token_env: bool,

    #[arg(long = "caller-id")]
    pub caller_id: Option<String>,

    #[arg(long = "require-caller-id")]
    pub require_caller_id: bool,

    #[arg(long = "rate-limit-per-minute")]
    pub rate_limit_per_minute: Option<usize>,

    #[arg(long = "require-rate-limit-per-minute")]
    pub require_rate_limit_per_minute: bool,

    #[arg(long)]
    pub metrics: bool,

    #[arg(long = "require-metrics")]
    pub require_metrics: bool,
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
    pub actor_key: Option<String>,

    #[arg(long = "issuer-key")]
    pub issuer_keys: Vec<String>,

    #[arg(long = "trust-bundle")]
    pub trust_bundle: Option<PathBuf>,

    #[arg(long = "require-fresh-trust-bundle")]
    pub require_fresh_trust_bundle: bool,

    #[arg(long = "now-unix")]
    pub now_unix: Option<i64>,

    #[arg(long = "replay-store")]
    pub replay_store: Option<PathBuf>,

    #[arg(long = "revocation-store")]
    pub revocation_store: Option<PathBuf>,

    #[arg(long = "require-fresh-revocations")]
    pub require_fresh_revocations: bool,

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
