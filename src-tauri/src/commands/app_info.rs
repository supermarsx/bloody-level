use std::path::Path;
use std::time::SystemTime;

use rusqlite::OptionalExtension;
use serde::Serialize;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Serialize)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub build_profile: &'static str,
    pub target_triple: &'static str,
    pub data_dir: String,
    pub data_dir_size_bytes: Option<u64>,
    pub keystore_path: String,
    pub keystore_size_bytes: Option<u64>,
    pub db_path: String,
    pub db_size_bytes: Option<u64>,
    /// Size of any sidecar files SQLite/SQLCipher creates next to the
    /// primary db (`data.db-wal`, `data.db-shm`, `data.db-journal`).
    /// Nice to know because the WAL can be larger than the DB itself
    /// during heavy ingest sessions.
    pub db_sidecar_bytes: Option<u64>,
    pub pdf_dir: String,
    pub pdf_count: u64,
    pub pdf_size_bytes: Option<u64>,
    pub models_dir: String,
    pub models_count: u64,
    pub models_size_bytes: Option<u64>,
    /// Set when the vault has been unlocked at least once during this
    /// session — drives the Settings UI which can show extra db-stats
    /// only when the connection is up. Stays `None` while locked.
    pub db_stats: Option<DbStats>,
    pub features: Features,
    pub pdfium_available: bool,
    pub pdfium_error: Option<String>,
}

#[derive(Serialize)]
pub struct Features {
    pub embedded_llm: bool,
    pub embedded_ocr_vision: bool,
    pub tesseract_ocr: bool,
    pub debug_assertions: bool,
}

/// Rich summary of what's living inside the encrypted DB. Only fetched
/// when the vault is unlocked — reads single-column counts off small
/// indexed queries so the cost is sub-millisecond on large vaults.
#[derive(Serialize)]
pub struct DbStats {
    pub patient_count: i64,
    pub report_count: i64,
    /// Excludes inline-prior rows so the headline matches the count
    /// users see in the Records / Patients tabs.
    pub result_count: i64,
    pub inline_prior_count: i64,
    pub analyte_count: i64,
    pub alias_count: i64,
    pub audit_count: i64,
    pub earliest_collection_date_iso: Option<String>,
    pub latest_collection_date_iso: Option<String>,
    /// e.g. "wal", "delete" — useful to know whether the db is
    /// running in WAL mode (default for unlocked sessions).
    pub journal_mode: Option<String>,
    pub page_size: Option<i64>,
    pub page_count: Option<i64>,
    pub sqlcipher_version: Option<String>,
}

#[tauri::command]
pub async fn app_info(state: State<'_, AppState>) -> AppResult<AppInfo> {
    let pdfium_check = crate::pdf::pdfium_available();
    let (pdfium_available, pdfium_error) = match pdfium_check {
        Ok(()) => (true, None),
        Err(e) => (false, Some(e)),
    };

    let db_path = state.db_path();
    let db_size_bytes = std::fs::metadata(&db_path).ok().map(|m| m.len());
    let db_sidecar_bytes: u64 = ["-wal", "-shm", "-journal"]
        .iter()
        .filter_map(|suffix| {
            let mut p = db_path.clone();
            p.set_file_name(format!(
                "{}{suffix}",
                db_path.file_name()?.to_string_lossy()
            ));
            std::fs::metadata(p).ok().map(|m| m.len())
        })
        .sum();

    let keystore_path = state.keystore_path();
    let keystore_size_bytes = std::fs::metadata(&keystore_path).ok().map(|m| m.len());

    let pdf_dir = state.pdf_dir();
    let pdf_files: Vec<u64> = std::fs::read_dir(&pdf_dir)
        .map(|it| {
            it.filter_map(Result::ok)
                .filter_map(|e| e.metadata().ok().filter(|m| m.is_file()).map(|m| m.len()))
                .collect()
        })
        .unwrap_or_default();
    let pdf_count = pdf_files.len() as u64;
    let pdf_size_bytes = if pdf_files.is_empty() { None } else { Some(pdf_files.iter().sum()) };

    let models_dir = state.models_dir();
    let models_size_bytes = dir_size(&models_dir);
    let models_count = std::fs::read_dir(&models_dir)
        .map(|it| it.filter_map(Result::ok).filter(|e| e.metadata().map(|m| m.is_file()).unwrap_or(false)).count() as u64)
        .unwrap_or(0);

    let data_dir_size_bytes = dir_size(&state.data_dir);

    // Pull DB-level stats only when the vault is unlocked. We never block
    // the lock screen or fail this command if the connection isn't ready.
    let db_stats = collect_db_stats(&state).await;

    Ok(AppInfo {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        build_profile: if cfg!(debug_assertions) { "debug" } else { "release" },
        target_triple: env!("TARGET_TRIPLE"),
        data_dir: state.data_dir.to_string_lossy().into(),
        data_dir_size_bytes,
        keystore_path: keystore_path.to_string_lossy().into(),
        keystore_size_bytes,
        db_path: db_path.to_string_lossy().into(),
        db_size_bytes,
        db_sidecar_bytes: Some(db_sidecar_bytes),
        pdf_dir: pdf_dir.to_string_lossy().into(),
        pdf_count,
        pdf_size_bytes,
        models_dir: models_dir.to_string_lossy().into(),
        models_count,
        models_size_bytes,
        db_stats,
        features: Features {
            embedded_llm: cfg!(feature = "embedded-llm"),
            embedded_ocr_vision: cfg!(feature = "embedded-ocr-vision"),
            tesseract_ocr: cfg!(feature = "tesseract-ocr"),
            debug_assertions: cfg!(debug_assertions),
        },
        pdfium_available,
        pdfium_error,
    })
}

