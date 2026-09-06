<div align="center">

# 🔐 Akron Security

**Security Policy & Responsible Disclosure**

Security guidance for Akron source code, desktop components, binary analysis, build automation, and published application artifacts.

[![Security](https://img.shields.io/badge/Security-responsible--disclosure-2ea44f?style=for-the-badge&color=2ea44f)](SECURITY.md)
[![License](https://img.shields.io/github/license/apxllo123/akron?style=for-the-badge&color=2ea043)](LICENSE)

</div>

---

## 🚨 Reporting a Vulnerability

> **Please do not open a public GitHub issue for a suspected security vulnerability.**

Use GitHub's private vulnerability reporting for this repository when available. Include enough information for the issue to be reproduced and assessed safely.

| Information | What to provide |
| --- | --- |
| **Impact** | What an attacker could cause, access, or expose |
| **Affected area** | Component, file, workflow, release, package, or trust boundary |
| **Reproduction** | Clear steps or a minimal proof of concept when appropriate |
| **Affected version** | Release, commit, or build identifier |
| **Environment** | OS, architecture, configuration, and relevant dependencies |
| **Evidence** | Logs, traces, or screenshots with sensitive information removed |

### 🔐 Never include secrets

Never include passwords, API keys, access tokens, private keys, signing credentials, cookies, recovery codes, or other authentication material in a report.

If credentials are exposed, revoke or rotate them immediately and report the incident without reproducing the secret.

---

## 🛡️ Supported Versions

Akron is under active development and security fixes are prioritized for the latest code on `main` and the newest published release.

| Version | Security support |
| --- | --- |
| **Latest `main`** | ✅ Active |
| **Latest release** | ✅ Active where practical |
| Older releases | ⚠️ Not guaranteed |

---

## 🔎 Security-Sensitive Areas

### Untrusted Game Content

Game directories, executables, DLLs, installers, archives, configuration files, and embedded resources must be treated as **untrusted input**.

Analysis should not execute untrusted content merely because it was discovered. Source game data should remain unchanged unless an explicit adaptation step creates a separate target artifact.

### Native Process Boundaries

The Electron renderer, preload bridge, desktop main process, Rust Analyzer, Rust Adapter, and future native conversion components cross different trust boundaries.

Process arguments and IPC inputs should be validated, shell interpretation should be avoided where unnecessary, and privileged operations should remain narrowly scoped.

### Binary Analysis

PE parsing, import inspection, dependency discovery, protection-signal detection, and future binary transformation features must tolerate malformed or adversarial input.

Security-relevant failure modes include path traversal, arbitrary process execution, unsafe memory behavior, parser crashes, resource exhaustion, and unintended modification of source data.

### 📦 Packaging & Release Automation

GitHub Actions can create distributable application artifacts, making workflow and packaging changes security-sensitive.

Review:

- Workflow and repository permissions
- Untrusted pull-request input
- Shell command construction
- Artifact provenance and integrity
- Signing credentials and secrets
- Automated versioning and release tags

Secrets must never be written to logs or embedded into published artifacts.

### 📚 Dependencies

Rust and Node.js dependencies are part of Akron's attack surface. Security advisories affecting dependencies used by the Analyzer, Adapter, desktop application, packaging system, or release pipeline should be evaluated promptly.

---

## 🧪 Secure Development

Security-sensitive changes should receive verification appropriate to their risk:

- Formatter, compiler, and linter checks
- Unit and integration tests
- Malformed-input and boundary-condition tests
- Packaging and artifact verification
- Dependency review
- Process-boundary and permission review

> **Do not disable a security check simply to make CI pass.** Fix the underlying issue or document a deliberate, reviewed exception.

---

## ✅ Pre-Release Security Checklist

```text
Input        → untrusted data is validated
Execution    → unexpected code is not executed
Paths        → file operations stay within intended boundaries
Processes    → IPC and child-process arguments are controlled
Secrets      → credentials never enter logs or artifacts
Dependencies → known advisories are reviewed
Artifacts    → releases contain only intended files
CI           → permissions are no broader than necessary
```

---

## 📬 What Happens After a Report?

Reports are reviewed as soon as practical. Depending on severity and scope, a valid report may result in a code fix, mitigation, dependency update, workflow correction, documentation change, or security release.

During investigation and remediation, security issues should remain private. After a fix is available, Akron may publish an advisory or release note when appropriate.

With the reporter's permission, responsible researchers may be credited for confirmed security fixes.

## 📦 Scope

This policy covers Akron source code and configuration, GitHub Actions workflows and release automation, published Akron artifacts, and vulnerabilities introduced by Akron's own code or build configuration.

Issues entirely contained within a third-party game, operating system, DRM system, library, service, or other external component should normally also be reported to the appropriate upstream maintainer.

## 🧭 Safe Security Research

Security research should be performed in a controlled environment and should avoid unnecessary access to other users' data or systems.

Do not intentionally disrupt shared services, expose private information, or publish working exploit details before coordinated remediation. Do not access systems outside the scope of Akron without authorization.

---

<div align="center">

**Responsible disclosure helps keep Akron secure.**

*Last reviewed: September 2026*

</div>
