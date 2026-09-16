# bloody-level

[![CI](https://github.com/supermarsx/bloody-level/actions/workflows/ci.yml/badge.svg)](https://github.com/supermarsx/bloody-level/actions/workflows/ci.yml)

Private, local-first desktop software for turning pathology-report PDFs into a longitudinal lab-results view.

> This is a tracking and review tool, not a diagnostic tool. It does not provide medical advice.

## The short version

1. Unlock the local vault with your password or registered passkey.
2. Drop one or more blood-work PDFs into **Ingest**.
3. Review parsed rows, flags, reference ranges, and parser diagnostics.
4. Follow a patient over time from the dashboard, reports, analyte trends, and compare view.

The app is designed for one person and one device:

- the database is encrypted at rest;
- original PDFs are copied into the app data directory;
- there is no telemetry, cloud sync, analytics, or required network service;
- PDF text extraction and optional OCR/model tiers run locally.

## Current status

The core workflow is implemented: vault setup and unlock, PDF ingestion, deterministic parsing, patient and report management, analyte trends, comparison charts, CSV export, audit diagnostics, ontology editing, settings, and cross-platform CI builds.

Optional model runtimes are intentionally incomplete. Tesseract OCR can be enabled for sparse or scanned PDFs. The current olmOCR-2 and Phi-4 integrations provide local model-path validation and lifecycle status, but do not yet return vision-OCR or repair results. See [the implementation status](docs/reference/status.md).

## Quick start

Prerequisites:

- Node.js 20 or newer
- Rust via [`rustup`](https://rustup.rs/), with Rust 1.95 or newer for the locked dependency set
- Tauri 2 system prerequisites for your operating system

From the repository root:

```bash
npm ci
npm run tauri:dev
```

The PDFium sidecar is downloaded during a native build when it is missing. If that is unavailable, place the matching `pdfium.dll`, `libpdfium.so`, or `libpdfium.dylib` in `src-tauri/binaries/` and rebuild.

## Useful commands

```bash
npm run tauri:dev       # desktop development app
npm run dev             # frontend-only Vite server
npm run check           # Svelte type check
npm run lint            # frontend lint
npm run format:check    # Prettier check for source and docs
npm run tauri:build     # local Tauri release bundle
npm run docs:serve      # local documentation server
npm run docs:build      # strict documentation build
```

Rust checks run from `src-tauri/`:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
```

## Documentation

The complete, searchable documentation lives in [docs/](docs/index.md) and is built with MkDocs Material. It is dark-first, supports light mode, uses nested navigation, and includes instant search with suggestions and highlighted results.

- [Install and run](docs/getting-started/installation.md)
- [Import and review a report](docs/user-guide/ingest.md)
- [Privacy, encryption, and backups](docs/data-and-security/privacy.md)
- [Architecture and development](docs/developer/architecture.md)
- [Release versioning](docs/developer/releases.md)
- [Troubleshooting](docs/reference/troubleshooting.md)

Install the docs toolchain with:

```bash
python -m pip install -r docs/requirements.txt
```

## Boundaries worth knowing

- PDF is the supported input format; HL7/FHIR and cloud imports are out of scope for the current version.
- Parsed values are descriptive records. A flag or reference-range comparison is not a diagnosis.
- Low-confidence rows remain visible in the report audit panel so they can be checked or linked to an ontology entry.
- Unsigned builds are expected until platform signing secrets are configured.
- Models are not bundled with the application and are never downloaded implicitly as part of normal use.

## Repository layout

| Path         | Purpose                                                                                    |
| ------------ | ------------------------------------------------------------------------------------------ |
| `src/`       | SvelteKit frontend and user interface                                                      |
| `src-tauri/` | Rust core, Tauri commands, encrypted database, migrations, parser, and native integrations |
| `ontology/`  | Seed analyte registry                                                                      |
| `docs/`      | MkDocs site source                                                                         |
| `mkdocs.yml` | Documentation theme, search, and navigation                                                |
| `static/`    | Frontend and README assets                                                                 |

## License

MIT. See [license.md](license.md).
