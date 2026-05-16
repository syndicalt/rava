use crate::canonical::CanonicalError;

#[derive(Debug, thiserror::Error)]
pub enum RavaError {
    #[error(transparent)]
    Canonical(#[from] CanonicalError),

    #[error("invalid hex: {0}")]
    Hex(#[from] hex::FromHexError),

    #[error("invalid signature")]
    InvalidSignature,

    #[error("invalid public key")]
    InvalidPublicKey,

    #[error("invalid signing key")]
    InvalidSigningKey,
}
