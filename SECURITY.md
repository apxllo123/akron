# Security Policy

<div align="center">

**Akron Security & Responsible Disclosure**

[Report a Vulnerability](#reporting-a-vulnerability) · [Security-Sensitive Areas](#security-sensitive-areas) · [Supported Versions](#supported-versions)

</div>

---

Akron is designed to inspect, analyze, adapt, package, and eventually run software originating from external game files. Because that work crosses file-system, native-process, build, and packaging boundaries, security is treated as a first-class engineering concern.

This document explains which versions are supported, how to report security issues privately, and the areas that receive additional security scrutiny.

## Supported Versions

Akron is under active development and is currently pre-1.0. Security fixes are prioritized for the latest code on `main` and the latest published release.

| Version | Security support |
| --- | --- |
| Latest `main` | ✅ Active |
| Latest published release | ✅ Active where practical |
| Older releases | ❌ Not guaranteed |

Older builds may contain issues that have already been corrected in newer revisions. Users should upgrade to the latest available release when possible.

## Reporting a Vulnerability

**Please do not open a public GitHub issue for a security vulnerability.**

Use GitHub's private vulnerability reporting mechanism for this repository when it is available. Private reporting lets the maintainers investigate the issue without immediately publishing exploit details.

A useful report should include:

- **Impact:** what an attacker could cause or access.
- **Location:** affected component, file, workflow, release, or boundary.
- **Reproduction:** clear steps or a minimal proof of concept, when safe to provide.
- **Affected version:** release, commit, or build identifier.
- **Evidence:** relevant logs, traces, or screenshots with sensitive information removed.
- **Conditions:** any permissions, platform, configuration, or input required to reproduce the issue.

### Never include secrets

Do not place passwords, API keys, access tokens, private keys, signing credentials, recovery codes, cookies, or other authentication material in a vulnerability report.

If sensitive credentials were accidentally exposed, revoke or rotate them immediately and then report the security issue without reproducing the secret in the report itself.

## What Happens After a Report

Reports are reviewed as soon as practical. Depending on severity and scope, a report may result in a code fix, mitigation, configuration change, dependency update, workflow correction, documentation change, or security release.

Security issues should be kept private while they are being investigated and remediated. Coordinated disclosure reduces the chance that users remain exposed while a fix is being prepared.

A public advisory or release note may be published after remediation when appropriate. With the reporter's permission, responsible researchers may be credited.

## Security-Sensitive Areas

The following parts of Akron receive particular security attention:

### Untrusted Game Files

Game directories, executables, libraries, archives, installers, configuration files, and other imported content must be treated as **untrusted input**.

Analysis should avoid executing or modifying untrusted content merely because it exists in a selected game directory. Source-game data should remain intact unless a conversion step explicitly creates a separate target copy or artifact.

### Native Process Boundaries

Akron's Rust Analyzer, Rust Adapter, desktop main process, preload bridge, and renderer operate across different trust boundaries.

IPC and process-launch inputs should be validated, argument handling should avoid shell interpretation, and privileged behavior should remain narrowly scoped.

### Executable and Binary Analysis

PE parsing, import inspection, dependency discovery, protection-signal detection, and future binary transformation features must be resilient to malformed or adversarial input.

Parser bugs, unsafe memory behavior, path traversal, resource exhaustion, unexpected process execution, and confused-deputy behavior are treated as security-relevant concerns.

### Packaging and Release Automation

GitHub Actions workflows can produce distributable application artifacts and therefore require careful handling of:

- repository and workflow permissions;
- untrusted pull-request input;
- shell command construction;
- artifact provenance and integrity;
- signing material and credentials;
- release tags and automated versioning.

Secrets must never be written to logs or embedded in artifacts.

### Dependencies

Rust and Node.js dependencies are part of Akron's security boundary. Dependency updates should be evaluated when advisories affect code used by the Analyzer, Adapter, desktop application, packaging system, or release pipeline.

## Secure Development Expectations

Security-sensitive changes should be accompanied by the verification appropriate to the change, such as:

- formatter, compiler, and linter checks;
- unit and integration tests;
- malformed-input and boundary-condition tests;
- packaging and artifact verification;
- review of permission scopes and process boundaries.

A security check should not be disabled simply to make CI pass. The preferred response is to correct the underlying issue or document a deliberate, reviewed exception.

## Scope

This policy covers:

- Akron's source code and configuration;
- GitHub Actions workflows and release automation;
- published Akron application artifacts;
- vulnerabilities introduced by Akron's own code or build configuration.

Issues that exist entirely within a third-party game, operating system, DRM system, library, service, or other dependency should normally also be reported to the appropriate upstream maintainer.

## Safe Research

Security research against Akron should be performed in a controlled environment and should avoid unnecessary access to data belonging to other users.

Please do not intentionally disrupt shared services, expose private information, publish working exploit details before coordinated remediation, or access systems that are outside the scope of Akron without authorization.

## Recognition

Akron values responsible security research. With the reporter's permission, confirmed security fixes may acknowledge the researcher in project documentation or release notes.

---

*Last reviewed: September 2026*