# Security Policy

Rava is a security-sensitive agent authorization protocol. Treat findings about canonicalization, signatures, delegation attenuation, replay, revocation, receipts, attestations, wrappers, and verifier service behavior as security issues.

Rava V0 is a draft reference implementation, not production-ready security software. No external security review has been completed.

## Reporting

Report suspected vulnerabilities through a private GitHub security advisory for `https://github.com/syndicalt/rava` when available. If private advisories are not available to you, open a minimal public issue that asks for a private contact path without disclosing exploit details.

Do not include private keys, credentials, access tokens, or raw sensitive action payloads in reports, issues, pull requests, logs, or screenshots. Use minimized synthetic examples whenever possible.

Useful reports include:

- affected commit, tag, or package version;
- affected component, such as `rava-core`, `rava`, `rava-wasm`, `rava-wasm-js`, docs, or examples;
- whether the issue can cause malformed, unsigned, unverifiable, expired, revoked, replayed, or over-scoped input to be accepted;
- minimal reproduction steps or a minimized fixture;
- expected fail-closed behavior;
- actual behavior.

## Review Scope

The V0 external review scope is documented in [docs/security/review-guide-v0.md](docs/security/review-guide-v0.md).

Findings and remediation status are tracked in [docs/security/review-register-v0.md](docs/security/review-register-v0.md).

External review kickoff should use [docs/security/external-review-kickoff-checklist-v0.md](docs/security/external-review-kickoff-checklist-v0.md) to freeze the review target, hand off the packet, track findings, and record optional longer fuzz evidence.

External reviewers can open structured remediation issues with [.github/ISSUE_TEMPLATE/security-review-finding.yml](.github/ISSUE_TEMPLATE/security-review-finding.yml). That issue form is a tracking aid, not evidence that Rava has been externally reviewed.

Security-sensitive repository paths are mapped in [.github/CODEOWNERS](.github/CODEOWNERS) so protocol, release, security, workflow, wrapper, fixture, and example changes have explicit review ownership.

Pull requests should use [.github/pull_request_template.md](.github/pull_request_template.md) to record security-boundary impact, required verification commands, and review artifacts.

Dependency update surfaces are tracked in [.github/dependabot.yml](.github/dependabot.yml) for Rust crates, the TypeScript wrapper package, and GitHub Actions.

Production deployment issues involving key custody, public-key discovery, distributed replay, distributed revocation, caller identity, distributed rate limiting, audit storage, or monitoring should also reference [docs/operations/production-trust-v0.md](docs/operations/production-trust-v0.md) and the relevant detailed runbook:

- [docs/operations/key-custody-v0.md](docs/operations/key-custody-v0.md);
- [docs/operations/key-discovery-v0.md](docs/operations/key-discovery-v0.md);
- [docs/operations/distributed-replay-v0.md](docs/operations/distributed-replay-v0.md);
- [docs/operations/distributed-revocation-v0.md](docs/operations/distributed-revocation-v0.md);
- [docs/operations/audit-storage-v0.md](docs/operations/audit-storage-v0.md);
- [docs/operations/caller-identity-v0.md](docs/operations/caller-identity-v0.md);
- [docs/operations/distributed-rate-limits-v0.md](docs/operations/distributed-rate-limits-v0.md);
- [docs/operations/monitoring-v0.md](docs/operations/monitoring-v0.md).

## Supported Status

The current repository is suitable for protocol development, examples, interop work, and review. It is not supported as a production authorization system.
