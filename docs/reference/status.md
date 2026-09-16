# Implementation status

This page describes the capability boundary users and contributors should rely on today.

## Available

- Local Tauri desktop shell with Svelte UI and Rust commands.
- Password-protected encrypted vault with lock/unlock and persisted unlock backoff.
- Optional passkey PRF registration and unlock where the platform supports it.
- PDF import, SHA-256 duplicate detection, PDFium extraction, source-PDF retention, parsing, normalized rows, and audit diagnostics.
- Patient and report organization, global search, comparison views, charts, CSV export, ontology controls, and storage backup/restore flows.
- Cross-platform CI and a release matrix covering the supported desktop architectures.

## Available with conditions

- Tesseract OCR is an optional, locally configured fallback and depends on the build and installed language data.
- PDFium may need a target-matched manual library when the build-time download is unavailable.
- Signing depends on platform credentials and repository secrets; default artifacts can be unsigned.

## In progress or experimental

- olmOCR-2 and Phi-4 integrations currently cover configuration/path validation and lifecycle status rather than a complete, guaranteed extraction path.
- Production confidence still depends on expanding synthetic and sanitized PDF fixture coverage across real-world report layouts.

For the full design history and acceptance criteria, read [the original implementation plan](../implementation-plan.md).
