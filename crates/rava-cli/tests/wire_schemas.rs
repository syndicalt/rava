use std::error::Error;
use std::path::{Path, PathBuf};

#[test]
fn v0_wire_schemas_are_documented_for_core_protocol_objects() -> Result<(), Box<dyn Error>> {
    let root = repository_root().join("docs/schemas/v0");
    let schemas = [
        (
            "action.schema.json",
            Some("rava-action-v0"),
            "ActionEnvelope",
        ),
        (
            "capability.schema.json",
            Some("rava-capability-v0"),
            "Capability",
        ),
        (
            "verification-receipt.schema.json",
            Some("rava-verification-receipt-v0"),
            "VerificationReceipt",
        ),
        (
            "attestation.schema.json",
            Some("rava-attestation-v0"),
            "Attestation",
        ),
        ("replay-store.schema.json", None, "ReplayStore"),
        ("revocation-store.schema.json", None, "RevocationStore"),
    ];

    for (file_name, expected_version, expected_title) in schemas {
        let schema_path = root.join(file_name);
        assert!(
            schema_path.exists(),
            "missing schema {}",
            schema_path.display()
        );
        let schema: serde_json::Value = serde_json::from_slice(&std::fs::read(&schema_path)?)?;
        assert_eq!(
            string_field(&schema, "$schema")?,
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(string_field(&schema, "title")?, expected_title);
        assert_eq!(
            schema.get("type").and_then(serde_json::Value::as_str),
            Some("object")
        );
        assert_eq!(
            schema
                .get("additionalProperties")
                .and_then(serde_json::Value::as_bool),
            Some(false),
            "{file_name} must document fail-closed object shape"
        );
        if let Some(expected_version) = expected_version {
            assert!(
                schema_mentions_version(&schema, expected_version),
                "{file_name} does not constrain expected version {expected_version}"
            );
        }
        if file_name == "revocation-store.schema.json" {
            assert!(
                schema
                    .pointer("/properties/fresh_until_unix/type")
                    .and_then(serde_json::Value::as_str)
                    == Some("integer"),
                "revocation schema must document optional freshness deadline"
            );
        }
    }

    Ok(())
}

#[test]
fn v0_wire_schema_readme_explains_schemas_are_not_verification() -> Result<(), Box<dyn Error>> {
    let readme = std::fs::read_to_string(repository_root().join("docs/schemas/v0/README.md"))?;

    assert!(readme.contains("Schemas describe wire shape only."));
    assert!(readme.contains("They do not verify signatures"));
    assert!(readme.contains("Use the Rust verifier"));
    Ok(())
}

fn string_field<'a>(value: &'a serde_json::Value, key: &str) -> Result<&'a str, Box<dyn Error>> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("missing string field {key:?}").into())
}

fn schema_mentions_version(schema: &serde_json::Value, expected_version: &str) -> bool {
    match schema {
        serde_json::Value::String(value) => value == expected_version,
        serde_json::Value::Array(values) => values
            .iter()
            .any(|value| schema_mentions_version(value, expected_version)),
        serde_json::Value::Object(values) => values
            .values()
            .any(|value| schema_mentions_version(value, expected_version)),
        _ => false,
    }
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