/// Recursive directory size in bytes. Returns `None` if the path doesn't
/// exist (vs. `Some(0)` for an empty directory). Best-effort: silently
/// skips entries that can't be stat'd.
fn dir_size(path: &Path) -> Option<u64> {
    if !path.exists() { return None; }
    fn walk(p: &Path) -> u64 {
        let mut total = 0u64;
        if let Ok(rd) = std::fs::read_dir(p) {
            for entry in rd.flatten() {
                if let Ok(md) = entry.metadata() {
                    if md.is_dir() {
                        total += walk(&entry.path());
                    } else if md.is_file() {
                        total += md.len();
                    }
                }
            }
        }
        total
    }
    Some(walk(path))
}

async fn collect_db_stats(state: &State<'_, AppState>) -> Option<DbStats> {
    let guard = state.db.lock().await;
    let db = guard.as_ref()?;
    let conn = &db.conn;

    let q_i64 = |sql: &str| -> Option<i64> {
        conn.query_row(sql, [], |r| r.get::<_, i64>(0)).optional().ok().flatten()
    };
    let q_str = |sql: &str| -> Option<String> {
        conn.query_row(sql, [], |r| r.get::<_, String>(0)).optional().ok().flatten()
    };
    Some(DbStats {
        patient_count:        q_i64("SELECT COUNT(*) FROM patients").unwrap_or(0),
        report_count:         q_i64("SELECT COUNT(*) FROM reports").unwrap_or(0),
        result_count:         q_i64("SELECT COUNT(*) FROM results WHERE inline_prior_pdf = 0").unwrap_or(0),
        inline_prior_count:   q_i64("SELECT COUNT(*) FROM results WHERE inline_prior_pdf = 1").unwrap_or(0),
        analyte_count:        q_i64("SELECT COUNT(*) FROM analytes").unwrap_or(0),
        alias_count:          q_i64("SELECT COUNT(*) FROM analyte_aliases").unwrap_or(0),
        audit_count:          q_i64("SELECT COUNT(*) FROM audit_log").unwrap_or(0),
        earliest_collection_date_iso: q_str("SELECT MIN(collection_date_iso) FROM reports"),
        latest_collection_date_iso:   q_str("SELECT MAX(collection_date_iso) FROM reports"),
        journal_mode:         q_str("PRAGMA journal_mode"),
        page_size:            q_i64("PRAGMA page_size"),
        page_count:           q_i64("PRAGMA page_count"),
        sqlcipher_version:    q_str("PRAGMA cipher_version"),
    })
}

// ─── Export / import vault ────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ExportResult {
    pub destination: String,
    pub bytes_copied: u64,
    pub files_copied: u64,
    pub manifest_path: String,
}

