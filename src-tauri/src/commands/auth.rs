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
}

const IS_DEV: bool = cfg!(debug_assertions);

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
    })
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

    let mut db = Database::open_encrypted(&state.db_path(), &dmk)?;
    db.migrate()?;
    let _ = db.ensure_ontology(state.ontology_seed_path().as_deref());
    *state.db.lock().await = Some(db);
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

    let mut db = Database::open_encrypted(&state.db_path(), &dmk)?;
    db.migrate()?;
    let _ = db.ensure_ontology(state.ontology_seed_path().as_deref());
    *state.db.lock().await = Some(db);

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
    let pk_kek = kdf::derive_kek_from_prf(&prf, b"blevel-tracker DMK wrap v1")?;
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
    let kek = kdf::derive_kek_from_prf(&prf, b"blevel-tracker DMK wrap v1")?;
    let nonce = B64
        .decode(target.0)
        .map_err(|e| AppError::Base64(e.to_string()))?;
    let ct = B64
        .decode(target.1)
        .map_err(|e| AppError::Base64(e.to_string()))?;

    let dmk = match wrap::unwrap_dmk(&kek, &nonce, &ct) {
        Ok(k) => k,
        Err(e) => {
            ks.record_failure();
            let _ = ks.save(&ks_path);
            return Err(e);
        }
    };

    let mut db = Database::open_encrypted(&state.db_path(), &dmk)?;
    db.migrate()?;
    let _ = db.ensure_ontology(state.ontology_seed_path().as_deref());
    *state.db.lock().await = Some(db);

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
