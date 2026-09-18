# Implementation status

This page describes the capability boundary users and contributors should rely on today.

## Available

- Local Tauri desktop shell with Svelte UI and Rust commands.
- Password-protected encrypted vault with lock/unlock and persisted unlock backoff.
- Optional passkey PRF registration and unlock where the platform supports it.
- Settings → Security controls for optional Windows Credential Manager, macOS
  Keychain, or Linux Secret Service vault wrapping, with explicit automatic
  unlock control.
- PDF import, SHA-256 duplicate detection, PDFium extraction, source-PDF retention, parsing, normalized rows, and audit diagnostics.
- Patient and report organization, global search, comparison views, charts, CSV export, ontology controls, and storage backup/restore flows.
- Cross-platform CI and a release matrix covering the supported desktop architectures.

## Available with conditions

- Tesseract OCR is compiled into distributed builds. A native executable can be
  bundled with `TESSERACT_BUNDLE_DIR`; Settings can download the `eng`/`por`
  language data, while an installed executable remains a supported fallback.
- PDFium may need a target-matched manual library when the build-time download is unavailable.
- Signing depends on platform credentials and repository secrets; default artifacts can be unsigned.

## In progress or experimental

- olmOCR-2 and Phi-4 integrations are compiled into distributed builds. Their
  settings controls can download/configure local model assets, but the current
  runtime still covers configuration/path validation and lifecycle status
  rather than a complete, guaranteed extraction path.
- Production confidence still depends on expanding synthetic and sanitized PDF fixture coverage across real-world report layouts.

For the full design history and acceptance criteria, read [the original implementation plan](../implementation-plan.md).
