# First run

## Create the vault password

On first launch, bloody-level asks you to create the password that protects the local vault. The password policy requires at least 10 characters and must pass the strength and entropy checks. Common words, obvious sequences, and repeated patterns are rejected.

Choose a password you can retain safely. There is no server-side password reset because the vault is local and encrypted. A forgotten password can make the encrypted data unrecoverable.

If the password cannot be recovered, the welcome screen provides **Reset this instance**. This permanently removes the encrypted database, password, passkeys, imported reports, PDFs, and local model files from the device. It is protected by an explicit confirmation and cannot be undone, so use a vault export or backup first whenever possible.

## Password-free OS-vault setup

On supported platforms, the welcome screen can create a vault without a
password. The generated data master key is protected by the operating system's
credential store and automatic OS-vault unlock is enabled; bloody-level never
stores the vault password there. Add a passkey from Settings as an additional
recovery method; on a shared device, prefer a strong vault password or passkey
instead of relying only on the OS account.

## Optional passkey unlock

After setup, you can register a passkey from Settings → Security when the
platform WebView and authenticator support the required PRF operation. On
Windows this commonly means Windows Hello or another compatible authenticator.
Passkey registration is an additional unlock route and can also protect a
password-free OS-vault setup.

If the platform cannot provide the required capability, the password flow remains available and the app reports why passkey setup was skipped.

## Unlock and lock

Unlock the app with the password, the native OS vault, or a registered passkey.
For a password-free OS-vault vault, leaving the password field empty invokes the
native key store; an empty string is never accepted as a password for a
password-protected vault. Use the lock control in the navigation bar whenever
you leave the device. The vault key is kept in the Rust process while unlocked
and is cleared when the app locks or closes.

Repeated failed unlocks use persisted backoff. Wait for the displayed retry time rather than repeatedly submitting guesses.

## Import your first report

1. Open [Ingest](../user-guide/ingest.md).
2. Drop a PDF on the import area or choose it with the file picker.
3. Wait for hashing, extraction, parsing, and database writing to finish.
4. Open the report detail and check the source metadata, values, flags, and audit diagnostics.

Keep the original report available for comparison. If a value is unmatched or looks wrong, use the report's diagnostics and original-PDF action before relying on the structured value.

## Good first settings

Review [Settings and Library](../user-guide/settings-and-library.md) before importing a large archive. In particular, confirm the ingestion tier, model paths if you use optional OCR, chart defaults, and the resolved storage location shown by the app.
