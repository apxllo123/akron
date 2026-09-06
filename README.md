<div align="center">

[<img src="https://raw.githubusercontent.com/apxllo123/akron/main/resources/icon.png?v=12" width="144" alt="Akron icon"/>](https://github.com/apxllo123/akron)

# Akron

<strong>Universal Game Analysis & Adaptation</strong>

<p>Akron analyzes a game's files, executables, dependencies, runtime requirements, graphics stack, and protection signals before an adaptation plan is generated.</p>

[![Build Akron](https://img.shields.io/github/actions/workflow/status/apxllo123/akron/build.yml?label=Build&style=for-the-badge&color=4c8bf5)](https://github.com/apxllo123/akron/actions/workflows/build.yml)
[![Release](https://img.shields.io/github/v/release/apxllo123/akron?display_name=tag&style=for-the-badge&color=7c5cff)](https://github.com/apxllo123/akron/releases)
[![Security](https://img.shields.io/badge/Security-responsible--disclosure-2ea44f?style=for-the-badge&color=2ea44f)](SECURITY.md)
[![License](https://img.shields.io/github/license/apxllo123/akron?style=for-the-badge&color=2ea043)](LICENSE)

</div>

---

## Overview

Akron is an experimental game analysis and adaptation platform built around a simple rule: **understand the game first, then decide what can be adapted**.

Instead of immediately attempting to transform a game, Akron builds a structured model of the game directory and its Windows binaries. That model can then be consumed by the Adapter to produce an explicit per-game plan, including dependencies that still need to be resolved and capabilities that are not yet implemented.

The repository currently contains the foundation of that pipeline: recursive analysis, PE inspection, dependency modeling, protection detection, capability profiling, adaptation-plan generation, and a desktop shell that connects the pieces together.

## ✨ What Akron Does Today

### 🔎 Analyzer

The Rust Analyzer builds a machine-readable manifest without modifying the source game files. It currently supports:

- Recursive game-directory inventory
- File metadata and SHA-256 hashing
- Executable candidate detection
- PE format and architecture detection
- PE headers and section inspection
- PE import inspection
- Binary dependency relationships
- Protection-signal detection, including packer/protector and anti-cheat indicators
- Per-game capability profiling

### 🧩 Adapter

The Rust Adapter consumes the Analyzer's profile and produces a structured adaptation plan.

Plans can describe:

- Graphics requirements and backend needs
- Windows API families
- Runtime requirements
- Dependency-resolution work
- Unresolved imports
- Protection information
- Capabilities that are currently blocked because their executor is not implemented yet

Akron intentionally reports unfinished conversion work as **blocked** instead of presenting it as supported functionality.

### 🖥️ Desktop

The desktop application provides the user-facing layer for the analysis pipeline.

Current desktop functionality includes:

- Native game-folder selection
- Analyzer execution from the UI
- Adapter-plan execution from the UI
- Controlled Electron preload communication
- Native packaging for macOS ARM64 and Windows

## 🏗️ Architecture

```text
                                Akron Desktop
                           Electron + TypeScript
                                     │
                              Controlled Bridge
                                     │
                           ┌─────────┴─────────┐
                           │                   │
                    Akron Analyzer       Akron Adapter
                         Rust                  Rust
                           │                   │
                           └─────────┬─────────┘
                                     │
                              Game directory
                                     │
                        ┌────────────┴────────────┐
                        │                         │
                     Manifest                GameProfile
                        │                         │
                        └────────────┬────────────┘
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

This separation keeps evidence gathering independent from conversion decisions and gives future conversion executors a stable input model.

## 🎯 Per-Game Capability Profiles

Akron is designed to make game-specific requirements visible before adaptation begins.

A capability profile can capture information such as:

- Direct3D 9 / 10 / 11 / 12 requirements discovered from PE imports
- DXGI requirements
- Windows API families such as windowing/input and networking
- Microsoft Visual C++ runtime requirements
- Binary-to-binary dependencies
- Unresolved imports that still require implementation or resolution
- Protection and anti-cheat signals

The resulting profile becomes the source of truth for adaptation planning rather than relying on assumptions about what a game needs.

## 🛡️ Protection & Safety Signals

The Analyzer can identify evidence associated with packers/protectors and anti-cheat technologies.

These signals are **diagnostic**. Akron does not treat detection as automatic permission to bypass or remove a protection mechanism, and a detected technology can cause an adaptation step to remain blocked until a legitimate implementation exists.

## 🎨 Branding & Assets

The canonical application artwork lives at:

```text
resources/icon.png
```

The original source artwork is retained as:

```text
resources/icon.jpeg
```

The PNG is the packaging source for generated macOS and Windows icon assets. The repository README references the canonical PNG directly so the project branding and packaged application icon use the same source artwork.

## 🚦 Project Status

### Working

- [x] Recursive game analysis
- [x] File inventory and SHA-256 hashing
- [x] Executable detection
- [x] PE architecture detection
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
- [x] Rust CI and Electron CI
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

These items are intentionally separated from the implemented feature set so the README does not overstate the current capabilities of the project.

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

The command outputs the generated game manifest as JSON.

### Run the Adapter

```bash
printf '%s\n' '<GameProfile JSON>' | cargo run -p akron-adapter
```

The Adapter reads a serialized `GameProfile` from standard input and writes the generated adaptation plan as JSON.

### Run the Desktop App

```bash
cd desktop
npm install
npm start
```

## 📦 Releases

Akron releases are versioned using the public sequence:

```text
v0.1 → v0.2 → … → v0.9 → v1.0 → v1.1 → …
```

Application versions remain valid semantic versions internally (`0.1.0`, `1.0.0`, etc.), while the public release tags use the shorter display form.

The release workflow is designed to:

1. Detect the next release number.
2. Update the desktop package and macOS bundle versions together.
3. Create the matching Git tag.
4. Publish the GitHub Release.
5. Build the tagged application.
6. Attach the macOS ARM64 ZIP, DMG, and checksums to the release.

Documentation-only changes are intentionally excluded from expensive application release/build automation.

## ⚙️ CI & Automation

Akron separates verification from packaging while keeping both automated.

### Build Akron

`.github/workflows/build.yml` handles application builds and packaging for the supported targets.

The macOS build validates the application bundle, bundle metadata, native runtime presence, executable architecture, signing state, and generated icon assets before publishing artifacts.

### Rust CI

`.github/workflows/rust.yml` checks formatting, workspace compilation, Clippy with warnings as errors, tests, and release builds.

### Release Automation

`.github/workflows/release.yml` handles release-number progression and coordinates tagged builds and release assets.

The workflows are path-aware so README/docs-only edits do not repeatedly consume build time.

## 📁 Repository Layout

```text
Akron/
├── Akron Analyzer/          # Rust game analysis engine
│   └── src/
├── Akron Adapter/           # Rust adaptation planning engine
│   └── src/
├── desktop/                 # Electron + TypeScript application
│   ├── src/
│   ├── renderer/
│   ├── scripts/
│   └── macos/
├── resources/               # Project artwork and shared assets
│   ├── icon.png
│   └── icon.jpeg
├── tests/                   # Integration and future compatibility tests
├── docs/                    # Architecture and platform documentation
└── .github/workflows/       # CI, build, and release automation
```

## 🧭 Engineering Approach

Akron follows a verification-first development loop:

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

The core principles are:

- **Analysis before adaptation** — establish evidence before making conversion decisions.
- **Explicit capabilities** — represent what is known, unknown, ready, or blocked.
- **Root-cause fixes** — correct the implementation instead of suppressing diagnostics.
- **Reproducible builds** — packaging and versioning should be deterministic and automatable.
- **Honest status reporting** — unfinished conversion functionality stays clearly marked as unfinished.

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
Launch and exercise converted applications automatically and collect failure data.

### Phase 6 — Improve
Use compatibility results to make future analyses and adaptations increasingly automatic.

## 🖥️ Platform Focus

```text
macOS / Apple Silicon   → primary
Windows                 → secondary
Linux                   → deferred
```

Linux is deferred at the product level for now; the core architecture remains structured so additional targets can be introduced later without redesigning the analysis model.

## 📄 License

Akron is licensed under the [MIT License](LICENSE).
