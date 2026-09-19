use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
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
    pub os_vault_configured: bool,
    pub os_vault_auto_unlock: bool,
    pub os_vault_supported: bool,
    pub os_vault_platform: String,
}

const PASSKEY_WRAP_CONTEXT: &[u8] = b"bloody-level DMK wrap v1";
// Passkeys created by pre-rename builds use this context. Keep it as a
// compatibility fallback so a product rename cannot strand an existing vault.
const LEGACY_PASSKEY_WRAP_CONTEXT: &[u8] = b"blevel-tracker DMK wrap v1";
const DMK_TRANSITION_CONTEXT: &[u8] = b"bloody-level DMK transition v1";

fn derive_passkey_kek(prf: &[u8], context: &[u8]) -> AppResult<secrecy::SecretBox<[u8; 32]>> {
    kdf::derive_kek_from_prf(prf, context)
}

fn derive_dmk_transition_kek(
    dmk: &secrecy::SecretBox<[u8; 32]>,
) -> AppResult<secrecy::SecretBox<[u8; 32]>> {
    kdf::derive_kek_from_prf(dmk.expose_secret(), DMK_TRANSITION_CONTEXT)
}

/// Rename legacy hash-named managed PDFs to opaque, salted names and keep the
/// encrypted report metadata pointing at the new paths. The source hash stays
/// in the encrypted database for duplicate detection, never in the filename.
fn migrate_pdf_cache_filenames(
    db: &mut Database,
    dir: &Path,
    key: &secrecy::SecretBox<[u8; 32]>,
) -> AppResult<()> {
    if !dir.exists() {
        return Ok(());
    }

    let records: Vec<(String, String, String)> = {
        let mut statement = db.conn.prepare(
            "SELECT id, raw_pdf_path, source_sha256
             FROM reports
             WHERE raw_pdf_path IS NOT NULL AND raw_pdf_path <> ''",
        )?;
        let rows = statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<Result<_, _>>()?;
        rows
    };

    let mut moved: Vec<(PathBuf, PathBuf)> = Vec::new();
    let result = (|| {
        let tx = db.conn.transaction()?;
        for (report_id, raw_pdf_path, source_sha256) in records {
            let old_path = PathBuf::from(raw_pdf_path);
            if !old_path.is_file()
                || !old_path.starts_with(dir)
                || crate::crypto::file::is_salted_cache_path(&old_path)
            {
                continue;
            }

            let new_path = dir.join(crate::crypto::file::salted_cache_filename(
                &source_sha256,
                key,
            ));
            std::fs::rename(&old_path, &new_path)?;
            tx.execute(
                "UPDATE reports SET raw_pdf_path = ?1 WHERE id = ?2",
                rusqlite::params![new_path.to_string_lossy().to_string(), report_id],
            )?;
            moved.push((old_path, new_path));
        }
        tx.commit()?;
        Ok::<(), AppError>(())
    })();

    if result.is_err() {
        for (old_path, new_path) in moved.into_iter().rev() {
            let _ = std::fs::rename(new_path, old_path);
        }
    }
    result
}

/// Resolve password-authenticated DMK rotations that happened while the
/// session was already unlocked. The password wrapper intentionally remains
/// unchanged in that case, and these encrypted links bring it forward to the
/// current DMK without requiring the password to be retained in memory.
fn resolve_dmk_transitions(
    ks: &Keystore,
    mut dmk: secrecy::SecretBox<[u8; 32]>,
) -> AppResult<secrecy::SecretBox<[u8; 32]>> {
    for wrapper in &ks.wrappers {
        let Wrapper::DmkTransition {
            nonce_b64,
            ciphertext_b64,
        } = wrapper
        else {
            continue;
        };
        let nonce = B64
            .decode(nonce_b64)
            .map_err(|error| AppError::Base64(error.to_string()))?;
        let ciphertext = B64
            .decode(ciphertext_b64)
            .map_err(|error| AppError::Base64(error.to_string()))?;
        let kek = derive_dmk_transition_kek(&dmk)?;
        dmk = wrap::unwrap_dmk(&kek, &nonce, &ciphertext)?;
    }
    Ok(dmk)
}

