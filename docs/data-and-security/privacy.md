# Privacy and encryption

## Local-first by design

The desktop app stores its vault, copied PDFs, settings, and optional local model configuration on the device. Normal use does not require a hosted account, telemetry service, analytics endpoint, or cloud sync.

Build-time dependency downloads are separate from runtime data handling. A first build may access package registries or download a matching PDFium archive; imported health reports are not sent to those services by the app at runtime.

## Encrypted vault

The vault uses a SQLCipher v4-compatible encrypted SQLite database. It is configured with a 4096-byte page size, PBKDF2-HMAC-SHA512 key derivation, and 256,000 PBKDF2 iterations. The database master key is held by the Rust desktop process while the vault is unlocked and is not exposed to the frontend as a raw key.

The password wrapper uses Argon2id with 64 MiB memory, three iterations, and one lane. A strong password still matters: encryption cannot recover data when the password is forgotten.

## Passkeys

On supported platforms, a passkey can unlock a password-wrapped vault key using the authenticator PRF capability. Passkeys are an additional local unlock mechanism, not a cloud account or a replacement for a backup password.

## Practical limits

Encryption protects stored data at rest, but it cannot protect an unlocked device from malware, a hostile administrator, or someone who can read exported files. CSV exports, copied PDFs, screenshots, and backups need their own protection. Lock the app when it is unattended and keep backups in a protected location.

For the exact storage and recovery sequence, see [Backup and restore](backup-restore.md).
