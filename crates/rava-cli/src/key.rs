use std::error::Error;
use std::io;

use rava_core::identity::Signer;
use rava_core::revocation::FileRevocationRegistry;

use crate::cli::{GenerateKeyArgs, RevokeKeyArgs};
use crate::key_file::{parse_signer_kind, write_signer_key_file};

pub fn run_key_generate(args: GenerateKeyArgs) -> Result<(), Box<dyn Error>> {
    if args.out.exists() && !args.force {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "key file already exists; pass --force to overwrite",
        )
        .into());
    }
    let signer = Signer::generate(parse_signer_kind(&args.kind)?);
    write_signer_key_file(&args.out, &signer, args.force)?;
    println!("Rava key written: {}", args.out.display());
    println!("Rava key id: {}", signer.id);
    println!("Rava key public key: {}", signer.public_key_hex);
    Ok(())
}

pub fn run_key_revoke(args: RevokeKeyArgs) -> Result<(), Box<dyn Error>> {
    if args.id.trim().is_empty() {
        return Err("revoked signer id must not be empty".into());
    }

    let mut revocations = FileRevocationRegistry::open(&args.revocation_store)?;
    revocations.revoke_and_persist(args.id.clone())?;
    println!("Rava key revoked: {}", args.id);
    println!(
        "Rava revocation store updated: {}",
        args.revocation_store.display()
    );
    Ok(())
}
