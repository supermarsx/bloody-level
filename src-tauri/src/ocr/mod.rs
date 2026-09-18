// Tier-2 OCR - Tesseract CLI (no LLM). Feature-gated.

#[cfg(feature = "tesseract-ocr")]
use std::path::{Path, PathBuf};
#[cfg(feature = "tesseract-ocr")]
use std::process::{Command, Output};
#[cfg(feature = "tesseract-ocr")]
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::pdf::RenderedPdfPageImage;

const DEFAULT_LANGUAGES: &[&str] = &["eng", "por"];
const SPARSE_MIN_ALNUM_TOTAL: usize = 80;
const SPARSE_MIN_ALNUM_PER_PAGE: usize = 80;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OcrConfig {
    pub enabled: bool,
    pub languages: Vec<String>,
    pub datapath: Option<String>,
    pub executable: String,
}

#[derive(Debug, Clone)]
pub struct OcrOutput {
    pub text: String,
    pub mean_confidence: Option<f32>,
    pub page_count: usize,
}

#[derive(Debug, Clone)]
pub struct OcrRuntimeStatus {
    pub compiled: bool,
    pub model_present: bool,
    pub loaded: bool,
    pub error: Option<String>,
}

pub fn is_available() -> bool {
    cfg!(feature = "tesseract-ocr")
}

