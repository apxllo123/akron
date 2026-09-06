<div align="center">

# 🔐 Akron Security

**Security policy · Responsible disclosure · Secure development**

[![Security Policy](https://img.shields.io/badge/security-responsible%20disclosure-2ea44f?style=flat-square)](SECURITY.md)
[![License](https://img.shields.io/github/license/apxllo123/akron?style=flat-square)](LICENSE)

Akron treats security as part of the product, not an afterthought. This policy explains how security issues should be reported, which releases are supported, and which parts of the project receive additional scrutiny.

</div>

---

## 🚨 Reporting a Vulnerability

> **Please do not open a public GitHub issue for a suspected security vulnerability.**

Use GitHub's **private vulnerability reporting** for this repository when available. Private reporting gives the maintainers a place to investigate an issue without immediately exposing technical details to the public.

A strong report should include:

| Information | What to provide |
| --- | --- |
| **Impact** | What an attacker could make the application do, access, or expose |
| **Affected area** | Component, source file, workflow, release, package, or trust boundary |
| **Reproduction** | Clear reproduction steps or a minimal proof of concept when appropriate |
| **Affected version** | Release, commit, or build identifier |
| **Environment** | Operating system, architecture, configuration, and relevant dependencies |
| **Evidence** | Logs, traces, screenshots, or other useful evidence with secrets removed |

### Never include secrets

Do **not** include passwords, API keys, access tokens, private keys, signing credentials, cookies, recovery codes, or other authentication material in a report.

If credentials were accidentally exposed, revoke or rotate them immediately and report the incident without reproducing the secret in the report.

---

## 🛡️ Supported Versions

Akron is under active development and is currently pre-1.0. Security fixes are prioritized for the latest code on `main` and the newest published release.

| Version | Security support |
| --- | --- |
| **Latest `main`** | ✅ Active |
| **Latest release** | ✅ Active where practical |
| Older releases | ⚠️ Not guaranteed |

Because Akron is evolving quickly, users should upgrade to the latest available build before investigating or reporting an issue that may already be fixed.

---

## 🔎 Security-Sensitive Areas

Akron works with game directories, executables, libraries, archives, native processes, and automated build infrastructure. These areas are considered especially security-sensitive.

### Untrusted game content

Selected game directories must be treated as **untrusted input**. Executables, DLLs, installers, archives, configuration files, and embedded resources must not be assumed to be safe merely because they came from a game installation.

Analysis should avoid executing untrusted content just because it was discovered. Source data should remain unchanged unless an explicit adaptation step creates a separate target artifact.

### Native process boundaries

The desktop application, Electron renderer, preload bridge, Rust Analyzer, Rust Adapter, and future native conversion components cross different trust boundaries.

Inputs crossing those boundaries should be validated, process arguments should avoid unnecessary shell interpretation, and privileged operations should remain narrowly scoped.

### Binary analysis

PE parsing, import inspection, dependency discovery, protection-signal detection, and future binary transformation features must tolerate malformed or adversarial files.

Potential security issues include:

- path traversal;
- arbitrary process execution;
- unsafe memory behavior;
- parser crashes or denial of service;
- resource-exhaustion conditions;
- confused-deputy behavior;
- unintended modification of source game data.

### Packaging and release automation

GitHub Actions can create distributable application artifacts, so workflow changes are security-sensitive. Particular care should be taken with:

- workflow and repository permissions;
- untrusted pull-request input;
- shell command construction;
- artifact provenance and integrity;
- signing credentials and secrets;
- automated versioning and release tags.

Secrets must never be written to logs or embedded into published artifacts.

### Dependencies

Rust and Node.js dependencies are part of Akron's attack surface. Security advisories affecting dependencies used by the Analyzer, Adapter, desktop application, packaging system, or release pipeline should be evaluated promptly.

---

## 🔧 Secure Development

Akron follows a verification-first engineering process. Security-sensitive changes should receive the checks appropriate to their risk, which may include:

- formatter, compiler, and linter checks;
- unit and integration tests;
- malformed-input and boundary-condition tests;
- packaging and artifact verification;
- dependency review;
- process-boundary and permission review.

> **Do not disable a security check simply to make CI pass.** Fix the underlying issue or document a deliberate, reviewed exception.

For changes involving GitHub Actions, native process execution, file-system access, binary parsing, or release packaging, review both the normal code path and failure paths.

---

## 📦 Scope

This policy covers:

- Akron source code and configuration;
- GitHub Actions workflows and release automation;
- published Akron application artifacts;
- vulnerabilities introduced by Akron's own code or build configuration.

Issues entirely contained within a third-party game, operating system, DRM system, library, service, or other external component should normally also be reported to the appropriate upstream maintainer.

---

## 🧪 Safe Security Research

Security research should be performed in a controlled environment and should avoid unnecessary access to other users' data or systems.

Please do not intentionally disrupt shared services, expose private information, or publish working exploit details before coordinated remediation. Do not access systems outside the scope of Akron without authorization.

---

## 📬 What Happens After a Report?

Reports are reviewed as soon as practical. Depending on severity and scope, a valid report may result in a code fix, mitigation, dependency update, workflow correction, documentation change, or security release.

During investigation and remediation, security issues should remain private. After a fix is available, Akron may publish an advisory or release note when appropriate.

With the reporter's permission, responsible researchers may be credited for confirmed security fixes.

---

## ✅ Security Checklist

Before shipping a security-sensitive change, Akron aims to verify:

**Input** → untrusted data is validated  
**Execution** → unexpected code is not executed  
**Paths** → file operations stay within intended boundaries  
**Processes** → IPC and child-process arguments are controlled  
**Secrets** → credentials never enter logs or artifacts  
**Dependencies** → known security advisories are reviewed  
**Artifacts** → releases contain only intended files  
**CI** → permissions are no broader than necessary

---

<div align="center">

**Responsible disclosure helps keep Akron safe for everyone.**

*Last reviewed: September 2026*

</div>
