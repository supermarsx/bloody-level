use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::crypto::{
    kdf,
    keystore::{generate_dmk, generate_salt, Keystore, Wrapper},
    password_strength, wrap,
};
use crate::db::Database;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Serialize)]
pub struct PasskeyStatus {
    pub label: String,
    pub credential_id_b64: String,
    pub prf_salt_b64: String,
}

#[derive(Serialize)]
pub struct AuthStatus {
    pub initialized: bool,
    pub has_password: bool,
    pub has_passkey: bool,
    pub passkey_count: usize,
    pub passkeys: Vec<PasskeyStatus>,
    pub unlocked: bool,
    pub failed_unlocks: u32,
    pub unlock_backoff_remaining_secs: i64,
    /// True only in debug builds. Frontend uses this to surface a dev "Skip"
    /// button that uses a known dev password — never compiled into release.
    pub is_dev: bool,
    pub os_vault_configured: bool,
    pub os_vault_auto_unlock: bool,
}

const IS_DEV: bool = cfg!(debug_assertions);
const PASSKEY_WRAP_CONTEXT: &[u8] = b"bloody-level DMK wrap v1";
// Passkeys created by pre-rename builds use this context. Keep it as a
// compatibility fallback so a product rename cannot strand an existing vault.
const LEGACY_PASSKEY_WRAP_CONTEXT: &[u8] = b"blevel-tracker DMK wrap v1";

fn derive_passkey_kek(prf: &[u8], context: &[u8]) -> AppResult<secrecy::SecretBox<[u8; 32]>> {
    kdf::derive_kek_from_prf(prf, context)
}

#[tauri::command]
pub async fn auth_status(state: State<'_, AppState>) -> AppResult<AuthStatus> {
    let unlocked = state.db.lock().await.is_some();
    let path = state.keystore_path();
    if !path.exists() {
        return Ok(AuthStatus {
            initialized: false,
            has_password: false,
            has_passkey: false,
            passkey_count: 0,
            passkeys: vec![],
            unlocked,
            failed_unlocks: 0,
            unlock_backoff_remaining_secs: 0,
            is_dev: IS_DEV,
            os_vault_configured: false,
            os_vault_auto_unlock: false,
        });
    }
    let ks = Keystore::load(&path)?;
    let passkeys = passkey_statuses(&ks);
    Ok(AuthStatus {
        initialized: true,
        has_password: ks.has_password(),
        has_passkey: ks.has_passkey(),
        passkey_count: ks.passkeys().len(),
        passkeys,
        unlocked,
        failed_unlocks: ks.failed_unlocks,
        unlock_backoff_remaining_secs: ks.unlock_backoff_remaining_secs(),
        is_dev: IS_DEV,
        os_vault_configured: ks.os_vault_enabled
            && crate::crypto::native_vault::status().credential_present,
        os_vault_auto_unlock: ks.os_vault_auto_unlock,
    })
}

#[tauri::command]
pub async fn auth_unlock_os_vault(state: State<'_, AppState>) -> AppResult<()> {
    crate::commands::security::security_unlock_os_vault(state)
        .await
        .map(|_| ())
}

/// Open the encrypted vault and retain the DMK only in the native process.
/// Keeping this in one path makes password, passkey, and OS-vault unlocks
/// apply the same PDF-cache migration and key lifetime rules.
pub(crate) async fn activate_dmk(
    state: &AppState,
    dmk: secrecy::SecretBox<[u8; 32]>,
) -> AppResult<()> {
    let mut db = Database::open_encrypted(&state.db_path(), &dmk)?;
    db.migrate()?;
    let _ = db.ensure_ontology(state.ontology_seed_path().as_deref());
    let pdf_key = kdf::derive_pdf_cache_key(&dmk)?;
    crate::crypto::file::migrate_plaintext_pdf_cache(&state.pdf_dir(), &pdf_key)?;
    let dmk_copy = secrecy::SecretBox::new(Box::new(*dmk.expose_secret()));
    *state.db.lock().await = Some(db);
    *state.dmk.lock().await = Some(dmk_copy);
    *state.pdf_key.lock().await = Some(pdf_key);
    Ok(())
}

#[tauri::command]
pub async fn auth_setup_password(state: State<'_, AppState>, password: String) -> AppResult<()> {
    let ks_path = state.keystore_path();
    if ks_path.exists() {
        return Err(AppError::AlreadyInitialized);
    }
    password_strength::validate_new_password(&password)?;
    let dmk = generate_dmk();
    let salt = generate_salt(16);
    let kek = kdf::derive_kek_from_password(&password, &salt)?;
    let (nonce, ct) = wrap::wrap_dmk(&kek, dmk.expose_secret())?;
    let mut ks = Keystore::empty();
    ks.replace_password(Wrapper::Password {
        kdf: "argon2id".into(),
        m: kdf::ARGON2_M_KIB,
        t: kdf::ARGON2_T,
        p: kdf::ARGON2_P,
        salt_b64: B64.encode(&salt),
        nonce_b64: B64.encode(&nonce),
        ciphertext_b64: B64.encode(&ct),
    })?;
    ks.save(&ks_path)?;

    activate_dmk(&state, dmk).await?;
    Ok(())
}

