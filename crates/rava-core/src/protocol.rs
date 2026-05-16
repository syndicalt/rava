pub const ACTION_VERSION: &str = "rava-action-v0";
pub const CAPABILITY_VERSION: &str = "rava-capability-v0";
pub const VERIFICATION_RECEIPT_VERSION: &str = "rava-verification-receipt-v0";
pub const ATTESTATION_VERSION: &str = "rava-attestation-v0";

pub const ACTION_PENDING_ID: &str = "act_pending";
pub const CAPABILITY_PENDING_ID: &str = "cap_pending";
pub const VERIFICATION_RECEIPT_PENDING_ID: &str = "ver_pending";
pub const ATTESTATION_PENDING_ID: &str = "att_pending";

pub const ACTION_ID_PREFIX: &str = "act_";
pub const CAPABILITY_ID_PREFIX: &str = "cap_";
pub const VERIFICATION_RECEIPT_ID_PREFIX: &str = "ver_";
pub const ATTESTATION_ID_PREFIX: &str = "att_";
pub const SHA256_PREFIX: &str = "sha256:";

pub fn is_supported_action_version(version: &str) -> bool {
    version == ACTION_VERSION
}

pub fn is_supported_capability_version(version: &str) -> bool {
    version == CAPABILITY_VERSION
}

pub fn is_supported_verification_receipt_version(version: &str) -> bool {
    version == VERIFICATION_RECEIPT_VERSION
}

pub fn is_supported_attestation_version(version: &str) -> bool {
    version == ATTESTATION_VERSION
}
