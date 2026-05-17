use serde_json::{Map, Value};

#[derive(Debug, thiserror::Error)]
pub enum CanonicalError {
    #[error("JSON serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub fn canonical_json(value: &Value) -> Result<String, CanonicalError> {
    let sorted = sort_json(value);
    Ok(serde_json::to_string(&sorted)?)
}

fn sort_json(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(sort_json).collect()),
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();

            for key in keys {
                if let Some(item) = map.get(key) {
                    sorted.insert(key.clone(), sort_json(item));
                }
            }

            Value::Object(sorted)
        }
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonical_json_sorts_object_keys() -> Result<(), CanonicalError> {
        let value = json!({ "b": 2, "a": 1 });

        let encoded = canonical_json(&value)?;

        assert_eq!(encoded, r#"{"a":1,"b":2}"#);
        Ok(())
    }

    #[test]
    fn canonical_json_sorts_nested_object_keys() -> Result<(), CanonicalError> {
        let value = json!({
            "outer": { "z": true, "a": false },
            "list": [{ "b": 2, "a": 1 }]
        });

        let encoded = canonical_json(&value)?;

        assert_eq!(
            encoded,
            r#"{"list":[{"a":1,"b":2}],"outer":{"a":false,"z":true}}"#
        );
        Ok(())
    }

    #[test]
    fn canonical_json_is_stable_across_object_insertion_order() -> Result<(), CanonicalError> {
        let equivalent_values = [
            json!({
                "z": 3,
                "a": { "y": true, "b": [3, 2, 1] },
                "m": [{ "k": "v", "c": false }]
            }),
            json!({
                "m": [{ "c": false, "k": "v" }],
                "z": 3,
                "a": { "b": [3, 2, 1], "y": true }
            }),
            json!({
                "a": { "y": true, "b": [3, 2, 1] },
                "m": [{ "k": "v", "c": false }],
                "z": 3
            }),
        ];
        let expected = canonical_json(&equivalent_values[0])?;

        for value in equivalent_values {
            assert_eq!(canonical_json(&value)?, expected);
        }

        Ok(())
    }

    #[test]
    fn canonical_json_is_stable_after_parse_round_trip_for_permuted_objects(
    ) -> Result<(), CanonicalError> {
        let equivalent_values = [
            json!({
                "z": 3,
                "a": { "y": true, "b": [3, 2, 1] },
                "m": [{ "k": "v", "c": false }]
            }),
            json!({
                "m": [{ "c": false, "k": "v" }],
                "z": 3,
                "a": { "b": [3, 2, 1], "y": true }
            }),
            json!({
                "a": { "y": true, "b": [3, 2, 1] },
                "m": [{ "k": "v", "c": false }],
                "z": 3
            }),
        ];

        for value in equivalent_values {
            let encoded = canonical_json(&value)?;
            let reparsed = serde_json::from_str(&encoded)?;

            assert_eq!(canonical_json(&reparsed)?, encoded);
        }

        Ok(())
    }
}
