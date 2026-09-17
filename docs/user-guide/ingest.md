# Importing PDFs

Ingest turns a source report into a stored report, structured rows, and review diagnostics.

## Import a report

Open [Ingest], then drop one or more PDF files onto the import area or use the file picker. Batch imports are processed sequentially so each file has a clear result and progress state.

## Supported report sources

bloody-level supports Portuguese (PT-PT) pathology and laboratory PDF reports
from CUF and Germano de Sousa. Text-based PDFs are handled by the local PDFium
extractor. Scanned or text-poor reports can use the optional local OCR tier when
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
4. Use optional OCR when the extracted text is sparse and the selected tier is available.
5. Parse and normalize rows against the local ontology.
6. Write the report, rows, source copy, and audit events to the encrypted vault.

## Duplicate handling

The source SHA-256 is the identity check for an imported file. Importing the same bytes again is treated as a duplicate rather than creating a second report. A report's original PDF is copied into application data so later review does not depend on the original path continuing to exist.

## Ingestion tiers

Tier 1 uses the local PDFium path and should be the default. Tier 2 can add Tesseract OCR for scanned or text-poor reports, but only when it has been compiled/enabled and the required local executable or language data is available. Model-backed paths are optional and should be considered experimental until the app reports a complete result.

Choose the tier and model paths in Settings. The UI shows availability and diagnostics instead of silently pretending a missing backend ran.

## After import

Open the report detail and verify the patient, date, analyte names, values, units, flags, and reference ranges against the original PDF. Treat unmatched, unparsed, or low-confidence rows as review work. See [reviewing results](reviewing-results.md) for the available actions.

## When an import fails

The failure is kept with the import attempt where possible. Read the diagnostic message, check [troubleshooting](../reference/troubleshooting.md), and retry after correcting the source file or local backend. A failed parse should not be treated as an empty normal report.