#[tauri::command]
pub async fn auth_status(state: State<'_, AppState>) -> AppResult<AuthStatus> {
    let unlocked = state.db.lock().await.is_some();
    let native = crate::crypto::native_vault::status();
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
            os_vault_configured: false,
            os_vault_auto_unlock: false,
            os_vault_supported: native.supported,
            os_vault_platform: native.platform.to_string(),
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
        os_vault_configured: ks.os_vault_enabled
            && crate::crypto::native_vault::status().credential_present,
        os_vault_auto_unlock: ks.os_vault_auto_unlock,
        os_vault_supported: native.supported,
        os_vault_platform: native.platform.to_string(),
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
    migrate_pdf_cache_filenames(&mut db, &state.pdf_dir(), &pdf_key)?;
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
    let native = crate::crypto::native_vault::status();
    if native.supported
        && crate::crypto::native_vault::set_dmk(
            state
                .dmk
                .lock()
                .await
                .as_ref()
                .ok_or(AppError::Locked)?
                .expose_secret(),
        )
        .is_ok()
    {
        let mut ks = Keystore::load(&ks_path)?;
        ks.os_vault_enabled = true;
        ks.os_vault_auto_unlock = true;
        ks.save(&ks_path)?;
    }
    Ok(())
}

/// Add the first password wrapper to an already-unlocked vault that was
/// initialized with the native OS vault only.
#[tauri::command]
pub async fn auth_set_password(state: State<'_, AppState>, password: String) -> AppResult<()> {
    password_strength::validate_new_password(&password)?;
    let ks_path = state.keystore_path();
    let mut ks = Keystore::load(&ks_path)?;
    if ks.has_password() {
        return Err(AppError::AlreadyInitialized);
    }
    let dmk = {
        let guard = state.dmk.lock().await;
        let current = guard.as_ref().ok_or(AppError::Locked)?;
        *current.expose_secret()
    };
    let salt = generate_salt(16);
    let kek = kdf::derive_kek_from_password(&password, &salt)?;
    let (nonce, ciphertext) = wrap::wrap_dmk(&kek, &dmk)?;
    ks.replace_password(Wrapper::Password {
        kdf: "argon2id".into(),
        m: kdf::ARGON2_M_KIB,
        t: kdf::ARGON2_T,
        p: kdf::ARGON2_P,
        salt_b64: B64.encode(&salt),
        nonce_b64: B64.encode(&nonce),
        ciphertext_b64: B64.encode(&ciphertext),
    })?;
    ks.save(&ks_path)
}

/// Initialize a vault with a native OS-vault wrapper and no password wrapper.
/// The OS account credential is the recovery boundary; users can add one or
/// more passkeys from Security after the vault is open.
#[tauri::command]
pub async fn auth_setup_os_vault(state: State<'_, AppState>) -> AppResult<()> {
    let ks_path = state.keystore_path();
    if ks_path.exists() {
        return Err(AppError::AlreadyInitialized);
    }
    let native = crate::crypto::native_vault::status();
    if !native.supported {
        return Err(AppError::BadRequest(
            "this operating system does not provide a supported native vault".into(),
        ));
    }

    let dmk = generate_dmk();
    crate::crypto::native_vault::set_dmk(dmk.expose_secret())?;
    let mut ks = Keystore::empty();
    ks.os_vault_enabled = true;
    ks.os_vault_auto_unlock = true;
    if let Err(error) = ks.save(&ks_path) {
        let _ = crate::crypto::native_vault::delete();
        return Err(error);
    }
    if let Err(error) = activate_dmk(&state, dmk).await {
        let _ = crate::crypto::native_vault::delete();
        let _ = std::fs::remove_file(&ks_path);
        return Err(error);
    }
    Ok(())
}

#[tauri::command]
pub async fn auth_unlock_password(state: State<'_, AppState>, password: String) -> AppResult<()> {
    let ks_path = state.keystore_path();
    let mut ks = Keystore::load(&ks_path)?;
    ks.enforce_unlock_backoff()?;

    // A passwordless vault still has an encrypted database: its DMK is held
    // by the configured native OS vault. Accepting an empty password here
    // gives the unlock screen a predictable fallback when automatic OS-vault
    // unlock is disabled or unavailable at launch. Never treat an empty
    // string as a password for a password-protected vault.
    if !ks.has_password() {
        if !password.is_empty() {
            return Err(AppError::BadRequest(
                "this vault has no password; leave the password field empty or use a passkey"
                    .into(),
            ));
        }
        if !(ks.os_vault_enabled && crate::crypto::native_vault::status().credential_present) {
            return Err(AppError::BadRequest(
                "no vault password is configured; use the native OS vault or a passkey to unlock"
                    .into(),
            ));
        }
        let dmk = crate::crypto::native_vault::get_dmk()?;
        activate_dmk(&state, dmk).await?;
        ks.record_success();
        ks.save(&ks_path)?;
        return Ok(());
    }

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
    let dmk = resolve_dmk_transitions(&ks, dmk)?;

    activate_dmk(&state, dmk).await?;

    ks.record_success();
    ks.save(&ks_path)?;
    Ok(())
}

