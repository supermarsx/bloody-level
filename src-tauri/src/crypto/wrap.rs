use chacha20poly1305::{
    aead::{generic_array::GenericArray, Aead, AeadCore, KeyInit, OsRng},
    XChaCha20Poly1305,
};
use secrecy::{ExposeSecret, SecretBox};

use crate::error::{AppError, AppResult};

pub fn wrap_dmk(
    kek: &SecretBox<[u8; 32]>,
    dmk: &[u8; 32],
) -> AppResult<(Vec<u8>, Vec<u8>)> {
    let cipher = XChaCha20Poly1305::new(GenericArray::from_slice(kek.expose_secret()));
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
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
    let cipher = XChaCha20Poly1305::new(GenericArray::from_slice(kek.expose_secret()));
    let nonce = GenericArray::from_slice(nonce);
    let pt = cipher
        .decrypt(nonce, ct)
        .map_err(|_| AppError::Crypto("unwrap failed (wrong key)".into()))?;
    if pt.len() != 32 {
        return Err(AppError::Crypto("dmk wrong size".into()));
    }
    let mut k = [0u8; 32];
    k.copy_from_slice(&pt);
    Ok(SecretBox::new(Box::new(k)))
}
