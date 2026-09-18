# Security controls

The **Settings → Security** tier is the place to review and manage the local
protection layers for an instance.

## Native OS vault

New vaults enable the native OS vault by default where the platform supports a
credential store. Existing unlocked vaults can choose **Enable native OS vault**
to store a device-unlock copy of the data master key in the operating system's
credential store:

Only the data master key is placed in the OS vault. The vault password is never
stored there. A password-free OS-vault vault can be unlocked by leaving the
password field empty, which retrieves the key from the configured native store;
an empty string is not treated as a valid password when a password wrapper
exists.

| Platform | Native store                                                                             |
| -------- | ---------------------------------------------------------------------------------------- |
| Windows  | Windows Credential Manager                                                               |
| macOS    | Keychain                                                                                 |
| Linux    | Secret Service (for example, GNOME Keyring or KWallet through the system Secret Service) |

The database and managed PDF cache remain encrypted independently. The OS
vault is an additional local wrapper that makes unlocking convenient; it does
not remove password or passkey recovery methods. A supported platform can use
the OS-vault-only first-run option, so a password is not mandatory. **Disable OS vault**
removes that credential and leaves the encrypted vault intact. Keep a strong
vault password or passkey before doing this; the app warns that the password
protects access to the data and blocks disabling the last recovery route.

## Automatic unlock

Automatic unlock is enabled for new OS-vault-only vaults. Enable it only on a
device and operating system account you control. On the next launch,
bloody-level asks the native credential store for the device-unlock copy; if
that is unavailable, the normal password and passkey controls remain available.

Keep automatic unlock off on shared, borrowed, or unattended machines. Anyone
who can unlock the same operating-system account may be able to use the app's
native unlock path.

## Other protection layers

- The database is SQLCipher-encrypted at rest.
- Managed PDF copies are encrypted with XChaCha20-Poly1305 and are only
  decrypted into a short-lived temporary hand-off when opened externally.
- Managed PDF filenames use a fresh per-file salt and an instance-keyed digest;
  the original source SHA-256 is retained only inside the encrypted database
  for duplicate detection and is not used as the on-disk filename. Existing
  hash-named cache files are migrated when the vault is unlocked.
- Password unlock uses Argon2id and failed attempts use persisted backoff.
- Passkeys are an additional local unlock method where WebAuthn PRF is
  supported.
- Change an existing vault password in Settings → Security, or add the first
  password to an OS-vault-only instance while it is unlocked.
- Remove the password wrapper when a passkey or native OS-vault recovery route
  remains configured. The app blocks removal of the last recovery method.
- Lock the app before leaving the device and protect exports, screenshots,
  original source files, and backups separately.

The OS vault cannot protect data from malware, a hostile administrator, or an
already-unlocked operating-system session. For the storage boundary and
recovery steps, see [Privacy and encryption](privacy.md) and [Backup and
restore](backup-restore.md).

## Passkeys and master-key rotation

Settings → Security can register, list, and remove WebAuthn passkeys with PRF
support. Removing the last passkey is blocked when it would leave the vault
without a password or OS-vault recovery route.

Rotate master key creates a fresh data master key and re-keys the encrypted
database, managed PDFs, OS-vault credential, password wrapper, and passkey
wrappers. The already-unlocked session authorizes the rotation; fresh
assertions are still required from every registered passkey. Keep a current
backup before rotating.
