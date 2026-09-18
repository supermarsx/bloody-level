//! Small, explicit adapter around the platform credential store.
//!
//! The OS vault stores a device-unlock copy of the data master key. The DB and
//! managed PDF cache remain independently encrypted; the vault is an optional
//! convenience/recovery wrapper, never the only authentication method.

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use secrecy::SecretBox;

use crate::error::{AppError, AppResult};

const SERVICE: &str = "bloody-level";
const USERNAME: &str = "vault-dmk-v1";

#[derive(Debug, Clone, serde::Serialize)]
pub struct Status {
    pub supported: bool,
    pub platform: &'static str,
    pub credential_present: bool,
    pub error: Option<String>,
}

pub fn platform() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "Windows Credential Manager"
    }
    #[cfg(target_os = "macos")]
    {
        "macOS Keychain"
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        "Linux Secret Service"
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
    {
        "unsupported platform"
    }
}

fn entry() -> AppResult<keyring::Entry> {
    keyring::Entry::new(SERVICE, USERNAME)
        .map_err(|error| AppError::Internal(format!("OS vault is unavailable: {error}")))
}

pub fn status() -> Status {
    let platform = platform();
    if !cfg!(any(target_os = "windows", target_os = "macos", unix)) {
        return Status {
            supported: false,
            platform,
            credential_present: false,
            error: Some("This operating system has no supported native credential store.".into()),
        };
    }

    let entry = match keyring::Entry::new(SERVICE, USERNAME) {
        Ok(entry) => entry,
        Err(error) => {
            return Status {
                supported: true,
                platform,
                credential_present: false,
                error: Some(format!("OS vault probe failed: {error}")),
            }
        }
    };
    match entry.get_password() {
        Ok(secret) => Status {
            supported: true,
            platform,
            credential_present: B64
                .decode(secret)
                .map(|bytes| bytes.len() == 32)
                .unwrap_or(false),
            error: None,
        },
        Err(keyring::Error::NoEntry) => Status {
            supported: true,
            platform,
            credential_present: false,
            error: None,
        },
        Err(error) => Status {
            supported: true,
            platform,
            credential_present: false,
            error: Some(error.to_string()),
        },
    }
}

pub fn set_dmk(dmk: &[u8; 32]) -> AppResult<()> {
    let entry = entry()?;
    // A password-shaped base64 value works consistently across all v1
    // backends, including stores with text-only APIs.
    let encoded = B64.encode(dmk);
    entry
        .set_password(&encoded)
        .map_err(|error| AppError::Internal(format!("save to OS vault failed: {error}")))
}

pub fn get_dmk() -> AppResult<SecretBox<[u8; 32]>> {
    let encoded = entry()?
        .get_password()
        .map_err(|error| AppError::Internal(format!("read from OS vault failed: {error}")))?;
    let bytes = B64
        .decode(encoded)
        .map_err(|error| AppError::Crypto(format!("OS vault secret is malformed: {error}")))?;
    if bytes.len() != 32 {
        return Err(AppError::Crypto(
            "OS vault secret has the wrong size".into(),
        ));
    }
    let mut dmk = [0u8; 32];
    dmk.copy_from_slice(&bytes);
    Ok(SecretBox::new(Box::new(dmk)))
}

pub fn delete() -> AppResult<()> {
    match entry()?.delete_credential() {
        Ok(()) => Ok(()),
        Err(error) if error.to_string().contains("No matching credential") => Ok(()),
        Err(error) => Err(AppError::Internal(format!(
            "remove OS vault credential failed: {error}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_name_is_non_empty() {
        assert!(!platform().is_empty());
    }
}
