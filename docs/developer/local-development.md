# Local development

## Set up

```bash
npm ci
npm run tauri:dev
```

For frontend-only work:

```bash
npm run dev
```

The desktop command is the meaningful integration environment because the application state, file dialogs, encryption, and native commands live in Tauri.

## Useful checks

```bash
npm run format:check
npm run lint
npm run check
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
mkdocs build --strict
```

Run Rust commands from the repository root with `--manifest-path`, or change into `src-tauri` before using Cargo commands. Avoid broad formatting changes when a focused file-level check is sufficient.

## Native dependencies

The build helper looks for a matching PDFium archive under `src-tauri/binaries` and can attempt a download. Keep target and architecture aligned. Optional OCR and model paths should be configured locally and must not be committed with health data or large binaries.

## Adding a feature

Keep the user-facing route, command, persistence, recovery behavior, tests, and documentation in the same change. For sensitive data flows, document what is stored, what crosses IPC, and how a user can verify or undo the operation.
