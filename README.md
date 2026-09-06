<div align="center">

[<img src="https://raw.githubusercontent.com/apxllo123/akron/refs/heads/main/resources/icon.png?v=5" width="144"/>](https://github.com/apxllo123/akron)

  <h1 align="center">Akron</h1>

  <p align="center">
    <strong>Akron is an experimental universal game analysis and adaptation platform designed to deeply understand a game's complete file set before attempting to adapt it.</strong>
  </p>

  <p align="center">
    Akron is being built as a macOS-first desktop application with an Electron + TypeScript interface and Rust-based analysis and adaptation engines.
  </p>

[![Build Akron](https://img.shields.io/github/actions/workflow/status/apxllo123/akron/build.yml?label=Build)](https://github.com/apxllo123/akron/actions/workflows/build.yml)
[![Latest Release](https://img.shields.io/github/v/release/apxllo123/akron?display_name=tag&label=Release)](https://github.com/apxllo123/akron/releases)
[![License](https://img.shields.io/github/license/apxllo123/akron)](LICENSE)

</div>

## What is Akron?

Akron is being designed as a universal game converter and adaptation system.

The long-term goal is to take a game's existing Windows or macOS application layout and determine everything required to reproduce its expected runtime environment on the target platform, while preserving the game's files, configuration, APIs, online functionality, and behavior as closely as technically possible.

Akron is intentionally built around analysis first. Before adaptation begins, Akron should understand the game's executables, libraries, packages, resources, runtimes, configuration, graphics dependencies, installers, and relationships between files.

The current repository is the foundation for that system. The deep conversion engine is **not finished yet** and is being built incrementally behind verified interfaces.

## Branding

The canonical Akron icon is stored at `resources/icon.png`. It is a rounded-corner PNG with transparent corners and is used as the source artwork for application packaging. The original JPEG artwork remains available at `resources/icon.jpeg`.

The build system converts the canonical PNG into the platform-specific `.icns` and `.ico` assets required by macOS and Windows packaging.

## Architecture

```text
                         Akron Desktop
                    Electron + TypeScript
                              │
                       Local Akron API
                              │
                ┌─────────────┴─────────────┐
                │                           │
         Akron Analyzer              Akron Adapter
              Rust                         Rust
                │                           │
                └─────────────┬─────────────┘
                              │
                         Game files
```

### Akron Analyzer

The Analyzer is the first major stage and the foundation for everything that follows.

Its job is to build a machine-readable understanding of a game directory without modifying the source files. The current implementation can recursively inventory files, record metadata and SHA-256 hashes, identify executable candidates, detect PE binaries, identify PE target architectures, inspect PE imports and sections, and construct binary dependency relationships.

The Analyzer is exposed as both a reusable Rust library and a command-line executable so the same engine can be used by the desktop application, automated tests, future local services, and eventually the Akron API.

The Analyzer is being expanded toward deeper inspection of:

- PE headers and sections
- imports and exports
- executable and DLL relationships
- embedded resources
- runtime and redistributable requirements
- graphics APIs and related dependencies
- installers and package formats
- configuration and content relationships
- archives and nested packages
- platform-specific assumptions
- validation data needed by the adaptation engine

### Akron Adapter

The Adapter is the adaptation and conversion layer that consumes Analyzer results.

The current project architecture keeps adaptation separate from analysis so the Analyzer can establish facts about a game before any conversion decisions are made.

The Adapter currently generates per-game adaptation plans with explicit dependency-resolution results and deliberately reports unimplemented conversion executors as blocked rather than claiming they are ready.

The long-term Adapter is intended to coordinate native target-platform adaptation, packaging, runtime preparation, file transformation, target-platform application generation, and post-conversion validation.

### Akron Desktop

Akron's user interface is built with Electron and TypeScript.

The desktop layer is intentionally separated from the Rust engines. The Electron renderer runs with Node integration disabled and communicates through a controlled preload bridge, while the main process handles native dialogs and launches the Analyzer and Adapter executables.

This gives Akron a richer UI surface while keeping core analysis and conversion logic platform-oriented and testable in Rust.

## Local-first API

Akron is designed around a local-first API.

The desktop application should remain useful without an account or online service. The same API boundary can later support optional Akron services such as cloud-assisted analysis, compatibility data, update services, diagnostics, or other explicitly enabled online features.

## Current status

Akron is currently in the foundation and verification phase.

Implemented today:

- Recursive game-directory analysis
- File metadata inventory
- SHA-256 file hashing
- Executable detection
- PE format detection
- PE architecture detection
- PE headers, sections, and import analysis
- Binary dependency graph construction
- Protection-signal detection for packers/protectors and anti-cheat indicators
- Per-game capability profiling
- Per-game adaptation plan generation
- Explicit dependency-resolution classification
- Rust Analyzer library + CLI
- Rust Adapter library + CLI
- Electron + TypeScript desktop shell
- Native game-folder selection
- Analyzer execution from the desktop application
- Adapter plan execution from the desktop application
- macOS ARM64 packaging
- Windows application packaging
- Rust formatting, compilation, Clippy, tests, and release-build CI
- Electron typechecking and build pipeline
- Automated release-versioning workflow

Not finished yet:

- Full conversion executor implementation
- Native D3D/Win32/runtime adaptation modules
- x86 → ARM64 recompilation backend
- Complete runtime detection and resolution coverage
- Automatic source-to-target file transformation
- Universal game conversion
- Converted-game validation and launch testing for arbitrary titles
- Full online/API service implementation
- Production signing and notarization
- Windows native icon packaging from the source artwork

These are deliberate roadmap items rather than features the current build claims to provide.

## Development

### Requirements

- Rust stable toolchain
- Node.js 24+
- npm

For the current macOS workflow, Apple Silicon is the primary target. Windows remains a secondary validation and build target. Linux support is intentionally deferred and can be restored later without changing the core architecture.

### Rust checks

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo build --workspace --release
```

### Electron checks

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

The command prints the generated game manifest as JSON.

### Run the Adapter

```bash
printf '%s\n' '<GameProfile JSON>' | cargo run -p akron-adapter
```

The Adapter reads a serialized `GameProfile` from standard input and prints the generated adaptation plan as JSON.

### Run the desktop application

```bash
cd desktop
npm install
npm start
```

## Releases

Release builds are driven automatically by `.github/workflows/release.yml` whenever important application changes land on `main`, and the workflow can also be started manually.

Public release tags use the sequence:

```text
v0.1 → v0.2 → v0.3 → … → v0.9 → v1.0 → v1.1 → …
```

The workflow updates the desktop package version and macOS bundle version together, creates the matching Git tag, and publishes a GitHub Release. The tag then triggers the Build Akron workflow, whose macOS artifacts are attached to the release.

## CI and builds

Akron keeps verification and packaging separate.

### Rust CI

`.github/workflows/rust.yml` runs on macOS and Windows and verifies formatting, workspace compilation, Clippy with warnings treated as errors, tests, and release builds.

### Build Akron

`.github/workflows/build.yml` builds the Rust runtime components and desktop application artifacts for the supported platform targets.

The macOS workflow validates the application bundle, `Info.plist`, generated native icon, executable signature, native runtime presence, and executable architecture before uploading the artifact.

## Project layout

```text
Akron/
├── Akron Analyzer/        # Rust analysis engine
│   └── src/
├── Akron Adapter/         # Rust adaptation/conversion core
│   └── src/
├── desktop/               # Electron + TypeScript desktop app
│   ├── src/
│   ├── renderer/
│   ├── resources/         # Packaged local runtime components
│   └── macos/              # Native macOS host and packaging
├── resources/              # Repository-level artwork and assets
│   ├── icon.png
│   └── icon.jpeg
├── tests/                 # Cross-component and future integration tests
├── docs/                  # Architecture and platform documentation
└── .github/workflows/     # CI, builds, and releases
```

## Engineering principles

Akron is being developed around a verification-first workflow.

The project should inspect the repository and dependency contracts before making changes, measure behavior instead of guessing, keep platform decisions explicit, prefer root-cause fixes over warning suppression, and verify every important change with the appropriate formatter, compiler, linter, test suite, or build artifact.

For large changes, the intended loop is:

```text
Inspect
  ↓
Measure / reproduce
  ↓
Implement
  ↓
Format + typecheck + compile
  ↓
Clippy + tests
  ↓
Build artifact
  ↓
Inspect the result
  ↓
Critique
  ↓
Fix
  ↓
Continue
```

## Commit conventions

Akron uses concise conventional commit prefixes:

- `feat:` — new functionality or capabilities
- `fix:` — bug fixes and corrections
- `chore:` — dependency, tooling, maintenance, or housekeeping changes
- `docs:` — documentation-only changes
- `test:` — tests or verification improvements

Keep commits focused so regressions can be traced to a specific change.

## Roadmap

### Phase 1 — Understand

Build a complete, reliable game manifest before attempting conversion.

### Phase 2 — Model

Construct dependency graphs and identify the actual runtime, graphics, packaging, and API requirements of the game.

### Phase 3 — Adapt

Generate a target-platform execution plan and the files/runtime components required to implement it.

### Phase 4 — Convert

Produce the target application package while preserving the source game's contents and behavior wherever technically possible.

### Phase 5 — Validate

Launch, exercise, compare, and diagnose the converted result automatically instead of assuming conversion succeeded.

### Phase 6 — Improve

Feed measured compatibility and failure data back into the analysis and adaptation systems so Akron becomes increasingly automatic and requires less manual maintenance over time.

## Platform scope

Current priority:

```text
macOS / Apple Silicon  → primary
Windows                 → secondary
Linux                   → deferred
```

Linux is intentionally deferred rather than removed from the architecture. The repository can return to Linux support later when its value justifies the additional build and maintenance surface.

## License

Akron is licensed under the [MIT License](LICENSE).
