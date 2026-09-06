# 🔐 Akron Security

<div align="center">

**Security, vulnerability reporting, and responsible disclosure**

[Report a Vulnerability](#-reporting-a-vulnerability) · [Security Model](#-security-model) · [Supported Versions](#-supported-versions) · [Research Guidelines](#-responsible-security-research)

</div>

---

Akron analyzes external game files, inspects native binaries, coordinates local processes, packages application artifacts, and is intended to perform increasingly sophisticated adaptation work. Those responsibilities create meaningful security boundaries around **untrusted input, native execution, IPC, filesystem access, dependencies, and build automation**.

Security is therefore treated as part of the engineering design, not as a final checklist.

> **Important:** Akron is still under active development. Features described in the roadmap may not yet be implemented, and security guarantees should not be inferred for functionality that does not currently exist.

## 🛡️ Supported Versions

Akron is currently pre-1.0. Security work is prioritized for the current `main` branch and the latest published release.

| Version | Support |
| --- | --- |
| Latest `main` | ✅ Active development |
| Latest published release | ✅ Security fixes prioritized |
| Older releases | ⚠️ Best effort only |

Because the project is evolving rapidly, users should upgrade to the newest release before investigating an issue that may already have been fixed.

## 🚨 Reporting a Vulnerability

**Do not open a public GitHub issue for a security vulnerability.** Public issues can expose exploit details before a fix is available.

Use GitHub's **private vulnerability reporting** feature for this repository when available. That provides a safer channel for submitting sensitive security information.

### A strong report includes

| Field | What to provide |
| --- | --- |
| **Impact** | What could an attacker make Akron do, access, modify, or disclose? |
| **Location** | Component, file, workflow, release, endpoint, or trust boundary involved. |
| **Reproduction** | Clear steps, a minimal proof of concept, or a small test case when safe. |
| **Affected version** | Release, commit SHA, or build identifier. |
| **Environment** | Operating system, architecture, configuration, and relevant permissions. |
| **Evidence** | Logs, traces, screenshots, or other evidence with secrets removed. |

### 🔑 Never include secrets

Never submit passwords, API keys, access tokens, private keys, signing credentials, recovery codes, cookies, session material, or other authentication secrets.

If a credential is accidentally exposed, **revoke or rotate it first**, then describe the exposure without reproducing the credential itself.

## 🔎 What Happens After a Report

Reports are reviewed as soon as practical. Depending on severity, the response may include a code change, mitigation, dependency update, workflow hardening, configuration change, documentation update, or security release.

During investigation, security details should remain private. Public disclosure should happen only after reasonable remediation and coordination with affected parties.

When appropriate, a security advisory or release note may summarize the issue without publishing unnecessary exploit details. Researchers may be credited with their permission.

## 🧭 Security Model

Akron's security model is centered on keeping **untrusted game content separate from trusted application logic** and keeping each process boundary explicit.

```text
                 Untrusted game content
                          │
                          ▼
                 ┌─────────────────┐
                 │     Analyzer    │
                 │ parse / inspect │
                 └────────┬────────┘
                          │ verified data
                          ▼
                 ┌─────────────────┐
                 │     Adapter     │
                 │ plan / transform│
                 └────────┬────────┘
                          │
                          ▼
                 ┌─────────────────┐
                 │     Desktop     │
                 │ UI / IPC / host │
                 └─────────────────┘
```

The boundaries above are intended to make it possible to reason about trust, validation, and privilege separately rather than allowing arbitrary game data to flow directly into privileged operations.

## 🔒 Security-Sensitive Areas

### Untrusted Game Files

Game directories, executables, DLLs, archives, installers, configuration files, embedded resources, and downloaded content must be treated as **untrusted input**.

Analysis should not execute or modify content merely because it is present in a selected game directory. Where adaptation requires changes, the preferred design is to produce a separate target artifact or working copy rather than silently altering the source.

### Native Execution

The Analyzer and Adapter are native Rust components and may eventually coordinate additional native tooling. Process launches must use explicit executable paths and validated arguments; shell interpretation should not be relied upon for untrusted values.

Any feature that crosses from parsing into execution requires additional scrutiny for command injection, path traversal, privilege escalation, and unexpected child-process behavior.

### Electron and IPC Boundaries

The desktop application contains renderer, preload, main-process, and native-process boundaries. Data crossing those boundaries should be validated and minimized.

The renderer should not receive unnecessary native capabilities, and process-launch APIs should not accept arbitrary shell fragments or unchecked filesystem paths.

### Binary and PE Analysis

PE parsing, import inspection, section handling, dependency discovery, and protection-signal analysis must remain resilient to malformed or adversarial binaries.

Security-relevant failure modes include:

- memory-safety bugs;
- parser crashes and denial of service;
- integer or size overflows;
- path traversal;
- uncontrolled resource consumption;
- accidental execution of analyzed content;
- incorrect trust decisions based on malformed metadata.

### Packaging and Release Automation

Build and release workflows can create downloadable application artifacts, so they are treated as part of the security boundary.

Particular attention should be given to:

- GitHub Actions permissions;
- untrusted pull-request input;
- shell command construction;
- artifact provenance and integrity;
- version and tag automation;
- signing credentials and secrets;
- accidental publication of internal files.

Secrets must never be logged, committed, embedded in artifacts, or copied into generated release notes.

### Dependencies

Third-party Rust and Node.js packages are part of Akron's attack surface. Security advisories affecting code used by the Analyzer, Adapter, desktop application, packaging pipeline, or release automation should be evaluated promptly.

Dependency changes should be verified with the appropriate compiler, test suite, linter, and build pipeline rather than being accepted solely because the package manager resolves them successfully.

## ✅ Secure Development Expectations

Security-sensitive changes should normally include the verification appropriate to their risk:

- formatting, compilation, and static analysis;
- unit or integration tests;
- malformed-input and boundary-condition coverage;
- artifact and packaging validation;
- review of IPC, process, filesystem, and permission boundaries.

**Do not suppress a security check simply to make CI pass.** Fix the underlying problem or document a deliberate, reviewed exception.

## 📦 Release Security

Published artifacts should be traceable to a specific source revision and release tag.

The release pipeline should preserve the relationship between:

```text
source commit
    ↓
version / tag
    ↓
verified build
    ↓
packaged artifact
    ↓
GitHub Release
```

Changes to release automation, artifact upload behavior, permissions, or signing should be considered security-sensitive changes even when the application source itself is unchanged.

## 🧪 Responsible Security Research

Security research against Akron should be performed in a controlled environment and only against systems, repositories, accounts, or services for which you have authorization.

Please avoid unnecessary access to other users' information, disruption of shared services, or publication of working exploit details before coordinated remediation.

Research involving malformed binaries, hostile archives, process execution, or packaging should use isolated test data and disposable environments whenever practical.

## 🎯 Scope

This policy covers security issues introduced by or materially enabled by:

- Akron source code and configuration;
- the Electron desktop application;
- Rust Analyzer and Adapter components;
- GitHub Actions workflows and release automation;
- published Akron application artifacts;
- repository-controlled dependencies and build tooling.

Issues entirely within a third-party game, operating system, DRM system, upstream library, or external service should also be reported to the responsible maintainer where appropriate.

## 🙏 Recognition

Akron appreciates responsible security research and coordinated disclosure. With the researcher's permission, confirmed security fixes may acknowledge contributors in release notes or project documentation.

---

<div align="center">

**Security is part of Akron's architecture.**

*Last reviewed: September 2026*

</div>
