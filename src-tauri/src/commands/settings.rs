use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Serialize, Deserialize)]
pub struct SettingEntry {
    pub key: String,
    pub value_json: String,
}

#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>, key: String) -> AppResult<Option<String>> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let value: Option<String> = db
        .conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            [&key],
            |r| r.get::<_, String>(0),
        )
        .optional()?;
    Ok(value)
}

#[tauri::command]
pub async fn settings_set(
    state: State<'_, AppState>,
    key: String,
    value_json: String,
) -> AppResult<()> {
    serde_json::from_str::<serde_json::Value>(&value_json)?;
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    db.conn.execute(
        "INSERT INTO settings(key, value_json) VALUES(?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
        [&key, &value_json],
    )?;
    Ok(())
}

#[tauri::command]
pub async fn settings_get_all(state: State<'_, AppState>) -> AppResult<Vec<SettingEntry>> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let mut stmt = db.conn.prepare("SELECT key, value_json FROM settings")?;
    let rows = stmt
        .query_map([], |r| {
            Ok(SettingEntry {
                key: r.get(0)?,
                value_json: r.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Remove user preferences only. Vault records, ontology edits, backups,
/// models, and authentication material are deliberately unaffected.
#[tauri::command]
pub async fn settings_reset_all(state: State<'_, AppState>) -> AppResult<()> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    db.conn.execute("DELETE FROM settings", [])?;
    Ok(())
}
