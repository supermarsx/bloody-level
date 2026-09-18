# Implementation status

This page describes the capability boundary users and contributors should rely on today.

## Available

- Local Tauri desktop shell with Svelte UI and Rust commands.
- Password-protected encrypted vault with lock/unlock and persisted unlock backoff.
- Optional passkey PRF registration and unlock where the platform supports it.
- New supported-platform vaults can use an OS-vault-only first-run path, with
  passkey management and authenticated master-key rotation available in
  Settings → Security.
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
- Ingestion stages are independently controlled in Settings: Tier 1 PDFium
  extraction, Tier 2 Tesseract OCR fallback, and Tier 3 hybrid OCR + LLM
  escalation. Tier 3 can load Phi-4 on demand and persist a Tier 3 report only
  when repaired output was actually used; missing or failed optional models
  fall back to the successful lower tier with diagnostics.
- PDFium may need a target-matched manual library when the build-time download is unavailable.
- Signing depends on platform credentials and repository secrets; default artifacts can be unsigned.

## In progress or experimental

- Phi-4 uses the bundled llama.cpp integration for local text repair when a
  compatible GGUF model is configured and loaded. olmOCR-2 model management and
  readiness reporting are present, but its safetensors vision inference runtime
  is not yet connected to ingestion.
- Production confidence still depends on expanding synthetic and sanitized PDF fixture coverage across real-world report layouts.

For the full design history and acceptance criteria, read [the original implementation plan](../implementation-plan.md).