/// Copy every file under the vault data dir to a user-chosen destination
/// directory, alongside a small JSON manifest for verification on import.
/// The DB itself is encrypted at rest — copying the file out preserves
/// that encryption — so this is safe to keep on a USB stick or sync into
/// an external backup tool. The keystore travels with the DB so the
/// receiving device can unlock it with the original password.
#[tauri::command]
pub async fn export_vault(
    state: State<'_, AppState>,
    destination: String,
) -> AppResult<ExportResult> {
    let dest = Path::new(&destination);
    if !dest.exists() {
        std::fs::create_dir_all(dest)
            .map_err(|e| AppError::Internal(format!("create dest dir: {e}")))?;
    }
    if !dest.is_dir() {
        return Err(AppError::BadRequest(format!(
            "destination must be a directory: {}",
            dest.display()
        )));
    }
    // Refuse to write into the vault itself — would silently shred the
    // db when the recursive copy hits its own destination.
    if dest.canonicalize().ok() == state.data_dir.canonicalize().ok() {
        return Err(AppError::BadRequest(
            "destination cannot be the vault itself".into(),
        ));
    }

    let mut bytes_copied = 0u64;
    let mut files_copied = 0u64;
    copy_dir_recursive(&state.data_dir, dest, &mut bytes_copied, &mut files_copied)
        .map_err(|e| AppError::Internal(format!("copy: {e}")))?;

    // Write a small manifest so a human can confirm the contents at a
    // glance and a future "import" check can validate compatibility.
    let manifest_path = dest.join("vault-manifest.json");
    let manifest = serde_json::json!({
        "app":             env!("CARGO_PKG_NAME"),
        "version":         env!("CARGO_PKG_VERSION"),
        "exported_at":     unix_timestamp(),
        "files_copied":    files_copied,
        "bytes_copied":    bytes_copied,
        "source_data_dir": state.data_dir.to_string_lossy(),
    });
    std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest).unwrap_or_default())
        .map_err(|e| AppError::Internal(format!("write manifest: {e}")))?;

    // Audit-trail the export. Best-effort — does not abort if the audit
    // table can't be written (e.g. just-locked vault).
    {
        let guard = state.db.lock().await;
        if let Some(db) = guard.as_ref() {
            crate::commands::audit::log(
                &db.conn,
                "export",
                "system",
                None,
                &format!(
                    "Exported vault to {} ({files_copied} files, {bytes_copied} bytes)",
                    dest.display()
                ),
                Some(&serde_json::json!({
                    "destination": dest.to_string_lossy(),
                    "bytes":       bytes_copied,
                    "files":       files_copied,
                })),
            );
        }
    }

    Ok(ExportResult {
        destination: dest.to_string_lossy().into(),
        bytes_copied,
        files_copied,
        manifest_path: manifest_path.to_string_lossy().into(),
    })
}

#[derive(Serialize)]
pub struct ImportResult {
    pub source: String,
    pub bytes_copied: u64,
    pub files_copied: u64,
    pub backup_dir: String,
}

/// Replace the current vault contents with the exported snapshot at
/// `source`. The current vault is renamed alongside as
/// `data_dir.backup-<timestamp>` rather than deleted, so the user can
/// roll back manually. Refuses to run while the DB connection is open
/// (i.e. the vault is unlocked) — the user must lock first; the file
/// handle would otherwise pin the old DB and corrupt the swap on
/// Windows.
#[tauri::command]
pub async fn import_vault(
    state: State<'_, AppState>,
    source: String,
) -> AppResult<ImportResult> {
    {
        let guard = state.db.lock().await;
        if guard.is_some() {
            return Err(AppError::BadRequest(
                "Lock the vault first — cannot replace files while the DB is open.".into(),
            ));
        }
    }

    let src = Path::new(&source);
    if !src.is_dir() {
        return Err(AppError::BadRequest(format!(
            "source must be a directory: {}",
            src.display()
        )));
    }
    // Sanity check — at minimum the source must contain a keystore + db.
    if !src.join("keystore.json").exists() || !src.join("data.db").exists() {
        return Err(AppError::BadRequest(
            "source directory missing keystore.json or data.db — is this a vault export?".into(),
        ));
    }

    // Move the current data dir aside (atomic rename) before laying the
    // new files down. We DON'T delete it — the user may want to roll back.
    let backup_dir = state
        .data_dir
        .parent()
        .map(|p| p.join(format!(
            "{}.backup-{}",
            state.data_dir.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("data"),
            unix_timestamp()
        )))
        .ok_or_else(|| AppError::Internal("data_dir has no parent".into()))?;
    if state.data_dir.exists() {
        std::fs::rename(&state.data_dir, &backup_dir)
            .map_err(|e| AppError::Internal(format!("backup current vault: {e}")))?;
    }
    std::fs::create_dir_all(&state.data_dir)
        .map_err(|e| AppError::Internal(format!("create new vault dir: {e}")))?;

    let mut bytes_copied = 0u64;
    let mut files_copied = 0u64;
    copy_dir_recursive(src, &state.data_dir, &mut bytes_copied, &mut files_copied)
        .map_err(|e| AppError::Internal(format!("copy: {e}")))?;

    Ok(ImportResult {
        source: src.to_string_lossy().into(),
        bytes_copied,
        files_copied,
        backup_dir: backup_dir.to_string_lossy().into(),
    })
}

fn copy_dir_recursive(
    src: &Path,
    dst: &Path,
    bytes: &mut u64,
    files: &mut u64,
) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let entry_src = entry.path();
        let entry_dst = dst.join(entry.file_name());
        let md = entry.metadata()?;
        if md.is_dir() {
            copy_dir_recursive(&entry_src, &entry_dst, bytes, files)?;
        } else if md.is_file() {
            std::fs::copy(&entry_src, &entry_dst)?;
            *bytes += md.len();
            *files += 1;
        }
    }
    Ok(())
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