#[tauri::command]
pub async fn auth_unlock_password(state: State<'_, AppState>, password: String) -> AppResult<()> {
    let ks_path = state.keystore_path();
    let mut ks = Keystore::load(&ks_path)?;
    ks.enforce_unlock_backoff()?;
    let pw = ks
        .wrappers
        .iter()
        .find_map(|w| match w {
            Wrapper::Password {
                salt_b64,
                nonce_b64,
                ciphertext_b64,
                ..
            } => Some((salt_b64.clone(), nonce_b64.clone(), ciphertext_b64.clone())),
            _ => None,
        })
        .ok_or_else(|| AppError::BadRequest("no password wrapper".into()))?;

    let salt = B64
        .decode(pw.0)
        .map_err(|e| AppError::Base64(e.to_string()))?;
    let nonce = B64
        .decode(pw.1)
        .map_err(|e| AppError::Base64(e.to_string()))?;
    let ct = B64
        .decode(pw.2)
        .map_err(|e| AppError::Base64(e.to_string()))?;

    let kek = kdf::derive_kek_from_password(&password, &salt)?;
    let dmk = match wrap::unwrap_dmk(&kek, &nonce, &ct) {
        Ok(k) => k,
        Err(e) => {
            ks.record_failure();
            let _ = ks.save(&ks_path);
            return Err(e);
        }
    };

    activate_dmk(&state, dmk).await?;

    ks.record_success();
    ks.save(&ks_path)?;
    Ok(())
}

#[derive(Deserialize)]
pub struct ChangePasswordArgs {
    pub current_password: String,
    pub new_password: String,
}

#[tauri::command]
pub async fn auth_change_password(
    state: State<'_, AppState>,
    args: ChangePasswordArgs,
) -> AppResult<()> {
    password_strength::validate_new_password(&args.new_password)?;
    let ks_path = state.keystore_path();
    let mut ks = Keystore::load(&ks_path)?;

    let pw = ks
        .wrappers
        .iter()
        .find_map(|w| match w {
            Wrapper::Password {
                salt_b64,
                nonce_b64,
                ciphertext_b64,
                ..
            } => Some((salt_b64.clone(), nonce_b64.clone(), ciphertext_b64.clone())),
            _ => None,
        })
        .ok_or_else(|| AppError::BadRequest("no password wrapper".into()))?;

    let salt = B64
        .decode(pw.0)
        .map_err(|e| AppError::Base64(e.to_string()))?;
    let nonce = B64
        .decode(pw.1)
        .map_err(|e| AppError::Base64(e.to_string()))?;
    let ct = B64
        .decode(pw.2)
        .map_err(|e| AppError::Base64(e.to_string()))?;
    let cur_kek = kdf::derive_kek_from_password(&args.current_password, &salt)?;
    let dmk = wrap::unwrap_dmk(&cur_kek, &nonce, &ct)?;

    let new_salt = generate_salt(16);
    let new_kek = kdf::derive_kek_from_password(&args.new_password, &new_salt)?;
    let (new_nonce, new_ct) = wrap::wrap_dmk(&new_kek, dmk.expose_secret())?;
    ks.replace_password(Wrapper::Password {
        kdf: "argon2id".into(),
        m: kdf::ARGON2_M_KIB,
        t: kdf::ARGON2_T,
        p: kdf::ARGON2_P,
        salt_b64: B64.encode(&new_salt),
        nonce_b64: B64.encode(&new_nonce),
        ciphertext_b64: B64.encode(&new_ct),
    })?;
    ks.save(&ks_path)?;
    Ok(())
}

#[tauri::command]
pub async fn auth_lock(state: State<'_, AppState>) -> AppResult<()> {
    *state.db.lock().await = None;
    *state.dmk.lock().await = None;
    *state.pdf_key.lock().await = None;
    Ok(())
}

/// Permanently remove this local instance so a user who cannot unlock it can
/// start again. The frontend must obtain an explicit confirmation first, and
/// the command repeats that check so an accidental IPC call cannot wipe data.
#[tauri::command]
pub async fn auth_reset_instance(state: State<'_, AppState>, confirm: bool) -> AppResult<()> {
    if !confirm {
        return Err(AppError::BadRequest(
            "instance reset requires explicit confirmation".into(),
        ));
    }

    {
        let guard = state.db.lock().await;
        if guard.is_some() {
            return Err(AppError::BadRequest(
                "Lock the vault before resetting the instance.".into(),
            ));
        }
    }
    *state.pdf_key.lock().await = None;
    *state.dmk.lock().await = None;
    let _ = crate::crypto::native_vault::delete();

    if state.data_dir.exists() {
        std::fs::remove_dir_all(&state.data_dir)
            .map_err(|e| AppError::Internal(format!("remove instance data: {e}")))?;
    }
    std::fs::create_dir_all(state.pdf_dir())
        .map_err(|e| AppError::Internal(format!("recreate PDF directory: {e}")))?;
    std::fs::create_dir_all(state.models_dir())
        .map_err(|e| AppError::Internal(format!("recreate models directory: {e}")))?;
    Ok(())
}

