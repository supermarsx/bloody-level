# bloody-level

Local-first blood-test tracking for people who want a clear history of their own results.

Import a PDF, review the extracted values, compare trends over time, and keep the original report beside the structured record. The application is designed for private review and organization; it is not a diagnostic service.

![bloody-level analyte detail view with a synthetic demo patient and quarterly one-year history](assets/demo-analyte.png)

_Captured from the built-in `/demo/analyte` view. All patient data is synthetic: five readings at three-month intervals across one year._

## The core loop

| Step       | What happens                                                                                |
| ---------- | ------------------------------------------------------------------------------------------- |
| 1. Import  | Select one or more blood-test PDFs.                                                         |
| 2. Extract | The local parser reads text and records diagnostics for anything it cannot confidently map. |
| 3. Review  | Inspect values, reference ranges, flags, source metadata, and the original report.          |
| 4. Compare | Follow an analyte across reports, patients, dates, and optional HRT anchors.                |

## What is included

- An encrypted local vault for reports, patients, audit events, and settings.
- Support for PT-PT laboratory PDFs from CUF and Germano de Sousa.
- Deterministic PDF extraction with optional OCR support for sparse or scanned reports.
- Search across patients, analytes, and reports with keyboard navigation.
- Charts, reference bands, deltas, gaps, report filtering, and CSV export.
- Backup and restore from Settings, with the encrypted source PDFs preserved.
- A tier-aware audit trail so low-confidence or unmatched data stays visible.

## Start here

- [Install bloody-level](getting-started/installation.md)
- [Complete the first run](getting-started/first-run.md)
- [Import a PDF](user-guide/ingest.md)
- [Understand privacy and encryption](data-and-security/privacy.md)
- [Set up a development environment](developer/local-development.md)

Use the search button or press `/` in the documentation site to find a topic instantly. The navigation is grouped by task, with expandable subsections for user, security, developer, and reference material.

For source-specific guidance, see [supported report sources](user-guide/ingest.md#supported-report-sources).

## Scope and limitations

bloody-level currently focuses on PDF reports and descriptive review. It does not claim to diagnose conditions, replace a clinician, or provide cloud synchronization. Experimental model integrations are guarded and should be treated as incomplete until their status is explicitly shown in the app.

See [implementation status](reference/status.md) for the current capability boundary and [the original implementation plan](implementation-plan.md) for the detailed project specification.
