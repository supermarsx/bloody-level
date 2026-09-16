# Backup and restore

Use Settings → Storage to export or restore a vault backup.

## Export a backup

1. Lock or pause other vault activity before starting.
2. Open Settings → Storage and choose the backup export action.
3. Select a location outside the app's own data directory.
4. Protect the resulting backup as sensitive health data.
5. Keep and test the password needed to unlock it.

An export includes the encrypted vault database, key-store material, copied source PDFs, available local model files, and a manifest describing the backup. The encryption is preserved; the backup is not a plaintext dump.

## Restore a backup

1. Make a fresh backup of the current vault if it contains anything you may need.
2. Ensure the app is unlocked and follow Settings → Storage → Restore.
3. Select the backup and confirm the replacement.
4. Allow the current vault to be moved aside as a safety copy.
5. Restart the application and unlock with the password belonging to the restored vault.
6. Open a few reports and verify that the original PDFs and patient history are present.

Restore is a replacement operation, not a merge. Do not delete the safety copy until you have tested the restored vault.

## Protect backups

Do not store a backup inside the directory it is backing up. Avoid unencrypted removable media and shared folders. If you export data separately as CSV, apply the same care: it is readable outside the vault's encryption boundary.
