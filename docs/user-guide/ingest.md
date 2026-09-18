# Importing PDFs

Ingest turns a source report into a stored report, structured rows, and review diagnostics.

## Import a report

Open [Ingest], then drop one or more PDF files onto the import area or use the file picker. Batch imports are processed sequentially so each file has a clear result and progress state.

## Supported report sources

bloody-level supports Portuguese (PT-PT) pathology and laboratory PDF reports
from CUF and Germano de Sousa. Text-based PDFs are handled by the local PDFium
extractor. Scanned or text-poor reports can use the locally compiled OCR tiers when
it is configured and available.

Provider layouts may vary by department, report type, and revision. After every
import, compare the patient, date, analyte names, values, units, flags, and
reference ranges with the original PDF. CUF and Germano de Sousa are document
sources referenced for compatibility; bloody-level is independent and is not
affiliated with either provider.

The pipeline records these stages:

1. Verify that the file exists and is a PDF.
2. Hash the source bytes with SHA-256.
3. Extract text with PDFium.
4. Use OCR when the extracted text is sparse and the selected tier is available.
5. Parse and normalize rows against the local ontology.
6. Encrypt the managed source copy, then write the report, rows, and audit events to the encrypted vault.

## Duplicate handling

The source SHA-256 is the identity check for an imported file. Importing the same bytes again is treated as a duplicate rather than creating a second report. A report's original PDF is copied into application data so later review does not depend on the original path continuing to exist. The managed copy is encrypted at rest; opening it creates a temporary hand-off for the system PDF viewer.

## Ingestion tiers

Settings presents three independent ingestion stages:

1. **Tier 1 — PDF extraction:** the local PDFium baseline. This must be enabled
   for imports to proceed.
2. **Tier 2 — OCR:** Tesseract fallback for sparse-text PDFs. It is used only
   when the toggle is enabled and the native executable and language data are
   available.
3. **Tier 3 — Hybrid OCR + LLM:** when Tier 2 reports low OCR confidence and
   the Phi-4 model is enabled, the app loads it on demand, repairs the OCR text,
   and parses that repaired text. The stored report is marked as Tier 3 only
   when a non-empty model output was actually used. If the model is missing,
   unloaded, or fails, the app keeps the Tier 1/2 text and records the reason.
   olmOCR-2 remains an optional vision resource and is shown as unavailable until
   its native inference runtime is ready.

Distributed builds compile the optional Tesseract, olmOCR-2, and Phi-4
integrations, but compiled code is not the same as an enabled, downloaded, or
loaded runtime asset. Choose the stage toggles and model paths in Settings. The
UI shows availability, download progress, cancellation, and diagnostics instead
of pretending a missing backend ran. A normal text-based report should remain
Tier 1: enabling a higher tier makes it an escalation fallback, not a forced
second pass over every document.

## After import

Open the report detail and verify the patient, date, analyte names, values, units, flags, and reference ranges against the original PDF. Treat unmatched, unparsed, or low-confidence rows as review work. See [reviewing results](reviewing-results.md) for the available actions.

## When an import fails

The failure is kept with the import attempt where possible. Read the diagnostic message, check [troubleshooting](../reference/troubleshooting.md), and retry after correcting the source file or local backend. A failed parse should not be treated as an empty normal report.
