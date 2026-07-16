use serde::Serialize;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Serialize)]
pub struct ModelLoadStatus {
    pub configured_model_path: Option<String>,
    pub configured_model_present: bool,
    #[serde(flatten)]
    pub tier: TierStatus,
}

#[derive(Serialize)]
pub struct TierStatus {
    pub feature: &'static str,
    pub compiled: bool,
    pub enabled_in_settings: bool,
    pub model_present: bool,
    pub loading: bool,
    pub loaded: bool,
    pub loaded_model_path: Option<String>,
    pub last_error: Option<String>,
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
    let config = crate::ocr::config_from_settings(&s);
    let runtime = crate::ocr::runtime_status(&config);
    Ok(TierStatus {
        feature: "tesseract-ocr",
        compiled: runtime.compiled,
        enabled_in_settings: config.enabled,
        model_present: runtime.model_present,
        loading: false,
        loaded: runtime.loaded,
        loaded_model_path: config.datapath,
        last_error: runtime.error,
    })
}

fn llm_tier_status(runtime: crate::llm::ModelStatus, enabled_in_settings: bool) -> ModelLoadStatus {
    ModelLoadStatus {
        configured_model_path: runtime.configured_model_path,
        configured_model_present: runtime.configured_model_present,
        tier: TierStatus {
            feature: runtime.feature,
            compiled: runtime.compiled,
            enabled_in_settings,
            model_present: runtime.configured_model_present,
            loading: runtime.loading,
            loaded: runtime.loaded,
            loaded_model_path: runtime.loaded_model_path,
            last_error: runtime.last_error,
        },
    }
}

fn olmocr_tier_status(
    runtime: crate::ocr_vision::ModelStatus,
    enabled_in_settings: bool,
) -> ModelLoadStatus {
    ModelLoadStatus {
        configured_model_path: runtime.configured_model_path,
        configured_model_present: runtime.configured_model_present,
        tier: TierStatus {
            feature: runtime.feature,
            compiled: runtime.compiled,
            enabled_in_settings,
            model_present: runtime.configured_model_present,
            loading: runtime.loading,
            loaded: runtime.loaded,
            loaded_model_path: runtime.loaded_model_path,
            last_error: runtime.last_error,
        },
    }
}

#[tauri::command]
pub async fn tier_status_llm(state: State<'_, AppState>) -> AppResult<ModelLoadStatus> {
    let s = read_setting(&state, "llm").await?;
    let model_path = s.get("model_path").and_then(|v| v.as_str()).unwrap_or("");
    let runtime = crate::llm::status_for_config(Some(model_path));
    Ok(llm_tier_status(
        runtime,
        s.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false),
    ))
}

#[tauri::command]
pub async fn tier_load_llm(
    state: State<'_, AppState>,
    model_path: String,
) -> AppResult<ModelLoadStatus> {
    let s = read_setting(&state, "llm").await?;
    let runtime = crate::llm::load_model(model_path)?;
    Ok(llm_tier_status(
        runtime,
        s.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false),
    ))
}

#[tauri::command]
pub async fn tier_unload_llm(state: State<'_, AppState>) -> AppResult<ModelLoadStatus> {
    crate::llm::unload_model();
    tier_status_llm(state).await
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
        Ok(()) => Ok(PdfiumStatus {
            available: true,
            error: None,
        }),
        Err(e) => Ok(PdfiumStatus {
            available: false,
            error: Some(e),
        }),
    }
}

#[tauri::command]
pub async fn tier_status_olmocr(state: State<'_, AppState>) -> AppResult<ModelLoadStatus> {
    let s = read_setting(&state, "olmocr").await?;
    let model_path = s.get("model_path").and_then(|v| v.as_str()).unwrap_or("");
    let runtime = crate::ocr_vision::status_for_config(Some(model_path));
    Ok(olmocr_tier_status(
        runtime,
        s.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false),
    ))
}

#[tauri::command]
pub async fn tier_load_olmocr(
    state: State<'_, AppState>,
    model_path: String,
) -> AppResult<ModelLoadStatus> {
    let s = read_setting(&state, "olmocr").await?;
    let runtime = crate::ocr_vision::load_model(model_path)?;
    Ok(olmocr_tier_status(
        runtime,
        s.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false),
    ))
}

#[tauri::command]
pub async fn tier_unload_olmocr(state: State<'_, AppState>) -> AppResult<ModelLoadStatus> {
    crate::ocr_vision::unload_model();
    tier_status_olmocr(state).await
}
