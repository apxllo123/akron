# Security Policy

## Supported Versions

Akron is currently under active development. Security fixes are prioritized for the latest version on the `main` branch.

| Version | Supported |
| --- | --- |
| Latest `main` | ✅ |
| Older releases | ❌ |

Because Akron is still pre-1.0, compatibility and security support for older releases may change as the project evolves.

## Reporting a Security Vulnerability

Please **do not** report security vulnerabilities through a public GitHub issue.

For a vulnerability that could affect users, builds, packaged applications, credentials, or the repository itself, use GitHub's private security reporting mechanism for this repository when available. This allows the issue to be reviewed without publicly disclosing exploit details.

When reporting a vulnerability, include enough information to reproduce and assess the issue safely:

- A clear description of the security impact.
- The affected component, file, workflow, or release.
- Reproduction steps or a minimal proof of concept when appropriate.
- The affected version or commit.
- Any relevant logs, error messages, or screenshots that do not contain secrets.

Please **never include passwords, API keys, tokens, private keys, personal access information, or other secrets** in a report.

## What to Expect

Reports will be reviewed as soon as practical. A valid report may result in a fix, mitigation, documentation update, or release containing the correction.

Please avoid publicly disclosing the vulnerability until there has been reasonable time to investigate and address it. Coordinated disclosure helps protect users who may be running affected builds.

## Security-Sensitive Areas

Akron includes several areas that deserve particular security attention:

- **Game and executable analysis:** files should be treated as untrusted input and analyzed without modifying the source game data.
- **Desktop application boundaries:** Electron renderer, preload, main-process, and native-process communication should remain explicitly separated and validated.
- **Native execution:** Analyzer and Adapter processes should not execute untrusted code merely because it is present in a game directory.
- **Packaging and releases:** release workflows should avoid exposing signing material, tokens, credentials, or other secrets.
- **Dependency management:** Rust and Node dependencies should be kept current where practical and reviewed when security advisories affect them.
- **Build automation:** GitHub Actions changes should be reviewed for permission scope, artifact integrity, command injection, and untrusted repository input.

## Secure Development Practices

Akron follows a verification-first development process. Security-sensitive changes should be accompanied by appropriate tests, static analysis, build verification, and review of the affected trust boundaries.

Do not disable a security check simply to make CI pass. Prefer fixing the underlying problem or documenting a deliberate, reviewed exception.

## Scope

This policy covers the Akron source repository, its GitHub Actions workflows, published Akron application artifacts, and security issues introduced by Akron's own code or build configuration.

Third-party products, games, operating systems, DRM systems, or dependencies may have their own security policies and reporting channels. Vulnerabilities that are entirely within a third-party component should normally be reported to that component's maintainer as well.

## Credits

Akron appreciates responsible security research and coordinated disclosure. With the reporter's permission, security fixes may acknowledge contributors in release notes or project documentation.
