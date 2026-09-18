use secrecy::ExposeSecret;
use serde::Serialize;
use tauri::State;

use crate::crypto::keystore::Keystore;
use crate::crypto::native_vault;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct SecurityStatus {
    pub master_key_enabled: bool,
    pub os_vault_supported: bool,
    pub os_vault_platform: String,
    pub os_vault_enabled: bool,
    pub os_vault_auto_unlock: bool,
    pub os_vault_credential_present: bool,
    pub session_unlocked: bool,
    pub last_error: Option<String>,
}

fn keystore_status(state: &AppState) -> AppResult<(bool, bool, bool)> {
    if !state.keystore_path().exists() {
        return Ok((false, false, false));
    }
    let ks = Keystore::load(&state.keystore_path())?;
    Ok((true, ks.os_vault_enabled, ks.os_vault_auto_unlock))
}

#[tauri::command]
pub async fn security_status(state: State<'_, AppState>) -> AppResult<SecurityStatus> {
    let (master_key_enabled, enabled, auto_unlock) = keystore_status(&state)?;
    let native = native_vault::status();
    Ok(SecurityStatus {
        master_key_enabled,
        os_vault_supported: native.supported,
        os_vault_platform: native.platform.to_string(),
        os_vault_enabled: enabled,
        os_vault_auto_unlock: auto_unlock,
        os_vault_credential_present: native.credential_present,
        session_unlocked: state.db.lock().await.is_some(),
        last_error: native.error,
    })
}

/// Store the in-memory DMK in the platform credential store. The vault must
/// already be unlocked so enabling this never asks the OS store to decrypt a
/// key derived from an unverified request.
#[tauri::command]
pub async fn security_enable_os_vault(state: State<'_, AppState>) -> AppResult<SecurityStatus> {
    let dmk = {
        let guard = state.dmk.lock().await;
        let dmk = guard.as_ref().ok_or(AppError::Locked)?;
        *dmk.expose_secret()
    };
    native_vault::set_dmk(&dmk)?;

    let path = state.keystore_path();
    let mut ks = Keystore::load(&path)?;
    ks.os_vault_enabled = true;
    ks.save(&path)?;
    security_status(state).await
}

#[tauri::command]
pub async fn security_disable_os_vault(
    state: State<'_, AppState>,
    confirm: bool,
) -> AppResult<SecurityStatus> {
    if !confirm {
        return Err(AppError::BadRequest(
            "disabling OS vault unlock requires explicit confirmation".into(),
        ));
    }
    native_vault::delete()?;
    let path = state.keystore_path();
    let mut ks = Keystore::load(&path)?;
    ks.os_vault_enabled = false;
    ks.os_vault_auto_unlock = false;
    ks.save(&path)?;
    security_status(state).await
}

#[tauri::command]
pub async fn security_set_auto_unlock(
    state: State<'_, AppState>,
    enabled: bool,
) -> AppResult<SecurityStatus> {
    let path = state.keystore_path();
    let mut ks = Keystore::load(&path)?;
    if enabled && !ks.os_vault_enabled {
        return Err(AppError::BadRequest(
            "enable the OS vault before enabling automatic unlock".into(),
        ));
    }
    ks.os_vault_auto_unlock = enabled;
    ks.save(&path)?;
    security_status(state).await
}

/// Unlock with the current user's native credential store. Password and
/// passkey unlock remain available as fallbacks and are never removed.
#[tauri::command]
pub async fn security_unlock_os_vault(state: State<'_, AppState>) -> AppResult<SecurityStatus> {
    let (_, enabled, _) = keystore_status(&state)?;
    if !enabled {
        return Err(AppError::BadRequest(
            "OS vault unlock is not enabled for this instance".into(),
        ));
    }
    let dmk = native_vault::get_dmk()?;
    crate::commands::auth::activate_dmk(&state, dmk).await?;
    security_status(state).await
}
