<div align="center">

[<img src="https://raw.githubusercontent.com/apxllo123/akron/main/resources/icon.png?v=13" width="144" alt="Akron icon"/>](https://github.com/apxllo123/akron)

  <h1 align="center">Akron</h1>

  <p align="center">
    <strong>Universal Game Analysis & Adaptation</strong>
  </p>

  <p align="center">
    Akron analyzes a game's files, executables, dependencies, runtime requirements, graphics stack, and protection signals before an adaptation plan is generated.
  </p>

[![Build Akron](https://img.shields.io/github/actions/workflow/status/apxllo123/akron/build.yml?label=Build&style=for-the-badge&color=4c8bf5)](https://github.com/apxllo123/akron/actions/workflows/build.yml)
[![Release](https://img.shields.io/github/v/release/apxllo123/akron?display_name=tag&style=for-the-badge&color=7c5cff)](https://github.com/apxllo123/akron/releases)
[![Security](https://img.shields.io/badge/Security-responsible--disclosure-2ea44f?style=for-the-badge&color=2ea44f)](SECURITY.md)
[![License](https://img.shields.io/github/license/apxllo123/akron?style=for-the-badge&color=2ea043)](LICENSE)

</div>

---

## Overview

Akron is an experimental game analysis and adaptation platform built around a simple rule: **understand the game first, then decide what can be adapted**.

The current repository contains the foundation of that pipeline: recursive analysis, PE inspection, dependency modeling, protection detection, capability profiling, adaptation-plan generation, and a desktop shell that connects the pieces together.

## ✨ What Akron Does Today

### 🔎 Analyzer

The Rust Analyzer builds a machine-readable manifest without modifying the source game files. It currently supports:

- Recursive game-directory inventory
- File metadata and SHA-256 hashing
- Executable candidate detection
- PE format and architecture detection
- PE headers, sections, and import inspection
- Binary dependency relationships
- Protection-signal detection
- Per-game capability profiling

### 🧩 Adapter

The Rust Adapter consumes the Analyzer's profile and produces a structured adaptation plan covering:

- Graphics requirements
- Windows API families
- Runtime requirements
- Dependency-resolution work
- Unresolved imports
- Protection information
- Ready / blocked adaptation steps

Akron intentionally reports unfinished conversion work as **blocked** instead of claiming support that does not exist yet.

### 🖥️ Desktop

The desktop application provides the user-facing analysis pipeline with:

- Native game-folder selection
- Analyzer execution
- Adapter-plan execution
- Controlled Electron preload communication
- macOS ARM64 and Windows packaging

## 🏗️ Architecture

```text
                         Akron Desktop
                    Electron + TypeScript
                              │
                       Controlled Bridge
                              │
                ┌─────────────┴─────────────┐
                │                           │
         Akron Analyzer              Akron Adapter
              Rust                         Rust
                │                           │
                └─────────────┬─────────────┘
                              │
                         Game files
                              │
                         GameProfile
                              │
                       Adaptation Plan
```

### Analyzer → Profile → Plan

```text
Game files
   ↓
Manifest
   ↓
PE / dependency / protection analysis
   ↓
GameProfile
   ↓
Adaptation planning
   ↓
Explicit Ready / Blocked steps
```

## 🎯 Per-Game Capability Profiles

Akron makes game-specific requirements visible before adaptation begins.

Profiles can capture Direct3D requirements, DXGI dependencies, Windows API families, Visual C++ runtime requirements, binary dependencies, unresolved imports, and protection signals discovered during analysis.

## 🛡️ Protection & Safety Signals

Protection and anti-cheat detections are diagnostic signals. Akron does not treat detection as permission to bypass or remove a protection mechanism. A detected technology may instead cause an adaptation step to remain blocked until a legitimate implementation exists.

## 🎨 Branding & Assets

The canonical application artwork is:

```text
resources/icon.png
```

The original source artwork remains available at:

```text
resources/icon.jpeg
```

The canonical PNG is used as the source artwork for packaged application icons.

## 🚦 Project Status

### Working

- [x] Recursive game analysis
- [x] File inventory and SHA-256 hashing
- [x] Executable and PE detection
- [x] PE headers, sections, and imports
- [x] Binary dependency graph generation
- [x] Protection-signal detection
- [x] Per-game capability profiles
- [x] Per-game adaptation plans
- [x] Explicit dependency-resolution states
- [x] Rust Analyzer library and CLI
- [x] Rust Adapter library and CLI
- [x] Electron desktop shell
- [x] Native game-folder selection
- [x] Analyzer/Adapter execution from the desktop app
- [x] macOS ARM64 packaging pipeline
- [x] Windows packaging pipeline
- [x] Rust and Electron CI
- [x] Automated release workflow

### In Progress / Planned

- [ ] Full adaptation executors
- [ ] Native D3D/graphics adaptation modules
- [ ] Win32 API implementation coverage
- [ ] Runtime resolution and installation coverage
- [ ] Source-to-target file transformation
- [ ] x86 → ARM64 recompilation backend
- [ ] Automatic converted-game validation and launch testing
- [ ] Broader graphics backend support
- [ ] Production signing and notarization
- [ ] Full online/API service layer

## 🧪 Development & Verification

### Requirements

- Rust stable toolchain
- Node.js 24+
- npm

### Rust

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo build --workspace --release
```

### Desktop

```bash
cd desktop
npm install
npm run typecheck
npm run compile
npm run build
```

### Run the Analyzer

```bash
cargo run -p akron-analyzer -- /path/to/game
```

### Run the Adapter

```bash
printf '%s\n' '<GameProfile JSON>' | cargo run -p akron-adapter
```

### Run the Desktop App

```bash
cd desktop
npm install
npm start
```

## 📦 Releases

Akron uses the public release sequence:

```text
v0.1 → v0.2 → … → v0.9 → v1.0 → v1.1 → …
```

Application versions remain valid semantic versions internally, while public release tags use the shorter display form.

Release automation updates application versions together, creates the matching tag, publishes the GitHub Release, builds the tagged application, and attaches the macOS ARM64 ZIP, DMG, and checksums.

Documentation-only changes are intentionally excluded from expensive application build and release automation.

## ⚙️ CI & Automation

### Build Akron

`.github/workflows/build.yml` handles application builds and packaging.

### Rust CI

`.github/workflows/rust.yml` checks formatting, compilation, Clippy, tests, and release builds.

### Release Automation

`.github/workflows/release.yml` handles release-number progression and coordinates tagged builds and release assets.

Workflows are path-aware so documentation-only changes do not repeatedly consume build time.

## 📁 Repository Layout

```text
Akron/
├── Akron Analyzer/          # Rust game analysis engine
├── Akron Adapter/           # Rust adaptation planning engine
├── desktop/                 # Electron + TypeScript application
├── resources/               # Project artwork and shared assets
│   ├── icon.png
│   └── icon.jpeg
├── tests/                   # Integration and future compatibility tests
├── docs/                    # Architecture and platform documentation
└── .github/workflows/       # CI, build, and release automation
```

## 🧭 Engineering Approach

Akron follows a verification-first loop:

```text
Inspect
  ↓
Reproduce / Measure
  ↓
Implement
  ↓
Format + Typecheck + Compile
  ↓
Clippy + Tests
  ↓
Build
  ↓
Inspect the artifact
  ↓
Fix and refine
```

Core principles:

- **Analysis before adaptation**
- **Explicit capabilities**
- **Root-cause fixes**
- **Reproducible builds**
- **Honest status reporting**

## 🗺️ Roadmap

### Phase 1 — Understand
Create complete and trustworthy game manifests.

### Phase 2 — Model
Turn evidence into dependency graphs and capability profiles.

### Phase 3 — Adapt
Resolve target-platform requirements and generate concrete adaptation plans.

### Phase 4 — Convert
Execute those plans through verified native conversion backends.

### Phase 5 — Validate
Launch and exercise converted applications automatically and collect diagnostics.

### Phase 6 — Improve
Use compatibility results to make future analyses and adaptations increasingly automatic.

## 🖥️ Platform Focus

```text
macOS / Apple Silicon   → primary
Windows                 → secondary
Linux                   → deferred
```

## 📄 License

Akron is licensed under the [MIT License](LICENSE).
