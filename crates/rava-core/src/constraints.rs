use std::collections::BTreeMap;

use crate::capability::ConstraintValue;

pub const ACTION_AMOUNT_USD: &str = "amount_usd";
pub const CAPABILITY_MAX_AMOUNT_USD: &str = "max_amount_usd";

pub fn value_is_no_broader_than(child: &ConstraintValue, parent: &ConstraintValue) -> bool {
    match (parent, child) {
        (ConstraintValue::Integer(parent_number), ConstraintValue::Integer(child_number)) => {
            child_number <= parent_number
        }
        _ => child == parent,
    }
}

pub fn action_constraint_is_covered(
    key: &str,
    action_value: &ConstraintValue,
    capability_constraints: &BTreeMap<String, ConstraintValue>,
) -> bool {
    action_constraint_violation(key, action_value, capability_constraints).is_none()
}

pub fn action_constraint_violation(
    key: &str,
    action_value: &ConstraintValue,
    capability_constraints: &BTreeMap<String, ConstraintValue>,
) -> Option<String> {
    if key == ACTION_AMOUNT_USD {
        let Some(ConstraintValue::Integer(max_amount)) =
            capability_constraints.get(CAPABILITY_MAX_AMOUNT_USD)
        else {
            return Some(ACTION_AMOUNT_USD.to_owned());
        };
        let ConstraintValue::Integer(action_amount) = action_value else {
            return Some(ACTION_AMOUNT_USD.to_owned());
        };
        if action_amount > max_amount {
            return Some(CAPABILITY_MAX_AMOUNT_USD.to_owned());
        }
        return None;
    }

    let Some(capability_value) = capability_constraints.get(key) else {
        return Some(key.to_owned());
    };
    if value_is_no_broader_than(action_value, capability_value) {
        None
    } else {
        Some(key.to_owned())
    }
}
