use std::fs;
use std::path::{Path, PathBuf};

use chacha20poly1305::{
    aead::{Aead, Generate, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use rand::RngExt;
use secrecy::{ExposeSecret, SecretBox};

use crate::error::{AppError, AppResult};

/// Managed PDF cache format: magic/version, 24-byte XChaCha nonce, ciphertext
/// and authentication tag. The format deliberately does not resemble a PDF.
pub const MAGIC: &[u8] = b"BLPDFENC\x01";
const NONCE_LEN: usize = 24;

fn cipher(key: &SecretBox<[u8; 32]>) -> AppResult<XChaCha20Poly1305> {
    XChaCha20Poly1305::new_from_slice(key.expose_secret())
        .map_err(|e| AppError::Crypto(format!("PDF cache key: {e}")))
}

fn encrypted_bytes(plaintext: &[u8], key: &SecretBox<[u8; 32]>) -> AppResult<Vec<u8>> {
    let cipher = cipher(key)?;
    let nonce = XNonce::generate();
    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: plaintext,
                aad: MAGIC,
            },
        )
        .map_err(|_| AppError::Crypto("PDF cache encryption failed".into()))?;

    let mut output = Vec::with_capacity(MAGIC.len() + NONCE_LEN + ciphertext.len());
    output.extend_from_slice(MAGIC);
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

pub fn decrypt_bytes(encrypted: &[u8], key: &SecretBox<[u8; 32]>) -> AppResult<Vec<u8>> {
    let minimum = MAGIC.len() + NONCE_LEN + 16;
    if encrypted.len() < minimum || !encrypted.starts_with(MAGIC) {
        return Err(AppError::Crypto("managed PDF is not encrypted".into()));
    }
    let nonce_end = MAGIC.len() + NONCE_LEN;
    let nonce = XNonce::try_from(&encrypted[MAGIC.len()..nonce_end])
        .map_err(|_| AppError::Crypto("invalid managed PDF nonce".into()))?;
    cipher(key)?
        .decrypt(
            &nonce,
            Payload {
                msg: &encrypted[nonce_end..],
                aad: MAGIC,
            },
        )
        .map_err(|_| AppError::Crypto("managed PDF decryption failed".into()))
}

pub fn decrypt_file(path: &Path, key: &SecretBox<[u8; 32]>) -> AppResult<Vec<u8>> {
    decrypt_bytes(&fs::read(path)?, key)
}

/// Encrypt a source file into the managed cache. A temporary sibling is
/// written first so a failed encryption never replaces the destination with a
/// partial file. This also supports in-place conversion of legacy plaintext
/// cache files during unlock.
pub fn encrypt_file(source: &Path, destination: &Path, key: &SecretBox<[u8; 32]>) -> AppResult<()> {
    let plaintext = fs::read(source)?;
    let encrypted = encrypted_bytes(&plaintext, key)?;
    let temporary = temporary_path(destination);
    fs::write(&temporary, encrypted)?;

    if destination.exists() {
        fs::remove_file(destination)?;
    }
    if let Err(error) = fs::rename(&temporary, destination) {
        let _ = fs::remove_file(&temporary);
        return Err(error.into());
    }
    Ok(())
}

/// Convert PDF files written by pre-encryption builds. Unlocking fails if a
/// conversion cannot complete, so the application never proceeds while a
/// known managed PDF remains plaintext.
pub fn migrate_plaintext_pdf_cache(dir: &Path, key: &SecretBox<[u8; 32]>) -> AppResult<usize> {
    if !dir.exists() {
        return Ok(0);
    }

    let mut migrated = 0;
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let is_pdf = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"));
        if !is_pdf || !path.is_file() {
            continue;
        }
        let bytes = fs::read(&path)?;
        if bytes.starts_with(MAGIC) {
            continue;
        }
        encrypt_file(&path, &path, key)?;
        migrated += 1;
    }
    Ok(migrated)
}

fn temporary_path(destination: &Path) -> PathBuf {
    let mut random = [0u8; 12];
    rand::rng().fill(&mut random);
    let suffix = random
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("pdf");
    destination.with_file_name(format!(".{name}.{suffix}.tmp"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn key(value: u8) -> SecretBox<[u8; 32]> {
        SecretBox::new(Box::new([value; 32]))
    }

    #[test]
    fn round_trip_does_not_leave_pdf_plaintext() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.pdf");
        let cached = dir.path().join("cached.pdf");
        fs::write(&source, b"%PDF-1.7\nprivate result").unwrap();

        encrypt_file(&source, &cached, &key(7)).unwrap();
        let encrypted = fs::read(&cached).unwrap();
        assert!(encrypted.starts_with(MAGIC));
        assert!(!encrypted.starts_with(b"%PDF"));
        assert!(!encrypted
            .windows(b"private result".len())
            .any(|window| window == b"private result"));
        assert_eq!(
            decrypt_file(&cached, &key(7)).unwrap(),
            b"%PDF-1.7\nprivate result"
        );
        assert!(decrypt_file(&cached, &key(8)).is_err());
    }

    #[test]
    fn migrates_only_plaintext_pdf_files() {
        let dir = tempdir().unwrap();
        let legacy = dir.path().join("legacy.pdf");
        let other = dir.path().join("notes.txt");
        fs::write(&legacy, b"%PDF legacy").unwrap();
        fs::write(&other, b"leave alone").unwrap();

        assert_eq!(migrate_plaintext_pdf_cache(dir.path(), &key(9)).unwrap(), 1);
        assert!(fs::read(&legacy).unwrap().starts_with(MAGIC));
        assert_eq!(fs::read(&other).unwrap(), b"leave alone");
        assert_eq!(migrate_plaintext_pdf_cache(dir.path(), &key(9)).unwrap(), 0);
    }
}
