# Settings and ontology

Settings controls how the desktop app looks, stores data, and interprets imported rows.

## Appearance

The theme control supports light, dark, and system modes. The navigation toggle cycles through them. Accent, density, font, and motion preferences are also kept locally. Dark mode is the documentation and product baseline, but the light scheme remains available for users who need it.

## Charts and comparison

Configure reference-band display, chart behavior, and saved comparison presets. These preferences change presentation and filtering; they do not change the source report or invent missing values.

## Ingestion

Use the Ingestion pipeline toggles to control the three stages independently:

1. **Tier 1 · PDF extraction** — the required PDFium foundation. Imports fail
   closed when this is disabled.
2. **Tier 2 · OCR** — allows Tesseract to run when embedded PDF text is sparse.
   It requires the compiled feature, a native Tesseract executable, and language
   data at runtime.
3. **Tier 3 · Hybrid OCR + LLM** — enables low-confidence escalation auditing
   for the optional olmOCR/Phi-4 model tiers. The current runtime records an
   explicit candidate and availability state; it does not claim to have changed
   report output unless a completed model path says so.

Below the pipeline controls, review the status of optional OCR/model paths. Use
the file picker to set a local model path when a feature requires one. A path
being configured does not prove that the model is installed, compatible, or used
successfully; check the displayed load status and report diagnostics. Model and
language downloads show their current file, byte progress, and a Cancel action.

## Ontology

The ontology maps report labels and units to canonical analytes. Search and filter entries, inspect aliases and unit information, and reload the seed data when needed.

Treat edits to bundled entries carefully: reloading from seed data can overwrite local changes. Reparse affected reports after a deliberate ontology change, then review the audit trail and source PDFs.

## Storage

Storage shows resolved application-data locations and vault statistics. It also provides encrypted backup export and restore. See [backup and restore](../data-and-security/backup-restore.md) for the safety sequence.

## About and advanced

About shows the application and build information. Advanced settings expose maintenance operations such as reload, reparse, diagnostics, and other guarded actions. Use them with a backup available when they can affect existing records.
