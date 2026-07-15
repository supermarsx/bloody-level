use serde::Serialize;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Serialize)]
pub struct TierStatus {
    pub feature: &'static str,
    pub compiled: bool,
    pub enabled_in_settings: bool,
    pub model_present: bool,
    pub loaded: bool,
}

async fn read_setting(state: &AppState, key: &str) -> AppResult<serde_json::Value> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let raw: Option<String> = db
        .conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            [key],
            |r| r.get::<_, String>(0),
        )
        .ok();
    match raw {
        Some(s) => Ok(serde_json::from_str(&s)?),
        None => Ok(serde_json::json!({})),
    }
}

#[tauri::command]
pub async fn tier_status_tesseract(state: State<'_, AppState>) -> AppResult<TierStatus> {
    let s = read_setting(&state, "tesseract").await?;
    Ok(TierStatus {
        feature: "tesseract-ocr",
        compiled: crate::ocr::is_available(),
        enabled_in_settings: s
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        model_present: crate::ocr::is_available(),
        loaded: crate::ocr::is_available(),
    })
}

#[tauri::command]
pub async fn tier_status_llm(state: State<'_, AppState>) -> AppResult<TierStatus> {
    let s = read_setting(&state, "llm").await?;
    let model_path = s.get("model_path").and_then(|v| v.as_str()).unwrap_or("");
    Ok(TierStatus {
        feature: "embedded-llm",
        compiled: crate::llm::is_available(),
        enabled_in_settings: s
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        model_present: !model_path.is_empty() && std::path::Path::new(model_path).exists(),
        loaded: crate::llm::is_loaded(),
    })
}

#[derive(Serialize)]
pub struct PdfiumStatus {
    pub available: bool,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn tier_status_pdfium() -> AppResult<PdfiumStatus> {
    // Cheap: just attempts to bind the library; doesn't open any document.
    match crate::pdf::pdfium_available() {
        Ok(()) => Ok(PdfiumStatus { available: true, error: None }),
        Err(e) => Ok(PdfiumStatus { available: false, error: Some(e) }),
    }
}

#[tauri::command]
pub async fn tier_status_olmocr(state: State<'_, AppState>) -> AppResult<TierStatus> {
    let s = read_setting(&state, "olmocr").await?;
    let model_path = s.get("model_path").and_then(|v| v.as_str()).unwrap_or("");
    Ok(TierStatus {
        feature: "embedded-ocr-vision",
        compiled: crate::ocr_vision::is_available(),
        enabled_in_settings: s
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        model_present: !model_path.is_empty() && std::path::Path::new(model_path).exists(),
        loaded: false,
    })
}
