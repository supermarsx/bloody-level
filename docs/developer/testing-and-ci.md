# Testing and CI

## Local gates

Frontend checks:

```bash
npm run format:check
npm run lint
npm run check
```

Rust checks:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Documentation checks:

```bash
python -m pip install -r docs/requirements.txt
mkdocs build --strict
```

The strict build catches missing navigation files and broken internal documentation references before publication.

## Hosted CI

The GitHub Actions workflow runs formatting, linting, TypeScript checks, Rust checks, tests, documentation validation, and gated Tauri builds. Release coverage is matrixed across Windows x64/ARM64, Linux x64/ARM64, and macOS x64/ARM64 where the hosted runner and signing conditions permit.

Changes to packaging, native dependencies, authentication, storage, or ingestion should be tested at the boundary they affect. A passing frontend check alone does not prove that a desktop command or encrypted backup works.

## Test data

Do not commit real medical reports. Use synthetic or deliberately sanitized fixtures. Tests should verify behavior such as duplicate detection, wrong-password rejection, tamper rejection, parser diagnostics, backup recovery, and command error handling.
