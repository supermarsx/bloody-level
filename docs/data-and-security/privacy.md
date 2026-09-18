# Privacy and encryption

## Local-first by design

The desktop app stores its vault, copied PDFs, settings, and optional local model configuration on the device. Normal use does not require a hosted account, telemetry service, analytics endpoint, or cloud sync.

Build-time dependency downloads are separate from runtime data handling. A first build may access package registries or download a matching PDFium archive; imported health reports are not sent to those services by the app at runtime.

## Encrypted vault

The vault uses a SQLCipher v4-compatible encrypted SQLite database. It is configured with a 4096-byte page size, PBKDF2-HMAC-SHA512 key derivation, and 256,000 PBKDF2 iterations. The database master key is held by the Rust desktop process while the vault is unlocked and is not exposed to the frontend as a raw key.

The password wrapper uses Argon2id with 64 MiB memory, three iterations, and one lane. A strong password still matters: encryption cannot recover data when the password is forgotten.

## Encrypted source-PDF cache

When a report is imported, bloody-level stores a managed copy under the
application data directory so later review does not depend on the original
file remaining in place. That managed PDF cache is encrypted with
XChaCha20-Poly1305 using an HKDF-derived key scoped to the vault. Existing
plaintext cache files from older builds are converted during unlock before the
vault is made available.

Opening a report creates a temporary plaintext hand-off for the operating
system's PDF viewer. It is kept outside the vault and scheduled for cleanup
after a short review window. The original source file selected for import is
not moved or encrypted by the app; protect that file separately.

## Passkeys

On supported platforms, a passkey can unlock a password-wrapped vault key using the authenticator PRF capability. Passkeys are an additional local unlock mechanism, not a cloud account. They can also protect a password-free OS-vault setup.

## Native OS vault

Settings → Security can place a device-unlock copy of the data master key in
Windows Credential Manager, macOS Keychain, or Linux Secret Service. This is a
native OS-vault wrapper around the already encrypted vault, not a replacement
for database encryption. New supported-platform vaults enable it by default and
can be created without a password. It can be disabled without deleting any
reports or PDFs. Automatic unlock should remain off on shared devices. See
[Security controls](security.md) for the setup and warning details.

## Practical limits

Encryption protects stored data at rest, but it cannot protect an unlocked device from malware, a hostile administrator, or someone who can read exported files. CSV exports, copied PDFs, screenshots, and backups need their own protection. Lock the app when it is unattended and keep backups in a protected location.

For the exact storage and recovery sequence, see [Backup and restore](backup-restore.md).
