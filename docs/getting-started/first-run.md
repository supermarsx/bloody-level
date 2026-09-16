# First run

## Create the vault password

On first launch, bloody-level asks you to create the password that protects the local vault. The password policy requires at least 10 characters and must pass the strength and entropy checks. Common words, obvious sequences, and repeated patterns are rejected.

Choose a password you can retain safely. There is no server-side password reset because the vault is local and encrypted. A forgotten password can make the encrypted data unrecoverable.

## Optional passkey unlock

After the password is set, you can register a passkey when the platform WebView and authenticator support the required PRF operation. On Windows this commonly means Windows Hello or another compatible authenticator. Passkey registration is an additional unlock route; it does not remove the vault password.

If the platform cannot provide the required capability, the password flow remains available and the app reports why passkey setup was skipped.

## Unlock and lock

Unlock the app with the password or a registered passkey. Use the lock control in the navigation bar whenever you leave the device. The vault key is kept in the Rust process while unlocked and is cleared when the app locks or closes.

Repeated failed unlocks use persisted backoff. Wait for the displayed retry time rather than repeatedly submitting guesses.

## Import your first report

1. Open [Ingest](../user-guide/ingest.md).
2. Drop a PDF on the import area or choose it with the file picker.
3. Wait for hashing, extraction, parsing, and database writing to finish.
4. Open the report detail and check the source metadata, values, flags, and audit diagnostics.

Keep the original report available for comparison. If a value is unmatched or looks wrong, use the report's diagnostics and original-PDF action before relying on the structured value.

## Good first settings

Review [Settings and ontology](../user-guide/settings-and-ontology.md) before importing a large archive. In particular, confirm the ingestion tier, model paths if you use optional OCR, chart defaults, and the resolved storage location shown by the app.
