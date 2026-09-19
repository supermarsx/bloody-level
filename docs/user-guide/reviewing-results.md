# Reviewing results

Review every newly imported report before using it for comparison. The structured view is designed to make uncertainty visible while keeping the source report close at hand.

## Report detail

Report detail includes the source metadata, imported rows, values, units, flags, reference ranges, and parser diagnostics. Depending on the report state, you can:

- Open the original PDF.
- Reparse the report after changing Library or ingestion settings.
- Export the report rows as CSV.
- Move through previous and next reports.
- Delete a report and its associated imported data when appropriate.

## Diagnostics and audit

Look for unmatched analytes, unparsed lines, unrecognized units, low-confidence mappings, and missing patient or date fields. The Audit area provides a broader chronological view of ingestion and other important vault operations.

Diagnostics are evidence about how the value was obtained; they are not a medical interpretation. If the structured value disagrees with the PDF, use the original document as the source of truth and correct or exclude the row.

## Analyte detail

Opening an analyte from a report or search result shows the patient-scoped history where available. Review the time series, deltas, gaps, source reports, and reference bands together. A change outside a reference band is a prompt to review context, not a diagnosis.

CSV export is useful for personal analysis and sharing with a clinician, but exported files are no longer protected by the vault once they leave the app.
