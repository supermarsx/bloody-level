# Architecture

bloody-level is a Tauri 2 desktop application with a Svelte 5/Vite frontend and a Rust core.

## Responsibility boundaries

| Area                         | Responsibility                                                                          |
| ---------------------------- | --------------------------------------------------------------------------------------- |
| Svelte routes and components | Navigation, forms, tables, charts, search, and user feedback.                           |
| Tauri commands               | The typed boundary between the frontend and native functionality.                       |
| Rust services                | Authentication, ingestion, parsing, reports, settings, exports, audit, and maintenance. |
| Encrypted SQLite             | Vault records, normalized rows, raw text, audit data, and configuration.                |
| Application data             | Database files, copied PDFs, keystore material, and optional local model assets.        |

The frontend does not open the database or hold the raw vault key. It calls registered Tauri commands, and the Rust side performs validation and persistence.

## Import path

The ingestion command validates the file, hashes it, extracts text, optionally invokes a configured OCR tier, parses rows, maps them through the analyte Library, and writes a report transaction. Diagnostics are retained for review rather than discarded when a mapping is incomplete.

## Main UI areas

The current route and navigation model is:

`Dashboard` · `Ingest` · `Patients` · `Records` · `Audit` · `Compare` · `Library` · `Settings`

Global search queries patient, analyte, and report data through a debounced command path. Settings cover appearance, charts, comparison, ingestion, Library, storage, about, and advanced maintenance.

## Security boundary

Authentication unlocks the vault in the Rust process. Database access, backup operations, and sensitive native actions stay behind Tauri commands. Any new command should validate its inputs, preserve the audit contract where appropriate, and avoid returning secrets to the webview.
