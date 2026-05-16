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
}
