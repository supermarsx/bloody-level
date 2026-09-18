# Backup and restore

Use Settings → Storage to export or restore a vault backup. Both operations use
a `.zip` package; the files inside remain in their encrypted-at-rest formats.

## Export a backup

1. Lock or pause other vault activity before starting.
2. Open Settings → Storage and choose **Export vault ZIP**.
3. Select a destination folder outside the app's own data directory.
4. Protect the resulting `.zip` package as sensitive health data.
5. Keep and test the password needed to unlock it.

An export includes the encrypted vault database, key-store material, copied source PDFs, available local model files, and an embedded manifest describing the backup. The encryption is preserved; the ZIP is not a plaintext dump.

## Restore a backup

1. Make a fresh backup of the current vault if it contains anything you may need.
2. Open Settings → Storage and choose **Import vault ZIP**.
3. Select the `.zip` backup and confirm the replacement.
4. Allow the current vault to be moved aside as a safety copy.
5. Restart the application and unlock with the password belonging to the restored vault.
6. Open a few reports and verify that the original PDFs and patient history are present.

Restore is a replacement operation, not a merge. Do not delete the safety copy until you have tested the restored vault.

## Protect backups

Do not store a backup inside the directory it is backing up. Avoid unencrypted removable media and shared folders. If you export data separately as CSV, apply the same care: it is readable outside the vault's encryption boundary.
