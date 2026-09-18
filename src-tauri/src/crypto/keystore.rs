use std::path::Path;

use rand::RngExt;
use secrecy::SecretBox;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

pub const UNLOCK_BACKOFF_GRACE_FAILURES: u32 = 2;

#[cfg(debug_assertions)]
pub const MAX_UNLOCK_BACKOFF_SECS: i64 = 10;

#[cfg(not(debug_assertions))]
pub const MAX_UNLOCK_BACKOFF_SECS: i64 = 300;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Keystore {
    pub version: u32,
    pub created_at: i64,
    pub updated_at: i64,
    pub failed_unlocks: u32,
    pub last_failed_unlock_at: Option<i64>,
    #[serde(default)]
    pub os_vault_enabled: bool,
    #[serde(default)]
    pub os_vault_auto_unlock: bool,
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
            os_vault_enabled: false,
            os_vault_auto_unlock: false,
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
        self.updated_at = chrono_now();
    }

    pub fn record_success(&mut self) {
        self.failed_unlocks = 0;
        self.last_failed_unlock_at = None;
        self.updated_at = chrono_now();
    }

    pub fn unlock_backoff_remaining_secs(&self) -> i64 {
        self.unlock_backoff_remaining_secs_at(chrono_now())
    }

    pub fn unlock_backoff_remaining_secs_at(&self, now: i64) -> i64 {
        let Some(last_failed_at) = self.last_failed_unlock_at else {
            return 0;
        };
        let delay = unlock_backoff_delay_secs(self.failed_unlocks);
        let elapsed = now.saturating_sub(last_failed_at).max(0);
        delay.saturating_sub(elapsed).max(0)
    }

    pub fn enforce_unlock_backoff(&self) -> AppResult<()> {
        let remaining = self.unlock_backoff_remaining_secs();
        if remaining > 0 {
            return Err(AppError::BadRequest(format!(
                "Too many failed unlock attempts. Try again in {remaining} seconds."
            )));
        }
        Ok(())
    }
}

pub fn unlock_backoff_delay_secs(failed_unlocks: u32) -> i64 {
    if failed_unlocks <= UNLOCK_BACKOFF_GRACE_FAILURES {
        return 0;
    }
    let exponent = (failed_unlocks - UNLOCK_BACKOFF_GRACE_FAILURES).min(30);
    (1_i64 << exponent).min(MAX_UNLOCK_BACKOFF_SECS)
}

pub fn generate_dmk() -> SecretBox<[u8; 32]> {
    let mut k = [0u8; 32];
    rand::rng().fill(&mut k);
    SecretBox::new(Box::new(k))
}

pub fn generate_salt(len: usize) -> Vec<u8> {
    let mut s = vec![0u8; len];
    rand::rng().fill(&mut s);
    s
}

fn chrono_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_keystore(failed_unlocks: u32, last_failed_unlock_at: Option<i64>) -> Keystore {
        Keystore {
            version: 1,
            created_at: 0,
            updated_at: 0,
            failed_unlocks,
            last_failed_unlock_at,
            os_vault_enabled: false,
            os_vault_auto_unlock: false,
            wrappers: vec![],
        }
    }

    #[test]
    fn backoff_has_initial_grace_then_doubles() {
        assert_eq!(unlock_backoff_delay_secs(0), 0);
        assert_eq!(unlock_backoff_delay_secs(1), 0);
        assert_eq!(unlock_backoff_delay_secs(2), 0);
        assert_eq!(unlock_backoff_delay_secs(3), 2);
        assert_eq!(unlock_backoff_delay_secs(4), 4);
        assert_eq!(unlock_backoff_delay_secs(5), 8);
    }

    #[test]
    fn backoff_is_capped() {
        assert_eq!(unlock_backoff_delay_secs(99), MAX_UNLOCK_BACKOFF_SECS);
    }

    #[test]
    fn remaining_backoff_uses_last_failure_time() {
        let ks = test_keystore(4, Some(100));
        assert_eq!(ks.unlock_backoff_remaining_secs_at(101), 3);
        assert_eq!(ks.unlock_backoff_remaining_secs_at(104), 0);
        assert_eq!(ks.unlock_backoff_remaining_secs_at(200), 0);
    }

    #[test]
    fn success_clears_backoff_state() {
        let mut ks = test_keystore(5, Some(100));
        ks.record_success();
        assert_eq!(ks.failed_unlocks, 0);
        assert_eq!(ks.last_failed_unlock_at, None);
        assert_eq!(ks.unlock_backoff_remaining_secs_at(101), 0);
    }
}
