#![forbid(unsafe_code)]

pub mod action;
pub mod attestation;
pub mod audit;
pub mod canonical;
pub mod capability;
pub mod constraints;
pub mod error;
pub mod hash;
pub mod identity;
pub mod nonce;
pub mod protocol;
pub mod replay;
pub mod revocation;
pub mod verifier;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::capability::ConstraintValue;
    use crate::constraints;
    use crate::protocol;

    #[test]
    fn protocol_exports_v0_object_versions() {
        assert_eq!(protocol::ACTION_VERSION, "rava-action-v0");
        assert_eq!(protocol::CAPABILITY_VERSION, "rava-capability-v0");
        assert_eq!(
            protocol::VERIFICATION_RECEIPT_VERSION,
            "rava-verification-receipt-v0"
        );
        assert_eq!(protocol::ATTESTATION_VERSION, "rava-attestation-v0");
    }

    #[test]
    fn protocol_version_helpers_accept_only_v0_versions() {
        assert!(protocol::is_supported_action_version(
            protocol::ACTION_VERSION
        ));
        assert!(!protocol::is_supported_action_version("rava-action-v1"));
        assert!(protocol::is_supported_capability_version(
            protocol::CAPABILITY_VERSION
        ));
        assert!(!protocol::is_supported_capability_version(
            "rava-capability-v1"
        ));
        assert!(protocol::is_supported_verification_receipt_version(
            protocol::VERIFICATION_RECEIPT_VERSION
        ));
        assert!(!protocol::is_supported_verification_receipt_version(
            "rava-verification-receipt-v1"
        ));
        assert!(protocol::is_supported_attestation_version(
            protocol::ATTESTATION_VERSION
        ));
        assert!(!protocol::is_supported_attestation_version(
            "rava-attestation-v1"
        ));
    }

    #[test]
    fn constraint_values_allow_integer_narrowing_and_exact_non_integer_matches() {
        assert!(constraints::value_is_no_broader_than(
            &ConstraintValue::Integer(800),
            &ConstraintValue::Integer(1_200)
        ));
        assert!(!constraints::value_is_no_broader_than(
            &ConstraintValue::Integer(1_500),
            &ConstraintValue::Integer(1_200)
        ));
        assert!(constraints::value_is_no_broader_than(
            &ConstraintValue::Text("airline".to_owned()),
            &ConstraintValue::Text("airline".to_owned())
        ));
        assert!(!constraints::value_is_no_broader_than(
            &ConstraintValue::Text("hotel".to_owned()),
            &ConstraintValue::Text("airline".to_owned())
        ));
        assert!(constraints::value_is_no_broader_than(
            &ConstraintValue::Bool(false),
            &ConstraintValue::Bool(false)
        ));
        assert!(!constraints::value_is_no_broader_than(
            &ConstraintValue::Bool(true),
            &ConstraintValue::Bool(false)
        ));
    }

    #[test]
    fn action_constraints_are_covered_by_exact_keys_or_amount_limit() {
        let capability = BTreeMap::from([
            ("max_amount_usd".to_owned(), ConstraintValue::Integer(1_200)),
            (
                "merchant_category".to_owned(),
                ConstraintValue::Text("airline".to_owned()),
            ),
            (
                "external_transfer_allowed".to_owned(),
                ConstraintValue::Bool(false),
            ),
        ]);

        assert!(constraints::action_constraint_is_covered(
            "amount_usd",
            &ConstraintValue::Integer(800),
            &capability
        ));
        assert!(!constraints::action_constraint_is_covered(
            "amount_usd",
            &ConstraintValue::Integer(1_500),
            &capability
        ));
        assert!(constraints::action_constraint_is_covered(
            "merchant_category",
            &ConstraintValue::Text("airline".to_owned()),
            &capability
        ));
        assert!(!constraints::action_constraint_is_covered(
            "merchant_category",
            &ConstraintValue::Text("hotel".to_owned()),
            &capability
        ));
        assert!(constraints::action_constraint_is_covered(
            "external_transfer_allowed",
            &ConstraintValue::Bool(false),
            &capability
        ));
        assert!(!constraints::action_constraint_is_covered(
            "external_transfer_allowed",
            &ConstraintValue::Bool(true),
            &capability
        ));
        assert!(!constraints::action_constraint_is_covered(
            "unknown",
            &ConstraintValue::Bool(true),
            &capability
        ));
    }
}
