<div align="center">

# Akron

<p>
  <strong>Akron is an experimental universal game analysis and adaptation project.</strong>
</p>

<p>
  The project is designed around a deep, non-destructive understanding of a game's complete file set before adaptation work begins.
</p>

</div>

## Project layout

```text
Akron/
├── Akron Analyzer/
│   └── src/
├── Akron Adapter/
│   └── src/
├── tests/
├── docs/
└── .github/workflows/
```

### Akron Analyzer

The Analyzer is the first stage. It recursively inspects a game directory, records file metadata and SHA-256 hashes, identifies executable candidates, detects PE binaries, and records their target architecture. The analyzer is exposed as both a reusable Rust library and a command-line tool.

The original input directory is treated as read-only by the analyzer.

### Akron Adapter

The Adapter is the desktop application and future adaptation engine. The current UI provides a native folder picker and runs analysis on a worker thread so large game directories do not block the interface.

## Development

Install a current stable Rust toolchain.

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo build --workspace --release
```

Run the desktop application:

```bash
cargo run -p akron-adapter
```

Run the command-line analyzer:

```bash
cargo run -p akron-analyzer -- /path/to/game
```

## CI and builds

`rust.yml` checks formatting, compilation, Clippy, tests, and release builds on Linux, macOS, and Windows.

`build.yml` produces release artifacts for Linux and Windows and creates a macOS ARM64 `Akron.app` bundle packaged as a ZIP artifact.

## Commit conventions

Use concise conventional commit prefixes:

- `feat:` — new functionality or capabilities
- `fix:` — bug fixes and corrections
- `chore:` — dependency, tooling, maintenance, or housekeeping changes
- `docs:` — documentation-only changes

Keep commits focused so failures and regressions can be traced cleanly.

## License

Akron is licensed under the MIT License.
