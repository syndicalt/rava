use std::error::Error;
use std::fs;
use std::process::Command;

use rava_core::action::{sign_action, ActionInput};
use rava_core::capability::{
    delegate_capability, mint_capability, CapabilityInput, ConstraintValue, DelegationInput,
};
use rava_core::identity::{Signer, SignerKind};
use time::OffsetDateTime;

use crate::temp::temp_directory;

pub struct Scenario {
    pub human: Signer,
    pub personal_agent: Signer,
    pub booking_agent: Signer,
    pub action: rava_core::action::ActionEnvelope,
    pub chain: Vec<rava_core::capability::Capability>,
}

pub struct ScenarioFiles {
    pub directory: std::path::PathBuf,
    pub action_path: std::path::PathBuf,
    pub chain_path: std::path::PathBuf,
}

impl Scenario {
    pub fn new(action_amount: u64) -> Result<Self, Box<dyn Error>> {
        let human = Signer::generate(SignerKind::Human);
        let personal_agent = Signer::generate(SignerKind::Agent);
        let booking_agent = Signer::generate(SignerKind::Agent);
        let root = mint_capability(
            &human,
            CapabilityInput {
                subject: personal_agent.id.clone(),
                resource: "travel.booking".to_owned(),
                operations: vec!["purchase".to_owned()],
                constraints: max_amount(1_200),
                expires_at: at(1_800_000_000)?,
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
                expires_at: at(1_700_000_000)?,
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
                constraints: amount(action_amount),
                capability_id: purchase.id.clone(),
                context_hash:
                    "sha256:2222222222222222222222222222222222222222222222222222222222222222"
                        .to_owned(),
            },
        )?;

        Ok(Self {
            human,
            personal_agent,
            booking_agent,
            action,
            chain: vec![root, purchase],
        })
    }

    pub fn write_files(&self, label: &str) -> Result<ScenarioFiles, Box<dyn Error>> {
        let directory = temp_directory(label)?;
        fs::create_dir_all(&directory)?;
        let action_path = directory.join("action.json");
        let chain_path = directory.join("chain.json");
        fs::write(&action_path, serde_json::to_vec_pretty(&self.action)?)?;
        fs::write(&chain_path, serde_json::to_vec_pretty(&self.chain)?)?;

        Ok(ScenarioFiles {
            directory,
            action_path,
            chain_path,
        })
    }

    pub fn run_verify_action(
        &self,
        files: &ScenarioFiles,
        extra_args: &[(&str, &std::path::Path)],
    ) -> Result<std::process::Output, Box<dyn Error>> {
        let mut command = Command::new(env!("CARGO_BIN_EXE_rava"));
        command.args([
            "verify",
            "action",
            "--action",
            files.action_path.to_str().ok_or("invalid action path")?,
            "--capability-chain",
            files.chain_path.to_str().ok_or("invalid chain path")?,
            "--actor-key",
            &self.booking_agent.public_key_hex,
            "--issuer-key",
            &format!("{}={}", self.human.id, self.human.public_key_hex),
            "--issuer-key",
            &format!(
                "{}={}",
                self.personal_agent.id, self.personal_agent.public_key_hex
            ),
            "--now-unix",
            "1650000000",
        ]);
        for (flag, path) in extra_args {
            command.args([*flag, path.to_str().ok_or("invalid extra arg path")?]);
        }

        Ok(command.output()?)
    }
}

fn at(seconds: i64) -> Result<OffsetDateTime, Box<dyn Error>> {
    Ok(OffsetDateTime::from_unix_timestamp(seconds)?)
}

fn max_amount(amount: u64) -> std::collections::BTreeMap<String, ConstraintValue> {
    std::collections::BTreeMap::from([(
        "max_amount_usd".to_owned(),
        ConstraintValue::Integer(amount),
    )])
}

fn amount(amount: u64) -> std::collections::BTreeMap<String, ConstraintValue> {
    std::collections::BTreeMap::from([("amount_usd".to_owned(), ConstraintValue::Integer(amount))])
}
