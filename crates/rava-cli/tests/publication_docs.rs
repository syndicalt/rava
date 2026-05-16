use std::error::Error;
use std::path::{Path, PathBuf};

#[test]
fn readme_states_publication_posture_and_operator_path() -> Result<(), Box<dyn Error>> {
    let readme = std::fs::read_to_string(repository_root().join("README.md"))?;

    for required in [
        "Rava V0 is a draft reference implementation, not production-ready security software.",
        "## Table of Contents",
        "## Requirements",
        "## Quickstart",
        "## Repository Layout",
        "## Verification Gates",
        "docs/roadmap.md",
        "cargo fmt --check",
        "cargo clippy --workspace --all-targets -- -D warnings",
        "cargo test --workspace",
        "cargo run -p rava -- demo flight-booking",
        "From this repository, prefix CLI commands with `cargo run -p rava --`.",
        "The preview service is not a production authorization service.",
    ] {
        assert!(readme.contains(required), "README missing: {required}");
    }

    Ok(())
}

#[test]
fn functional_roadmap_separates_current_state_from_future_work() -> Result<(), Box<dyn Error>> {
    let roadmap = std::fs::read_to_string(repository_root().join("docs/roadmap.md"))?;

    for required in [
        "# Rava Functional Roadmap",
        "## Current Baseline",
        "## Release Readiness",
        "## V0 Hardening",
        "## V1 Developer Preview",
        "## Interop",
        "## Production Trust and Operations",
        "## Non-Goals",
        "not implemented guarantees today",
    ] {
        assert!(roadmap.contains(required), "roadmap missing: {required}");
    }

    Ok(())
}

#[test]
fn workspace_publish_metadata_uses_real_repository_url() -> Result<(), Box<dyn Error>> {
    let manifest = std::fs::read_to_string(repository_root().join("Cargo.toml"))?;

    assert!(manifest.contains(r#"repository = "https://github.com/syndicalt/rava""#));
    assert!(!manifest.contains("https://example.invalid/rava"));
    Ok(())
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
