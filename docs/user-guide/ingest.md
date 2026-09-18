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

Tier 1 uses the local PDFium path and is always available when its matching
sidecar is present. Distributed builds also compile Tier 2 Tesseract OCR, Tier 3
olmOCR-2 vision OCR, and Tier 4 Phi-4 repair. OCR/model execution remains
opt-in and requires the matching local runtime assets; a compiled feature is not
the same as a loaded or usable model.

Choose the tier and model paths in Settings. The UI shows availability and diagnostics instead of silently pretending a missing backend ran.

## After import

Open the report detail and verify the patient, date, analyte names, values, units, flags, and reference ranges against the original PDF. Treat unmatched, unparsed, or low-confidence rows as review work. See [reviewing results](reviewing-results.md) for the available actions.

## When an import fails

The failure is kept with the import attempt where possible. Read the diagnostic message, check [troubleshooting](../reference/troubleshooting.md), and retry after correcting the source file or local backend. A failed parse should not be treated as an empty normal report.
