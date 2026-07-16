// Tier-3 vision OCR - olmOCR-2. Feature-gated.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

static STATE: Lazy<Mutex<ModelRuntimeState>> =
    Lazy::new(|| Mutex::new(ModelRuntimeState::default()));

#[derive(Debug, Default)]
struct ModelRuntimeState {
    model_path: Option<PathBuf>,
    loading: bool,
    loaded: bool,
    last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ModelStatus {
    pub feature: &'static str,
    pub compiled: bool,
    pub configured_model_path: Option<String>,
    pub configured_model_present: bool,
    pub loaded_model_path: Option<String>,
    pub loading: bool,
    pub loaded: bool,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ExtractRequest {
    pub image_path: Option<String>,
    pub pdf_path: Option<String>,
    pub page_index: Option<usize>,
    pub source_text: Option<String>,
    pub tesseract_confidence: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RepairRequest {
    pub source_text: String,
    pub doc_confidence: Option<f32>,
    pub page_index: Option<usize>,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OcrTextOutput {
    pub text: String,
    pub confidence: f32,
    pub model_path: Option<String>,
    pub warnings: Vec<String>,
}

pub fn is_available() -> bool {
    cfg!(feature = "embedded-ocr-vision")
}

pub fn is_loading() -> bool {
    STATE.lock().map(|s| s.loading).unwrap_or(false)
}

pub fn is_loaded() -> bool {
    STATE.lock().map(|s| s.loaded).unwrap_or(false)
}

pub fn model_present(model_path: Option<&str>) -> bool {
    model_path
        .and_then(non_empty)
        .map(|p| Path::new(p).exists())
        .unwrap_or(false)
}

pub fn status_for_config(model_path: Option<&str>) -> ModelStatus {
    let configured = model_path.and_then(non_empty).map(ToOwned::to_owned);
    let configured_model_present = model_present(configured.as_deref());
    let state = STATE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    ModelStatus {
        feature: "embedded-ocr-vision",
        compiled: is_available(),
        configured_model_path: configured,
        configured_model_present,
        loaded_model_path: state
            .model_path
            .as_ref()
            .map(|path| path_to_string(path.as_path())),
        loading: state.loading,
        loaded: state.loaded,
        last_error: state.last_error.clone(),
    }
}

pub fn load_model(model_path: impl AsRef<Path>) -> AppResult<ModelStatus> {
    if !is_available() {
        return Err(AppError::BadRequest(
            "embedded OCR vision tier is not compiled; rebuild with --features embedded-ocr-vision"
                .into(),
        ));
    }

    let model_path = model_path.as_ref();
    if !model_path.exists() {
        return Err(AppError::NotFound(format!(
            "OCR vision model path does not exist: {}",
            model_path.display()
        )));
    }

    let canonical = canonical_or_original(model_path);
    {
        let mut state = STATE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.model_path = Some(canonical.clone());
        state.loading = true;
        state.loaded = false;
        state.last_error = None;
    }

    // The real olmOCR-2 session belongs here. This skeleton validates/tracks
    // the local model path but does not yet synthesize OCR output.
    let mut state = STATE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    state.loading = false;
    state.loaded = true;
    drop(state);
    Ok(status_for_config(Some(&path_to_string(&canonical))))
}

pub fn unload_model() {
    let mut state = STATE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *state = ModelRuntimeState::default();
}

pub fn extract_text(_request: ExtractRequest) -> AppResult<OcrTextOutput> {
    ensure_ready()?;
    Err(AppError::Internal(
        "olmOCR-2 extraction runtime is not implemented yet; no OCR output was produced".into(),
    ))
}

pub fn repair_text(_request: RepairRequest) -> AppResult<OcrTextOutput> {
    ensure_ready()?;
    Err(AppError::Internal(
        "olmOCR-2 repair runtime is not implemented yet; no OCR output was produced".into(),
    ))
}

fn ensure_ready() -> AppResult<()> {
    if !is_available() {
        return Err(AppError::BadRequest(
            "embedded OCR vision tier is not compiled; rebuild with --features embedded-ocr-vision"
                .into(),
        ));
    }
    if !is_loaded() {
        return Err(AppError::BadRequest(
            "embedded OCR vision model is not loaded; call load_model with a local olmOCR model path first"
                .into(),
        ));
    }
    Ok(())
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn canonical_or_original(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_model_file(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let unique = format!(
            "blevel-{name}-{}-{}.model",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        path.push(unique);
        std::fs::write(&path, b"placeholder").unwrap();
        path
    }

    #[test]
    fn model_present_checks_real_path() {
        let path = temp_model_file("ocr-present");
        assert!(model_present(Some(path.to_str().unwrap())));
        std::fs::remove_file(path).unwrap();
        assert!(!model_present(Some("")));
        assert!(!model_present(None));
    }

    #[test]
    fn status_reflects_configured_path_presence() {
        unload_model();
        let path = temp_model_file("ocr-status");
        let status = status_for_config(Some(path.to_str().unwrap()));
        assert_eq!(status.feature, "embedded-ocr-vision");
        assert_eq!(status.compiled, is_available());
        assert!(status.configured_model_present);
        assert!(!status.loading);
        assert!(!status.loaded);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    #[cfg(not(feature = "embedded-ocr-vision"))]
    fn extract_reports_unavailable_when_feature_missing() {
        let err = extract_text(ExtractRequest {
            image_path: None,
            pdf_path: None,
            page_index: None,
            source_text: Some("raw".into()),
            tesseract_confidence: Some(0.2),
        })
        .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    #[cfg(feature = "embedded-ocr-vision")]
    fn load_model_records_validated_path_when_feature_enabled() {
        unload_model();
        let path = temp_model_file("ocr-load");
        let status = load_model(&path).unwrap();
        assert!(status.loaded);
        assert!(!status.loading);
        assert!(status.loaded_model_path.is_some());
        unload_model();
        std::fs::remove_file(path).unwrap();
    }
}