#[derive(Deserialize)]
pub struct ChangePasswordArgs {
    pub current_password: Option<String>,
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
    let cur_kek =
        kdf::derive_kek_from_password(args.current_password.as_deref().unwrap_or_default(), &salt)?;
    let dmk = wrap::unwrap_dmk(&cur_kek, &nonce, &ct)?;
    let dmk = resolve_dmk_transitions(&ks, dmk)?;

    let new_salt = generate_salt(16);
    let new_kek = kdf::derive_kek_from_password(&args.new_password, &new_salt)?;
    let (new_nonce, new_ct) = wrap::wrap_dmk(&new_kek, dmk.expose_secret())?;
    ks.wrappers
        .retain(|wrapper| !matches!(wrapper, Wrapper::DmkTransition { .. }));
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

/// Remove the password wrapper from an unlocked vault. A passkey or a
/// configured native OS-vault credential must remain so the vault is never
/// left without a recovery route.
#[tauri::command]
pub async fn auth_remove_password(state: State<'_, AppState>) -> AppResult<()> {
    let ks_path = state.keystore_path();
    let mut ks = Keystore::load(&ks_path)?;
    if !ks.has_password() {
        return Err(AppError::BadRequest(
            "no vault password is configured".into(),
        ));
    }
    let has_os_vault =
        ks.os_vault_enabled && crate::crypto::native_vault::status().credential_present;
    if !ks.has_passkey() && !has_os_vault {
        return Err(AppError::BadRequest(
            "add a passkey or enable the native OS vault before removing the last password recovery method".into(),
        ));
    }
    ks.wrappers.retain(|wrapper| {
        !matches!(
            wrapper,
            Wrapper::Password { .. } | Wrapper::DmkTransition { .. }
        )
    });
    ks.save(&ks_path)
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
    pub current_password: Option<String>,
}

#[tauri::command]
pub async fn auth_register_passkey(
    state: State<'_, AppState>,
    args: RegisterPasskeyArgs,
) -> AppResult<()> {
    let ks_path = state.keystore_path();
    let mut ks = Keystore::load(&ks_path)?;

    let dmk = if args
        .current_password
        .as_deref()
        .is_some_and(|password| !password.is_empty())
    {
        let password = args.current_password.as_deref().unwrap_or_default();
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
        let kek = kdf::derive_kek_from_password(password, &salt)?;
        wrap::unwrap_dmk(&kek, &nonce, &ct)?
    } else {
        let guard = state.dmk.lock().await;
        let dmk = guard.as_ref().ok_or(AppError::Locked)?;
        secrecy::SecretBox::new(Box::new(*dmk.expose_secret()))
    };

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
pub struct RemovePasskeyArgs {
    pub credential_id_b64: String,
}

#[tauri::command]
pub async fn auth_remove_passkey(
    state: State<'_, AppState>,
    args: RemovePasskeyArgs,
) -> AppResult<()> {
    if state.dmk.lock().await.is_none() {
        return Err(AppError::Locked);
    }
    let path = state.keystore_path();
    let mut ks = Keystore::load(&path)?;
    let before = ks.passkeys().len();
    if before == 0 {
        return Err(AppError::NotFound("passkey not registered".into()));
    }
    if !ks.has_password()
        && before == 1
        && !(ks.os_vault_enabled && crate::crypto::native_vault::status().credential_present)
    {
        return Err(AppError::BadRequest(
            "keep a password or native OS-vault recovery method before removing the last passkey"
                .into(),
        ));
    }
    ks.wrappers.retain(|wrapper| {
        !matches!(
            wrapper,
            Wrapper::Passkey {
                credential_id_b64,
                ..
            } if credential_id_b64 == &args.credential_id_b64
        )
    });
    if ks.passkeys().len() == before {
        return Err(AppError::NotFound("passkey not registered".into()));
    }
    ks.save(&path)
}

#[derive(Deserialize)]
pub struct PasskeyRewrap {
    pub credential_id_b64: String,
    pub prf_output_b64: String,
}

#[derive(Deserialize)]
pub struct RotateMasterKeyArgs {
    pub current_password: Option<String>,
    #[serde(default)]
    pub passkeys: Vec<PasskeyRewrap>,
}

/// Replace the data master key while preserving every configured unlock
/// method. Passkey PRF assertions are supplied by the WebAuthn frontend so
/// their wrappers can be re-encrypted without exposing the PRF to disk.
#[tauri::command]
pub async fn auth_rotate_master_key(
    state: State<'_, AppState>,
    args: RotateMasterKeyArgs,
) -> AppResult<()> {
    let old_dmk = {
        let guard = state.dmk.lock().await;
        let dmk = guard.as_ref().ok_or(AppError::Locked)?;
        secrecy::SecretBox::new(Box::new(*dmk.expose_secret()))
    };
    let path = state.keystore_path();
    let ks = Keystore::load(&path)?;
    let password = args
        .current_password
        .as_deref()
        .filter(|value| !value.is_empty());
    let new_dmk = generate_dmk();
    let mut wrappers = Vec::with_capacity(ks.wrappers.len());
    let mut retained_password_wrapper = false;
    let mut rekeyed_password_wrapper = false;

    for wrapper in &ks.wrappers {
        match wrapper {
            Wrapper::Password { .. } => {
                if let Some(password) = password {
                    let current = ks
                        .wrappers
                        .iter()
                        .find_map(|candidate| match candidate {
                            Wrapper::Password {
                                salt_b64,
                                nonce_b64,
                                ciphertext_b64,
                                ..
                            } => Some((salt_b64, nonce_b64, ciphertext_b64)),
                            _ => None,
                        })
                        .ok_or_else(|| AppError::BadRequest("no password wrapper".into()))?;
                    let salt = B64
                        .decode(current.0)
                        .map_err(|error| AppError::Base64(error.to_string()))?;
                    let nonce = B64
                        .decode(current.1)
                        .map_err(|error| AppError::Base64(error.to_string()))?;
                    let ciphertext = B64
                        .decode(current.2)
                        .map_err(|error| AppError::Base64(error.to_string()))?;
                    let kek = kdf::derive_kek_from_password(password, &salt)?;
                    let verified =
                        resolve_dmk_transitions(&ks, wrap::unwrap_dmk(&kek, &nonce, &ciphertext)?)?;
                    if verified.expose_secret() != old_dmk.expose_secret() {
                        return Err(AppError::Crypto("current password is incorrect".into()));
                    }
                    let new_salt = generate_salt(16);
                    let new_kek = kdf::derive_kek_from_password(password, &new_salt)?;
                    let (new_nonce, new_ciphertext) =
                        wrap::wrap_dmk(&new_kek, new_dmk.expose_secret())?;
                    wrappers.push(Wrapper::Password {
                        kdf: "argon2id".into(),
                        m: kdf::ARGON2_M_KIB,
                        t: kdf::ARGON2_T,
                        p: kdf::ARGON2_P,
                        salt_b64: B64.encode(new_salt),
                        nonce_b64: B64.encode(new_nonce),
                        ciphertext_b64: B64.encode(new_ciphertext),
                    });
                    rekeyed_password_wrapper = true;
                } else {
                    wrappers.push(wrapper.clone());
                    retained_password_wrapper = true;
                }
            }
            Wrapper::Passkey {
                label,
                credential_id_b64,
                prf_salt_b64,
                nonce_b64,
                ciphertext_b64,
            } => {
                let rewrap = args
                    .passkeys
                    .iter()
                    .find(|candidate| candidate.credential_id_b64 == *credential_id_b64)
                    .ok_or_else(|| {
                        AppError::BadRequest(format!(
                            "authenticate passkey '{}' before rotating the master key",
                            label
                        ))
                    })?;
                let prf = B64
                    .decode(&rewrap.prf_output_b64)
                    .map_err(|error| AppError::Base64(error.to_string()))?;
                let nonce = B64
                    .decode(nonce_b64)
                    .map_err(|error| AppError::Base64(error.to_string()))?;
                let ciphertext = B64
                    .decode(ciphertext_b64)
                    .map_err(|error| AppError::Base64(error.to_string()))?;
                let old_passkey_dmk = [PASSKEY_WRAP_CONTEXT, LEGACY_PASSKEY_WRAP_CONTEXT]
                    .into_iter()
                    .find_map(|context| {
                        let kek = derive_passkey_kek(&prf, context).ok()?;
                        wrap::unwrap_dmk(&kek, &nonce, &ciphertext).ok()
                    })
                    .ok_or_else(|| {
                        AppError::Crypto(format!("passkey '{label}' assertion failed"))
                    })?;
                if old_passkey_dmk.expose_secret() != old_dmk.expose_secret() {
                    return Err(AppError::Crypto(format!(
                        "passkey '{label}' assertion failed"
                    )));
                }
                let pk_kek = derive_passkey_kek(&prf, PASSKEY_WRAP_CONTEXT)?;
                let (new_nonce, new_ciphertext) = wrap::wrap_dmk(&pk_kek, new_dmk.expose_secret())?;
                wrappers.push(Wrapper::Passkey {
                    label: label.clone(),
                    credential_id_b64: credential_id_b64.clone(),
                    prf_salt_b64: prf_salt_b64.clone(),
                    nonce_b64: B64.encode(new_nonce),
                    ciphertext_b64: B64.encode(new_ciphertext),
                });
            }
            Wrapper::DmkTransition { .. } => {}
        }
    }

    if retained_password_wrapper && !rekeyed_password_wrapper {
        let transition_kek = derive_dmk_transition_kek(&old_dmk)?;
        let (nonce, ciphertext) = wrap::wrap_dmk(&transition_kek, new_dmk.expose_secret())?;
        wrappers.push(Wrapper::DmkTransition {
            nonce_b64: B64.encode(nonce),
            ciphertext_b64: B64.encode(ciphertext),
        });
    }

    let old_pdf_key = kdf::derive_pdf_cache_key(&old_dmk)?;
    let new_pdf_key = kdf::derive_pdf_cache_key(&new_dmk)?;
    crate::crypto::file::reencrypt_cache(&state.pdf_dir(), &old_pdf_key, &new_pdf_key)?;

    if ks.os_vault_enabled {
        if let Err(error) = crate::crypto::native_vault::set_dmk(new_dmk.expose_secret()) {
            let _ =
                crate::crypto::file::reencrypt_cache(&state.pdf_dir(), &new_pdf_key, &old_pdf_key);
            return Err(error);
        }
    }

    let rekey_result = {
        let guard = state.db.lock().await;
        let db = guard.as_ref().ok_or(AppError::Locked)?;
        db.rekey(&new_dmk)
    };
    if let Err(error) = rekey_result {
        let _ = crate::crypto::file::reencrypt_cache(&state.pdf_dir(), &new_pdf_key, &old_pdf_key);
        if ks.os_vault_enabled {
            let _ = crate::crypto::native_vault::set_dmk(old_dmk.expose_secret());
        }
        return Err(error);
    }

    let mut new_ks = ks;
    new_ks.wrappers = wrappers;
    if let Err(error) = new_ks.save(&path) {
        let rollback_db = state.db.lock().await;
        if let Some(db) = rollback_db.as_ref() {
            let _ = db.rekey(&old_dmk);
        }
        let _ = crate::crypto::file::reencrypt_cache(&state.pdf_dir(), &new_pdf_key, &old_pdf_key);
        if new_ks.os_vault_enabled {
            let _ = crate::crypto::native_vault::set_dmk(old_dmk.expose_secret());
        }
        return Err(error);
    }

    *state.dmk.lock().await = Some(new_dmk);
    *state.pdf_key.lock().await = Some(new_pdf_key);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_chained_dmk_transitions() {
        let first = generate_dmk();
        let second = generate_dmk();
        let third = generate_dmk();
        let first_kek = derive_dmk_transition_kek(&first).expect("first transition key");
        let (first_nonce, first_ciphertext) =
            wrap::wrap_dmk(&first_kek, second.expose_secret()).expect("first transition");
        let second_kek = derive_dmk_transition_kek(&second).expect("second transition key");
        let (second_nonce, second_ciphertext) =
            wrap::wrap_dmk(&second_kek, third.expose_secret()).expect("second transition");
        let mut ks = Keystore::empty();
        ks.wrappers.push(Wrapper::DmkTransition {
            nonce_b64: B64.encode(first_nonce),
            ciphertext_b64: B64.encode(first_ciphertext),
        });
        ks.wrappers.push(Wrapper::DmkTransition {
            nonce_b64: B64.encode(second_nonce),
            ciphertext_b64: B64.encode(second_ciphertext),
        });

        let resolved = resolve_dmk_transitions(&ks, first).expect("resolve transitions");
        assert_eq!(resolved.expose_secret(), third.expose_secret());
    }
}
