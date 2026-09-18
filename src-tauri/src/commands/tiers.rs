use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{AppHandle, Emitter, State};

use crate::error::{AppError, AppResult};
use crate::state::AppState;

// The TensorBlock model card still lists this quantization, but its resolve
// endpoint currently returns 404 for direct downloads. The maintained
// lmstudio-community mirror exposes the same Phi-4-mini-reasoning filename
// and is the working direct-download source.
const PHI4_URL: &str = "https://huggingface.co/lmstudio-community/Phi-4-mini-reasoning-GGUF/resolve/main/Phi-4-mini-reasoning-Q4_K_M.gguf?download=true";
const OLMOCR_REPO: &str = "allenai/olmOCR-2-7B-1025";
const OLMOCR_FILES: &[&str] = &[
    "added_tokens.json",
    "chat_template.jinja",
    "chat_template.json",
    "config.json",
    "generation_config.json",
    "merges.txt",
    "model-00001-of-00004.safetensors",
    "model-00002-of-00004.safetensors",
    "model-00003-of-00004.safetensors",
    "model-00004-of-00004.safetensors",
    "model.safetensors.index.json",
    "preprocessor_config.json",
    "special_tokens_map.json",
    "tokenizer.json",
    "tokenizer_config.json",
    "video_preprocessor_config.json",
    "vocab.json",
];
const EVT_DOWNLOAD_PROGRESS: &str = "tier:download-progress";

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
    let config = crate::ocr::config_from_settings_with_paths(
        &s,
        Some(&state.data_dir),
        state.resource_dir.as_deref(),
    );
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

fn download_context(
    app: &AppHandle,
    resource: &str,
    file_index: usize,
    file_count: usize,
    cancel: Arc<AtomicBool>,
) -> crate::resources::DownloadContext {
    let reporter_app = app.clone();
    crate::resources::DownloadContext {
        resource: resource.to_string(),
        file_index,
        file_count,
        cancel,
        reporter: Arc::new(move |progress| {
            let _ = reporter_app.emit(EVT_DOWNLOAD_PROGRESS, &progress);
        }),
    }
}

async fn begin_download(state: &AppState) -> AppResult<Arc<AtomicBool>> {
    let mut active = state.download_cancel.lock().await;
    if active.is_some() {
        return Err(AppError::BadRequest(
            "another resource download is already in progress".into(),
        ));
    }
    let cancel = Arc::new(AtomicBool::new(false));
    *active = Some(cancel.clone());
    Ok(cancel)
}

async fn finish_download(state: &AppState, cancel: &Arc<AtomicBool>) {
    let mut active = state.download_cancel.lock().await;
    if active
        .as_ref()
        .is_some_and(|current| Arc::ptr_eq(current, cancel))
    {
        *active = None;
    }
}

#[tauri::command]
pub async fn tier_cancel_download(state: State<'_, AppState>) -> AppResult<()> {
    if let Some(cancel) = state.download_cancel.lock().await.as_ref() {
        cancel.store(true, Ordering::Relaxed);
    }
    Ok(())
}

async fn download_one(
    app: AppHandle,
    resource: &str,
    file_index: usize,
    file_count: usize,
    url: String,
    destination: std::path::PathBuf,
    cancel: Arc<AtomicBool>,
) -> AppResult<crate::resources::DownloadResult> {
    let context = download_context(&app, resource, file_index, file_count, cancel);
    tokio::task::spawn_blocking(move || {
        crate::resources::download_file(&url, &destination, &context)
    })
    .await
    .map_err(|error| AppError::Internal(format!("resource download task failed: {error}")))?
}

async fn set_model_path(state: &AppState, key: &str, path: &std::path::Path) -> AppResult<()> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let current: serde_json::Value = db
        .conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            [key],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    let mut object = current.as_object().cloned().unwrap_or_default();
    object.insert(
        "model_path".into(),
        serde_json::Value::String(path.to_string_lossy().into_owned()),
    );
    let value = serde_json::Value::Object(object).to_string();
    db.conn.execute(
        "INSERT INTO settings(key, value_json) VALUES(?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
        [key, &value],
    )?;
    Ok(())
}

#[tauri::command]
pub async fn tier_download_llm(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<ModelLoadStatus> {
    let cancel = begin_download(&state).await?;
    let destination = state.models_dir().join("phi-4-mini-reasoning-Q4_K_M.gguf");
    let result = download_one(
        app,
        "llm",
        0,
        1,
        PHI4_URL.to_string(),
        destination.clone(),
        cancel.clone(),
    )
    .await;
    finish_download(&state, &cancel).await;
    result?;
    set_model_path(&state, "llm", &destination).await?;
    tier_status_llm(state).await
}

#[tauri::command]
pub async fn tier_download_olmocr(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<ModelLoadStatus> {
    let cancel = begin_download(&state).await?;
    let root = state.models_dir().join("olmOCR-2-7B-1025");
    let root_for_download = root.clone();
    let app_for_download = app.clone();
    let cancel_for_download = cancel.clone();
    let result = match tokio::task::spawn_blocking(move || -> AppResult<()> {
        for (index, file) in OLMOCR_FILES.iter().enumerate() {
            let url =
                format!("https://huggingface.co/{OLMOCR_REPO}/resolve/main/{file}?download=true");
            let destination = crate::resources::safe_child(&root_for_download, file)?;
            let context = download_context(
                &app_for_download,
                "olmocr",
                index,
                OLMOCR_FILES.len(),
                cancel_for_download.clone(),
            );
            crate::resources::download_file(&url, &destination, &context)?;
        }
        Ok(())
    })
    .await
    {
        Ok(result) => result,
        Err(error) => Err(AppError::Internal(format!(
            "olmOCR download task failed: {error}"
        ))),
    };
    finish_download(&state, &cancel).await;
    result?;
    set_model_path(&state, "olmocr", &root).await?;
    tier_status_olmocr(state).await
}

#[tauri::command]
pub async fn tier_download_tesseract_language(
    app: AppHandle,
    state: State<'_, AppState>,
    language: String,
) -> AppResult<TierStatus> {
    let language = language.trim().to_ascii_lowercase();
    if language.is_empty()
        || language.len() > 12
        || !language
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(AppError::BadRequest(
            "invalid Tesseract language code".into(),
        ));
    }
    let url = format!(
        "https://raw.githubusercontent.com/tesseract-ocr/tessdata/main/{language}.traineddata"
    );
    let destination = state
        .models_dir()
        .join("tesseract")
        .join("tessdata")
        .join(format!("{language}.traineddata"));
    let cancel = begin_download(&state).await?;
    let result = download_one(app, "tesseract", 0, 1, url, destination, cancel.clone()).await;
    finish_download(&state, &cancel).await;
    result?;
    tier_status_tesseract(state).await
}

#[cfg(test)]
mod tests {
    use super::PHI4_URL;

    #[test]
    fn phi_model_download_url_uses_the_live_mirror() {
        assert!(PHI4_URL.starts_with(
            "https://huggingface.co/lmstudio-community/Phi-4-mini-reasoning-GGUF/resolve/main/"
        ));
        assert!(PHI4_URL.ends_with("Phi-4-mini-reasoning-Q4_K_M.gguf?download=true"));
    }
}
