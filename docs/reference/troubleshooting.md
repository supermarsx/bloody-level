# Troubleshooting

## The app will not start because PDFium is missing

Install or supply the PDFium library that matches the operating system and architecture, then rebuild. The build helper may download it automatically, but a network failure only produces a warning; it does not make an incompatible library safe to use.

## A PDF is rejected

Confirm that the file exists, opens in a normal PDF viewer, and is not still being written by another application. Password-protected or malformed PDFs may need to be exported again without protection. Check the import diagnostic for the exact stage that failed.

## The report imported but has no useful rows

Open the report diagnostics. A scanned or text-poor PDF may need the optional OCR tier, and a layout the parser does not recognize may produce unmatched lines. Compare with the original PDF before enabling another tier or editing ontology aliases.

## The same report appears twice

Duplicate detection uses the source bytes' SHA-256. If two files look the same but differ in metadata or bytes, they can have different identities. Review the source metadata and remove the unintended report after confirming which copy is correct.

## A value is unmatched or low confidence

Treat it as unverified. Check the original report, unit, reference range, and parser diagnostics. Correct the ontology or reparse only when the mapping is understood, then review the updated row.

## OCR or a model is unavailable

Go to Settings → Ingestion and check the selected tier, configured path, and
load status. Use **Download eng data** or **Download por data** for Tesseract
language files, or the model's explicit **Download** button. A configured path
must point to a compatible local installation. Downloads are user-initiated;
the app never fetches large OCR/model assets during startup or ingestion.

If the native Tesseract executable is missing, set `TESSERACT_BUNDLE_DIR` for
a source build or install a platform package using the [Tesseract download
guidance](https://github.com/tesseract-ocr/tessdoc/blob/main/Downloads.md),
then restart the app and check the tier again.

## OS vault unlock is unavailable

Open Settings → Security. The status card identifies the native store and any
access error. On Linux, ensure a Secret Service provider such as GNOME Keyring
or KWallet is running. You can always disable the native path and use the
vault password or a registered passkey instead.

## Unlock is temporarily delayed

Persisted backoff is active after failed attempts. Wait for the displayed delay and use the correct password or registered passkey. Repeated attempts do not bypass the delay.

## Restore needs a restart

Complete the restore, close and reopen the app, then unlock with the restored vault's password. Keep the safety copy until several reports and original PDFs have been checked.

## The documentation build fails

Install the pinned documentation dependency range and run the strict build from the repository root:

```bash
python -m pip install -r docs/requirements.txt
mkdocs build --strict
```

The error usually names a missing page in `mkdocs.yml` or a malformed Markdown extension.
