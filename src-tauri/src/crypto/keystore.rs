use std::path::Path;

use rand::RngCore;
use secrecy::SecretBox;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Keystore {
    pub version: u32,
    pub created_at: i64,
    pub updated_at: i64,
    pub failed_unlocks: u32,
    pub last_failed_unlock_at: Option<i64>,
    pub wrappers: Vec<Wrapper>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum Wrapper {
    Password {
        kdf: String,
        m: u32,
        t: u32,
        p: u32,
        salt_b64: String,
        nonce_b64: String,
        ciphertext_b64: String,
    },
    Passkey {
        label: String,
        credential_id_b64: String,
        prf_salt_b64: String,
        nonce_b64: String,
        ciphertext_b64: String,
    },
}

impl Keystore {
    pub fn empty() -> Self {
        let now = chrono_now();
        Self {
            version: 1,
            created_at: now,
            updated_at: now,
            failed_unlocks: 0,
            last_failed_unlock_at: None,
            wrappers: vec![],
        }
    }

    pub fn load(path: &Path) -> AppResult<Self> {
        let bytes = std::fs::read(path)?;
        let ks: Keystore = serde_json::from_slice(&bytes)?;
        Ok(ks)
    }

    pub fn save(&self, path: &Path) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(self)?;
        // Atomic-ish write: temp then rename.
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, bytes)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    pub fn has_password(&self) -> bool {
        self.wrappers
            .iter()
            .any(|w| matches!(w, Wrapper::Password { .. }))
    }

    pub fn has_passkey(&self) -> bool {
        self.wrappers
            .iter()
            .any(|w| matches!(w, Wrapper::Passkey { .. }))
    }

    pub fn passkeys(&self) -> Vec<&Wrapper> {
        self.wrappers
            .iter()
            .filter(|w| matches!(w, Wrapper::Passkey { .. }))
            .collect()
    }

    pub fn replace_password(&mut self, w: Wrapper) -> AppResult<()> {
        if !matches!(w, Wrapper::Password { .. }) {
            return Err(AppError::BadRequest("not a password wrapper".into()));
        }
        self.wrappers
            .retain(|w| !matches!(w, Wrapper::Password { .. }));
        self.wrappers.push(w);
        self.updated_at = chrono_now();
        Ok(())
    }

    pub fn add_passkey(&mut self, w: Wrapper) -> AppResult<()> {
        if !matches!(w, Wrapper::Passkey { .. }) {
            return Err(AppError::BadRequest("not a passkey wrapper".into()));
        }
        self.wrappers.push(w);
        self.updated_at = chrono_now();
        Ok(())
    }

    pub fn record_failure(&mut self) {
        self.failed_unlocks = self.failed_unlocks.saturating_add(1);
        self.last_failed_unlock_at = Some(chrono_now());
    }

    pub fn record_success(&mut self) {
        self.failed_unlocks = 0;
        self.last_failed_unlock_at = None;
    }
}

pub fn generate_dmk() -> SecretBox<[u8; 32]> {
    let mut k = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut k);
    SecretBox::new(Box::new(k))
}

pub fn generate_salt(len: usize) -> Vec<u8> {
    let mut s = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut s);
    s
}

fn chrono_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
