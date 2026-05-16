use std::error::Error;
use std::io;

use rava_core::identity::Signer;

use crate::cli::GenerateKeyArgs;
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
    write_signer_key_file(&args.out, &signer)?;
    println!("Rava key written: {}", args.out.display());
    println!("Rava key id: {}", signer.id);
    println!("Rava key public key: {}", signer.public_key_hex);
    Ok(())
}
