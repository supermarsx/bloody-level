use argon2::{Algorithm, Argon2, Params, Version};
use hkdf::Hkdf;
use secrecy::SecretBox;
use sha2::Sha256;

use crate::error::{AppError, AppResult};

pub const KEY_LEN: usize = 32;

// Argon2id params — m=64MiB, t=3, p=1. Tuned for desktop on a single core to
// keep unlock latency tolerable while still expensive enough to deter offline attack.
pub const ARGON2_M_KIB: u32 = 65_536;
pub const ARGON2_T: u32 = 3;
pub const ARGON2_P: u32 = 1;

pub fn derive_kek_from_password(password: &str, salt: &[u8]) -> AppResult<SecretBox<[u8; 32]>> {
    let params = Params::new(ARGON2_M_KIB, ARGON2_T, ARGON2_P, Some(KEY_LEN))
        .map_err(|e| AppError::Crypto(format!("argon2 params: {e}")))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; KEY_LEN];
    argon
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| AppError::Crypto(format!("argon2 hash: {e}")))?;
    Ok(SecretBox::new(Box::new(key)))
}

pub fn derive_kek_from_prf(prf_output: &[u8], info: &[u8]) -> AppResult<SecretBox<[u8; 32]>> {
    let hk = Hkdf::<Sha256>::new(None, prf_output);
    let mut key = [0u8; KEY_LEN];
    hk.expand(info, &mut key)
        .map_err(|e| AppError::Crypto(format!("hkdf: {e}")))?;
    Ok(SecretBox::new(Box::new(key)))
}