// PRF output reaches Rust over Tauri IPC as base64 bytes. This is a
// same-process boundary (WebView2 ↔ Tauri core), so the secret never leaves the
// process — equivalent to passing it across a function call. Acceptable.
#[derive(Deserialize)]
pub struct RegisterPasskeyArgs {
    pub label: String,
    pub credential_id_b64: String,
    pub prf_salt_b64: String,
    pub prf_output_b64: String,
    pub current_password: String,
}

#[tauri::command]
pub async fn auth_register_passkey(
    state: State<'_, AppState>,
    args: RegisterPasskeyArgs,
) -> AppResult<()> {
    let ks_path = state.keystore_path();
    let mut ks = Keystore::load(&ks_path)?;

    let pw = ks
        .wrappers
        .iter()
        .find_map(|w| match w {
            Wrapper::Password {
                salt_b64,
                nonce_b64,
                ciphertext_b64,
                ..
            } => Some((salt_b64.clone(), nonce_b64.clone(), ciphertext_b64.clone())),
            _ => None,
        })
        .ok_or_else(|| AppError::BadRequest("password required to register passkey".into()))?;

    let salt = B64
        .decode(pw.0)
        .map_err(|e| AppError::Base64(e.to_string()))?;
    let nonce = B64
        .decode(pw.1)
        .map_err(|e| AppError::Base64(e.to_string()))?;
    let ct = B64
        .decode(pw.2)
        .map_err(|e| AppError::Base64(e.to_string()))?;
    let kek = kdf::derive_kek_from_password(&args.current_password, &salt)?;
    let dmk = wrap::unwrap_dmk(&kek, &nonce, &ct)?;

    let prf = B64
        .decode(&args.prf_output_b64)
        .map_err(|e| AppError::Base64(e.to_string()))?;
    let pk_kek = derive_passkey_kek(&prf, PASSKEY_WRAP_CONTEXT)?;
    let (pk_nonce, pk_ct) = wrap::wrap_dmk(&pk_kek, dmk.expose_secret())?;

    ks.add_passkey(Wrapper::Passkey {
        label: args.label,
        credential_id_b64: args.credential_id_b64,
        prf_salt_b64: args.prf_salt_b64,
        nonce_b64: B64.encode(&pk_nonce),
        ciphertext_b64: B64.encode(&pk_ct),
    })?;
    ks.save(&ks_path)?;
    Ok(())
}

#[derive(Deserialize)]
pub struct UnlockPasskeyArgs {
    pub credential_id_b64: String,
    pub prf_output_b64: String,
}

#[tauri::command]
pub async fn auth_unlock_passkey(
    state: State<'_, AppState>,
    args: UnlockPasskeyArgs,
) -> AppResult<()> {
    let ks_path = state.keystore_path();
    let mut ks = Keystore::load(&ks_path)?;
    ks.enforce_unlock_backoff()?;
    let target = ks
        .wrappers
        .iter()
        .find_map(|w| match w {
            Wrapper::Passkey {
                credential_id_b64,
                nonce_b64,
                ciphertext_b64,
                ..
            } if credential_id_b64 == &args.credential_id_b64 => {
                Some((nonce_b64.clone(), ciphertext_b64.clone()))
            }
            _ => None,
        })
        .ok_or_else(|| {
            ks.record_failure();
            let _ = ks.save(&ks_path);
            AppError::NotFound("passkey not registered".into())
        })?;

    let prf = B64
        .decode(&args.prf_output_b64)
        .map_err(|e| AppError::Base64(e.to_string()))?;
    let nonce = B64
        .decode(target.0)
        .map_err(|e| AppError::Base64(e.to_string()))?;
    let ct = B64
        .decode(target.1)
        .map_err(|e| AppError::Base64(e.to_string()))?;

    let dmk = [PASSKEY_WRAP_CONTEXT, LEGACY_PASSKEY_WRAP_CONTEXT]
        .into_iter()
        .find_map(|context| {
            let kek = derive_passkey_kek(&prf, context).ok()?;
            wrap::unwrap_dmk(&kek, &nonce, &ct).ok()
        })
        .ok_or_else(|| {
            ks.record_failure();
            let _ = ks.save(&ks_path);
            AppError::Crypto("unwrap failed (wrong passkey)".into())
        })?;

    activate_dmk(&state, dmk).await?;

    ks.record_success();
    ks.save(&ks_path)?;
    Ok(())
}

fn passkey_statuses(ks: &Keystore) -> Vec<PasskeyStatus> {
    ks.wrappers
        .iter()
        .filter_map(|w| match w {
            Wrapper::Passkey {
                label,
                credential_id_b64,
                prf_salt_b64,
                ..
            } => Some(PasskeyStatus {
                label: label.clone(),
                credential_id_b64: credential_id_b64.clone(),
                prf_salt_b64: prf_salt_b64.clone(),
            }),
            _ => None,
        })
        .collect()
}
