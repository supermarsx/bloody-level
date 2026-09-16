# How data moves

bloody-level keeps the import path local and inspectable:

```text
PDF file
  -> SHA-256 identity check
  -> PDFium text extraction
  -> optional local OCR
  -> deterministic parsing and ontology mapping
  -> encrypted vault and audit events
  -> report, patient, chart, and CSV views
```

The original PDF is copied into application data. Extracted text, canonical rows, inline prior rows, parser diagnostics, and audit information are stored with the report so a later review can distinguish source data from interpretation.

## What the app does not do

- It does not upload reports to a required cloud service.
- It does not silently replace a failed parser with an invented value.
- It does not treat an unmatched line as a confirmed analyte.
- It does not turn a reference range or trend into a diagnosis.

Optional OCR and model-backed paths remain local and are only used when configured and available. Their status should be checked in Settings and in the report diagnostics.

## Trust the source in the right order

When something looks surprising, check the original PDF first, then the report diagnostics, then the canonical row and chart. A chart is a convenient view over stored data; it is not a replacement for the source document.
