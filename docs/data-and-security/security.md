# Security controls

The **Settings → Security** tier is the place to review and manage the local
protection layers for an instance.

## Native OS vault

When the vault is unlocked, choose **Enable native OS vault** to store a
device-unlock copy of the data master key in the operating system's credential
store:

| Platform | Native store                                                                             |
| -------- | ---------------------------------------------------------------------------------------- |
| Windows  | Windows Credential Manager                                                               |
| macOS    | Keychain                                                                                 |
| Linux    | Secret Service (for example, GNOME Keyring or KWallet through the system Secret Service) |

The database and managed PDF cache remain encrypted independently. The OS
vault is an additional local wrapper that makes unlocking convenient; it does
not remove the password or passkey recovery methods. **Disable OS vault**
removes that credential and leaves the encrypted vault intact.

## Automatic unlock

Automatic unlock is off by default. Enable it only on a device and operating
system account you control. On the next launch, bloody-level asks the native
credential store for the device-unlock copy; if that is unavailable, the
normal password and passkey controls remain available.

Keep automatic unlock off on shared, borrowed, or unattended machines. Anyone
who can unlock the same operating-system account may be able to use the app's
native unlock path.

## Other protection layers

- The database is SQLCipher-encrypted at rest.
- Managed PDF copies are encrypted with XChaCha20-Poly1305 and are only
  decrypted into a short-lived temporary hand-off when opened externally.
- Password unlock uses Argon2id and failed attempts use persisted backoff.
- Passkeys are an additional local unlock method where WebAuthn PRF is
  supported.
- Lock the app before leaving the device and protect exports, screenshots,
  original source files, and backups separately.

The OS vault cannot protect data from malware, a hostile administrator, or an
already-unlocked operating-system session. For the storage boundary and
recovery steps, see [Privacy and encryption](privacy.md) and [Backup and
restore](backup-restore.md).