pub fn config_from_settings(settings: &Value) -> OcrConfig {
    OcrConfig {
        enabled: settings
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        languages: languages_from_settings(settings),
        datapath: settings
            .get("datapath")
            .or_else(|| settings.get("data_path"))
            .or_else(|| settings.get("tessdata_path"))
            .or_else(|| settings.get("model_path"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned),
        executable: settings
            .get("executable")
            .or_else(|| settings.get("binary_path"))
            .or_else(|| settings.get("tesseract_path"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("tesseract")
            .to_string(),
    }
}

/// Resolve managed Tesseract resources only when the user has not supplied a
/// path. A build can bundle a native executable under `binaries/tesseract`
/// and the settings UI can download official tessdata language files into the
/// per-instance models folder.
pub fn config_from_settings_with_paths(
    settings: &Value,
    data_dir: Option<&std::path::Path>,
    resource_dir: Option<&std::path::Path>,
) -> OcrConfig {
    let mut config = config_from_settings(settings);
    if config.executable == "tesseract" {
        if let Some(path) = bundled_executable(data_dir, resource_dir) {
            config.executable = path.to_string_lossy().into_owned();
        }
    }
    if config.datapath.is_none() {
        if let Some(path) = data_dir
            .map(|root| root.join("models").join("tesseract").join("tessdata"))
            .filter(|path| path.is_dir())
        {
            config.datapath = Some(path.to_string_lossy().into_owned());
        }
    }
    config
}

fn bundled_executable(
    data_dir: Option<&std::path::Path>,
    resource_dir: Option<&std::path::Path>,
) -> Option<std::path::PathBuf> {
    let name = if cfg!(target_os = "windows") {
        "tesseract.exe"
    } else {
        "tesseract"
    };
    [
        data_dir.map(|root| root.join("models").join("tesseract").join(name)),
        resource_dir.map(|root| root.join("binaries").join("tesseract").join(name)),
    ]
    .into_iter()
    .flatten()
    .find(|path| path.is_file())
}

pub fn languages_from_settings(settings: &Value) -> Vec<String> {
    let languages: Vec<String> = settings
        .get("languages")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default();

    if languages.is_empty() {
        DEFAULT_LANGUAGES.iter().map(|s| (*s).to_string()).collect()
    } else {
        languages
    }
}

#[allow(dead_code)]
pub fn language_spec(languages: &[String]) -> String {
    effective_languages(languages).join("+")
}

fn effective_languages(languages: &[String]) -> Vec<String> {
    let cleaned: Vec<String> = languages
        .iter()
        .map(|language| language.trim())
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    if cleaned.is_empty() {
        DEFAULT_LANGUAGES.iter().map(|s| (*s).to_string()).collect()
    } else {
        cleaned
    }
}

pub fn should_fallback_to_ocr(text: &str, page_count: usize) -> bool {
    if page_count == 0 {
        return false;
    }

    let alnum = text.chars().filter(|c| c.is_alphanumeric()).count();
    alnum < SPARSE_MIN_ALNUM_TOTAL || (alnum / page_count.max(1)) < SPARSE_MIN_ALNUM_PER_PAGE
}

pub fn runtime_status(config: &OcrConfig) -> OcrRuntimeStatus {
    runtime_status_impl(config)
}

pub fn recognize_pages(pages: &[RenderedPdfPageImage], config: &OcrConfig) -> AppResult<OcrOutput> {
    recognize_pages_impl(pages, config)
}

#[cfg(not(feature = "tesseract-ocr"))]
fn runtime_status_impl(_config: &OcrConfig) -> OcrRuntimeStatus {
    OcrRuntimeStatus {
        compiled: false,
        model_present: false,
        loaded: false,
        error: Some(
            "Tesseract OCR is not compiled; rebuild with --features tesseract-ocr".to_string(),
        ),
    }
}

#[cfg(feature = "tesseract-ocr")]
fn runtime_status_impl(config: &OcrConfig) -> OcrRuntimeStatus {
    match available_languages(config) {
        Ok(available) => {
            let missing = missing_languages(config, &available);
            OcrRuntimeStatus {
                compiled: true,
                model_present: missing.is_empty(),
                loaded: missing.is_empty(),
                error: (!missing.is_empty()).then(|| {
                    format!(
                        "Tesseract is installed, but tessdata is missing for: {}",
                        missing.join(", ")
                    )
                }),
            }
        }
        Err(err) => OcrRuntimeStatus {
            compiled: true,
            model_present: false,
            loaded: false,
            error: Some(err),
        },
    }
}

#[cfg(not(feature = "tesseract-ocr"))]
fn recognize_pages_impl(
    _pages: &[RenderedPdfPageImage],
    _config: &OcrConfig,
) -> AppResult<OcrOutput> {
    Err(AppError::BadRequest(
        "Tesseract OCR is not compiled; rebuild with --features tesseract-ocr".to_string(),
    ))
}

#[cfg(feature = "tesseract-ocr")]
fn recognize_pages_impl(
    pages: &[RenderedPdfPageImage],
    config: &OcrConfig,
) -> AppResult<OcrOutput> {
    if pages.is_empty() {
        return Ok(OcrOutput {
            text: String::new(),
            mean_confidence: None,
            page_count: 0,
        });
    }

    let status = runtime_status_impl(config);
    if !status.loaded {
        return Err(AppError::BadRequest(format!(
            "Tesseract OCR is unavailable: {}",
            status
                .error
                .unwrap_or_else(|| "language data is missing".to_string())
        )));
    }

    let temp_dir = create_temp_dir()?;
    let _cleanup = TempDirCleanup(temp_dir.clone());
    let mut text = String::new();

    for page in pages {
        let image_path = temp_dir.join(format!("page-{:04}.png", page.page_index + 1));
        save_page_image(page, &image_path)?;
        let page_text = run_tesseract_page(config, &image_path, page.page_index)?;
        text.push_str(&page_text);
        if !text.ends_with('\n') {
            text.push('\n');
        }
    }

    Ok(OcrOutput {
        text,
        mean_confidence: None,
        page_count: pages.len(),
    })
}

#[cfg(feature = "tesseract-ocr")]
fn save_page_image(page: &RenderedPdfPageImage, image_path: &Path) -> AppResult<()> {
    let width = u32::try_from(page.width).map_err(|_| {
        AppError::Pdf(format!(
            "OCR page {} has an invalid width",
            page.page_index + 1
        ))
    })?;
    let height = u32::try_from(page.height).map_err(|_| {
        AppError::Pdf(format!(
            "OCR page {} has an invalid height",
            page.page_index + 1
        ))
    })?;
    let expected_len = (width as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| AppError::Pdf("OCR image dimensions overflowed".to_string()))?;
    if page.bytes_per_pixel != 1 || page.bytes_per_line != page.width {
        return Err(AppError::Pdf(format!(
            "OCR page {} is not an 8-bit grayscale image",
            page.page_index + 1
        )));
    }
    if page.bytes.len() != expected_len {
        return Err(AppError::Pdf(format!(
            "OCR page {} image buffer length does not match dimensions",
            page.page_index + 1
        )));
    }

    let image = image::GrayImage::from_raw(width, height, page.bytes.clone()).ok_or_else(|| {
        AppError::Pdf(format!(
            "OCR page {} could not be converted to an image",
            page.page_index + 1
        ))
    })?;
    image.save(image_path).map_err(|err| {
        AppError::Internal(format!(
            "failed to write OCR image for page {}: {err}",
            page.page_index + 1
        ))
    })
}

#[cfg(feature = "tesseract-ocr")]
fn run_tesseract_page(
    config: &OcrConfig,
    image_path: &Path,
    page_index: usize,
) -> AppResult<String> {
    let mut cmd = tesseract_command(config);
    add_tessdata_args(&mut cmd, config);
    cmd.arg(image_path)
        .arg("stdout")
        .arg("-l")
        .arg(language_spec(&config.languages))
        .arg("--psm")
        .arg("3");

    let output = cmd.output().map_err(|err| {
        AppError::Internal(format!(
            "Tesseract executable '{}' failed on page {}: {err}",
            config.executable,
            page_index + 1
        ))
    })?;
    if !output.status.success() {
        return Err(AppError::Internal(format!(
            "Tesseract failed to recognize rendered page {}: {}",
            page_index + 1,
            output_message(&output)
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(feature = "tesseract-ocr")]
fn available_languages(config: &OcrConfig) -> Result<Vec<String>, String> {
    let mut cmd = tesseract_command(config);
    add_tessdata_args(&mut cmd, config);
    cmd.arg("--list-langs");

    let output = cmd.output().map_err(|err| {
        format!(
            "Tesseract executable '{}' is not available: {err}",
            config.executable
        )
    })?;
    if !output.status.success() {
        return Err(format!(
            "Tesseract language probe failed: {}",
            output_message(&output)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(stdout
        .lines()
        .chain(stderr.lines())
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with("List of available languages"))
        .map(ToOwned::to_owned)
        .collect())
}

#[cfg(feature = "tesseract-ocr")]
fn missing_languages(config: &OcrConfig, available: &[String]) -> Vec<String> {
    effective_languages(&config.languages)
        .into_iter()
        .filter(|language| !available.iter().any(|item| item == language))
        .collect()
}

#[cfg(feature = "tesseract-ocr")]
fn tesseract_command(config: &OcrConfig) -> Command {
    Command::new(&config.executable)
}

#[cfg(feature = "tesseract-ocr")]
fn add_tessdata_args(command: &mut Command, config: &OcrConfig) {
    if let Some(dir) = tessdata_dir(config) {
        command.arg("--tessdata-dir").arg(dir);
    }
}

#[cfg(feature = "tesseract-ocr")]
fn tessdata_dir(config: &OcrConfig) -> Option<PathBuf> {
    let root = config
        .datapath
        .as_deref()
        .map(PathBuf::from)
        .or_else(|| std::env::var("TESSDATA_PREFIX").ok().map(PathBuf::from))?;

    let preferred = effective_languages(&config.languages)
        .into_iter()
        .next()
        .unwrap_or_else(|| "eng".to_string());
    let traineddata = format!("{preferred}.traineddata");
    if root.join(&traineddata).exists() {
        return Some(root);
    }
    let nested = root.join("tessdata");
    if nested.join(&traineddata).exists() {
        return Some(nested);
    }
    Some(root)
}

#[cfg(feature = "tesseract-ocr")]
fn output_message(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let message = stderr.trim();
    if !message.is_empty() {
        return message.to_string();
    }
    let message = stdout.trim();
    if !message.is_empty() {
        return message.to_string();
    }
    format!("exit status {}", output.status)
}

#[cfg(feature = "tesseract-ocr")]
fn create_temp_dir() -> AppResult<PathBuf> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut dir = std::env::temp_dir();
    dir.push(format!("bloody-level-ocr-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|err| {
        AppError::Internal(format!(
            "failed to create temporary OCR directory {}: {err}",
            dir.display()
        ))
    })?;
    Ok(dir)
}

#[cfg(feature = "tesseract-ocr")]
struct TempDirCleanup(PathBuf);

#[cfg(feature = "tesseract-ocr")]
impl Drop for TempDirCleanup {
    fn drop(&mut self) {
        if let Err(err) = std::fs::remove_dir_all(&self.0) {
            tracing::warn!(
                path = %self.0.display(),
                error = %err,
                "failed to clean temporary OCR directory"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_detection_ignores_empty_pdf() {
        assert!(!should_fallback_to_ocr("", 0));
    }

    #[test]
    fn sparse_detection_triggers_for_empty_page_text() {
        assert!(should_fallback_to_ocr(" \n \t", 2));
    }

    #[test]
    fn sparse_detection_accepts_reasonable_embedded_text() {
        let text = "Hemograma completo\n".repeat(30);
        assert!(!should_fallback_to_ocr(&text, 1));
    }

    #[test]
    fn language_settings_fall_back_to_defaults() {
        assert_eq!(
            languages_from_settings(&serde_json::json!({})),
            vec!["eng".to_string(), "por".to_string()]
        );
    }

    #[test]
    fn language_spec_joins_for_tesseract() {
        assert_eq!(
            language_spec(&["eng".to_string(), "por".to_string()]),
            "eng+por"
        );
    }
}
