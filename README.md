<div align="center">

# Akron

<p align="center">
  <strong>Akron is a universal game analysis and adaptation project designed to deeply inspect software, understand its requirements, and build platform-specific application artifacts automatically.</strong>
</p>

</div>

## Project structure

```text
Akron/
├── Akron Analyzer/
│   └── src/
│       ├── scanner/
│       ├── pe/
│       ├── dependencies/
│       ├── runtimes/
│       ├── graphics/
│       ├── archives/
│       ├── config/
│       └── manifest/
│
├── Akron Adapter/
│   └── src/
│       ├── converter/
│       ├── cpu/
│       ├── winapi/
│       ├── graphics/
│       ├── runtimes/
│       ├── filesystem/
│       ├── packaging/
│       └── validation/
│
├── docs/
├── tests/
├── tools/
└── Cargo.toml
```

## Design goals

- Non-destructive recursive game analysis
- Deep executable, library, runtime, asset, and package inspection
- Machine-readable game manifests
- Automatic requirement discovery
- Modular adaptation and conversion pipeline
- Validation and diagnostics after every transformation
- Apple Silicon and macOS as a first-class development target
- Minimal per-game manual configuration

## Development

Requirements:

- Rust stable toolchain
- macOS for the macOS-specific validation path

Build the workspace:

```bash
cargo check --workspace
cargo test --workspace
```

## Commit conventions

Use concise conventional commit prefixes:

- `feat:` — new functionality or capabilities
- `fix:` — bug fixes and corrections
- `chore:` — dependency, tooling, maintenance, or housekeeping changes
- `docs:` — documentation-only changes

Avoid using `test:` as the primary commit prefix for workflow or verification changes; use `feat:` or `fix:` when the change affects functionality or CI behavior, and `chore:` for maintenance/tooling work.

## License

Akron is licensed under the MIT License.
