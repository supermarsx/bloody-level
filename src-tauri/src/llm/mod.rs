// Tier-4 LLM - Phi-4-mini-reasoning via llama.cpp. Feature-gated.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

static STATE: Lazy<Mutex<ModelRuntimeState>> =
    Lazy::new(|| Mutex::new(ModelRuntimeState::default()));

#[cfg(feature = "embedded-llm")]
struct LoadedModel {
    model: llama_cpp_2::model::LlamaModel,
    backend: llama_cpp_2::llama_backend::LlamaBackend,
}

#[derive(Default)]
struct ModelRuntimeState {
    model_path: Option<PathBuf>,
    loading: bool,
    loaded: bool,
    last_error: Option<String>,
    #[cfg(feature = "embedded-llm")]
    runtime: Option<LoadedModel>,
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
pub struct RepairRequest {
    pub source_text: String,
    pub doc_confidence: Option<f32>,
    pub report_id: Option<String>,
    pub max_output_chars: Option<usize>,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RepairOutput {
    pub repaired_text: String,
    pub confidence: f32,
    pub model_path: Option<String>,
    pub prompt_hash: Option<String>,
    pub warnings: Vec<String>,
}

pub fn is_available() -> bool {
    cfg!(feature = "embedded-llm")
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
        feature: "embedded-llm",
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
            "embedded LLM tier is not compiled; rebuild with --features embedded-llm".into(),
        ));
    }

    let model_path = model_path.as_ref();
    if !model_path.exists() {
        return Err(AppError::NotFound(format!(
            "LLM model path does not exist: {}",
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

    #[cfg(feature = "embedded-llm")]
    let runtime = (|| {
        let backend = llama_cpp_2::llama_backend::LlamaBackend::init()
            .map_err(|error| AppError::Internal(format!("initialize llama.cpp: {error}")))?;
        let model = llama_cpp_2::model::LlamaModel::load_from_file(
            &backend,
            &canonical,
            &llama_cpp_2::model::params::LlamaModelParams::default(),
        )
        .map_err(|error| AppError::Internal(format!("load LLM model: {error}")))?;
        Ok::<LoadedModel, AppError>(LoadedModel { model, backend })
    })();

    #[cfg(feature = "embedded-llm")]
    let runtime = match runtime {
        Ok(runtime) => runtime,
        Err(error) => {
            let mut state = STATE
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.loading = false;
            state.loaded = false;
            state.last_error = Some(error.to_string());
            return Err(error);
        }
    };

    let mut state = STATE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    #[cfg(feature = "embedded-llm")]
    {
        state.runtime = Some(runtime);
    }
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

pub fn repair_text(request: RepairRequest) -> AppResult<RepairOutput> {
    if !is_available() {
        return Err(AppError::BadRequest(
            "embedded LLM tier is not compiled; rebuild with --features embedded-llm".into(),
        ));
    }
    if !is_loaded() {
        return Err(AppError::BadRequest(
            "embedded LLM model is not loaded; call load_model with a local Phi-4 model path first"
                .into(),
        ));
    }
    #[cfg(feature = "embedded-llm")]
    {
        use std::num::NonZeroU32;

        use llama_cpp_2::context::params::LlamaContextParams;
        use llama_cpp_2::llama_batch::LlamaBatch;
        use llama_cpp_2::model::AddBos;
        use llama_cpp_2::sampling::LlamaSampler;

        let state = STATE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let loaded = state.runtime.as_ref().ok_or_else(|| {
            AppError::BadRequest(
                "embedded LLM model is not loaded; call load_model with a local Phi-4 model path first"
                    .into(),
            )
        })?;

        let max_output_chars = request.max_output_chars.unwrap_or(8_000).clamp(256, 16_000);
        let source_text: String = request.source_text.chars().take(24_000).collect();
        let prompt = format!(
            "You repair OCR text from a Portuguese clinical laboratory report.\n\
             Return only the corrected plain-text report, with no commentary, markdown,\n\
             diagnosis, or invented values. Preserve patient names, dates, analyte names,\n\
             numbers, units, and reference ranges. Fix broken line joins and obvious OCR\n\
             character errors only. If uncertain, preserve the original text.\n\n\
             OCR text:\n{}\n",
            source_text
        );
        let tokens = loaded
            .model
            .str_to_token(&prompt, AddBos::Always)
            .map_err(|error| AppError::Internal(format!("tokenize LLM prompt: {error}")))?;
        let n_ctx = 4096usize;
        if tokens.len() + 256 >= n_ctx {
            return Err(AppError::BadRequest(
                "LLM repair input is too large for the configured context window".into(),
            ));
        }
        let max_output_tokens = max_output_chars.min(n_ctx - tokens.len() - 1);

        let thread_count = std::thread::available_parallelism()
            .map(|count| count.get().min(8) as i32)
            .unwrap_or(4);
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(n_ctx as u32))
            .with_n_batch(n_ctx as u32)
            .with_n_threads(thread_count);
        let mut ctx = loaded
            .model
            .new_context(&loaded.backend, ctx_params)
            .map_err(|error| AppError::Internal(format!("create LLM context: {error}")))?;
        let mut prompt_batch = LlamaBatch::new(tokens.len(), 1);
        prompt_batch
            .add_sequence(&tokens, 0, false)
            .map_err(|error| AppError::Internal(format!("prepare LLM prompt: {error}")))?;
        ctx.decode(&mut prompt_batch)
            .map_err(|error| AppError::Internal(format!("evaluate LLM prompt: {error}")))?;

        let mut sampler = LlamaSampler::greedy();
        let mut output = String::new();
        let mut next_batch = LlamaBatch::new(1, 1);
        for (next_pos, _) in (tokens.len() as i32..).zip(0..max_output_tokens) {
            let token = sampler.sample(&ctx, -1);
            sampler.accept(token);
            if token == loaded.model.token_eos() {
                break;
            }
            let piece = loaded
                .model
                .token_to_piece_bytes(token, 128, false, None)
                .map_err(|error| AppError::Internal(format!("decode LLM output: {error}")))?;
            output.push_str(&String::from_utf8_lossy(&piece));
            if output.chars().count() >= max_output_chars {
                break;
            }
            next_batch.clear();
            next_batch
                .add(token, next_pos, &[0], true)
                .map_err(|error| AppError::Internal(format!("prepare LLM token: {error}")))?;
            ctx.decode(&mut next_batch)
                .map_err(|error| AppError::Internal(format!("evaluate LLM token: {error}")))?;
        }

        let repaired_text = output.trim().to_string();
        if repaired_text.is_empty() {
            return Err(AppError::Internal(
                "Phi-4 produced no repaired text; original OCR text was kept".into(),
            ));
        }
        Ok(RepairOutput {
            repaired_text,
            confidence: 0.75,
            model_path: state
                .model_path
                .as_ref()
                .map(|path| path_to_string(path.as_path())),
            prompt_hash: None,
            warnings: vec![
                "LLM output is advisory and must be reviewed against the source PDF".into(),
            ],
        })
    }

    #[cfg(not(feature = "embedded-llm"))]
    {
        let _ = request;
        unreachable!("feature check above returns when embedded-llm is unavailable")
    }
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

    static TEST_STATE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn temp_model_file(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let unique = format!(
            "bloody-level-{name}-{}-{}.gguf",
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
        let path = temp_model_file("llm-present");
        assert!(model_present(Some(path.to_str().unwrap())));
        std::fs::remove_file(path).unwrap();
        assert!(!model_present(Some("")));
        assert!(!model_present(None));
    }

    #[test]
    fn status_reflects_configured_path_presence() {
        let _guard = TEST_STATE_LOCK.lock().unwrap();
        unload_model();
        let path = temp_model_file("llm-status");
        let status = status_for_config(Some(path.to_str().unwrap()));
        assert_eq!(status.feature, "embedded-llm");
        assert_eq!(status.compiled, is_available());
        assert!(status.configured_model_present);
        assert!(!status.loading);
        assert!(!status.loaded);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    #[cfg(not(feature = "embedded-llm"))]
    fn load_model_reports_unavailable_when_feature_missing() {
        let path = temp_model_file("llm-unavailable");
        let err = load_model(&path).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    #[cfg(feature = "embedded-llm")]
    fn load_model_rejects_invalid_model_when_feature_enabled() {
        let _guard = TEST_STATE_LOCK.lock().unwrap();
        unload_model();
        let path = temp_model_file("llm-load");
        let error = load_model(&path).unwrap_err();
        assert!(matches!(error, AppError::Internal(_)));
        assert!(!is_loaded());
        unload_model();
        std::fs::remove_file(path).unwrap();
    }
}
