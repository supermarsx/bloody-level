# bloody-level

Local-first desktop app for tracking blood-work / lab levels over time. Built with **SvelteKit** (static adapter) and **Tauri 2** (Rust backend, SQLite storage).

## Prerequisites

- Node.js 20+
- Rust toolchain (`rustup`) with the Tauri prerequisites for your OS
- The PDF sidecar (`pdfium.dll` / platform equivalent) placed in `src-tauri/binaries/` (not committed)

## Getting started

```bash
npm install          # install frontend deps
npm run tauri:dev    # run the desktop app in dev mode
```

## Build

```bash
npm run tauri:build  # produce a release bundle
```

## Layout

- `src/` — SvelteKit frontend (routes, components)
- `src-tauri/` — Rust backend, Tauri config, SQLite migrations
- `ontology/` — analyte seed data
- `PLAN.md` — design/architecture notes

## Notes

App data (databases, keystore, imported reports, downloaded models) lives outside the
repo and is git-ignored. See `.gitignore` for the full list.
