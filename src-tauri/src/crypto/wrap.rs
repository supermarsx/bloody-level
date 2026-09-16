use chacha20poly1305::{
    aead::{Aead, Generate, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use secrecy::{ExposeSecret, SecretBox};

use crate::error::{AppError, AppResult};

pub fn wrap_dmk(kek: &SecretBox<[u8; 32]>, dmk: &[u8; 32]) -> AppResult<(Vec<u8>, Vec<u8>)> {
    let cipher = XChaCha20Poly1305::new_from_slice(kek.expose_secret())
        .map_err(|e| AppError::Crypto(format!("wrap key: {e}")))?;
    let nonce = XNonce::generate();
    let ct = cipher
        .encrypt(&nonce, dmk.as_slice())
        .map_err(|e| AppError::Crypto(format!("wrap: {e}")))?;
    Ok((nonce.to_vec(), ct))
}

pub fn unwrap_dmk(
    kek: &SecretBox<[u8; 32]>,
    nonce: &[u8],
    ct: &[u8],
) -> AppResult<SecretBox<[u8; 32]>> {
    if nonce.len() != 24 {
        return Err(AppError::Crypto("xchacha20 nonce must be 24 bytes".into()));
    }
    let cipher = XChaCha20Poly1305::new_from_slice(kek.expose_secret())
        .map_err(|e| AppError::Crypto(format!("unwrap key: {e}")))?;
    let nonce = XNonce::try_from(nonce)
        .map_err(|_| AppError::Crypto("xchacha20 nonce must be 24 bytes".into()))?;
    let pt = cipher
        .decrypt(&nonce, ct)
        .map_err(|_| AppError::Crypto("unwrap failed (wrong key)".into()))?;
    if pt.len() != 32 {
        return Err(AppError::Crypto("dmk wrong size".into()));
    }
    let mut k = [0u8; 32];
    k.copy_from_slice(&pt);
    Ok(SecretBox::new(Box::new(k)))
}
