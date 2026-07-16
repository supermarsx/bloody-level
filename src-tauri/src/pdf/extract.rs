use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use pdfium_render::prelude::*;
use sha2::{Digest, Sha256};

use crate::error::{AppError, AppResult};

/// App resource directory (set at Tauri setup). pdfium discovery checks here
/// after looking next to the executable.
static APP_RESOURCE_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn set_resource_dir(path: PathBuf) {
    let _ = APP_RESOURCE_DIR.set(path);
}

#[derive(Debug, Clone)]
pub struct ExtractedPdf {
    pub sha256_hex: String,
    pub byte_count: usize,
    pub combined_text: String,
    pub page_count: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RenderedPdfPageImage {
    pub page_index: usize,
    pub width: i32,
    pub height: i32,
    pub bytes_per_pixel: i32,
    pub bytes_per_line: i32,
    pub source_resolution: i32,
    pub bytes: Vec<u8>,
}

/// Locate and bind to pdfium.
fn build_pdfium() -> AppResult<Pdfium> {
    let mut tried: Vec<String> = Vec::new();

    if let Ok(env_path) = std::env::var("PDFIUM_LIB_PATH") {
        match Pdfium::bind_to_library(&env_path) {
            Ok(b) => return Ok(Pdfium::new(b)),
            Err(e) => tried.push(format!("PDFIUM_LIB_PATH={env_path} ({e})")),
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for candidate in candidate_lib_paths(dir) {
                if candidate.exists() {
                    match Pdfium::bind_to_library(&candidate) {
                        Ok(b) => return Ok(Pdfium::new(b)),
                        Err(e) => tried.push(format!("{} ({e})", candidate.display())),
                    }
                }
            }
        }
    }

    if let Some(rd) = APP_RESOURCE_DIR.get() {
        let bin_dir = rd.join("binaries");
        for candidate in candidate_lib_paths(&bin_dir) {
            if candidate.exists() {
                match Pdfium::bind_to_library(&candidate) {
                    Ok(b) => return Ok(Pdfium::new(b)),
                    Err(e) => tried.push(format!("{} ({e})", candidate.display())),
                }
            }
        }
    }

    let dev_bin_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries");
    if dev_bin_dir.exists() {
        for candidate in candidate_lib_paths(&dev_bin_dir) {
            if candidate.exists() {
                match Pdfium::bind_to_library(&candidate) {
                    Ok(b) => return Ok(Pdfium::new(b)),
                    Err(e) => tried.push(format!("{} ({e})", candidate.display())),
                }
            }
        }
    }

    match Pdfium::bind_to_system_library() {
        Ok(b) => Ok(Pdfium::new(b)),
        Err(e) => {
            tried.push(format!("system library ({e})"));
            Err(AppError::Pdf(format!(
                "pdfium library not found. The build script normally downloads it \
                 automatically; check the cargo build log for download errors. \
                 You can also install manually from \
                 https://github.com/bblanchon/pdfium-binaries/releases. \
                 Tried: [{}]",
                tried.join(" | ")
            )))
        }
    }
}

fn candidate_lib_paths(dir: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    for name in ["pdfium.dll", "libpdfium.so", "libpdfium.dylib", "pdfium.so"] {
        v.push(dir.join(name));
    }
    v
}

/// Cheap probe used by the Settings page to surface tier-1 availability without
/// actually parsing a PDF.
pub fn pdfium_available() -> Result<(), String> {
    build_pdfium().map(|_| ()).map_err(|e| e.to_string())
}

pub fn extract(path: &Path) -> AppResult<ExtractedPdf> {
    let bytes = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let sha256_hex = hex_lower(&hasher.finalize());
    extract_bytes(&bytes, sha256_hex)
}

pub fn extract_bytes(bytes: &[u8], sha256_hex: String) -> AppResult<ExtractedPdf> {
    let pdfium = build_pdfium()?;
    let doc = pdfium
        .load_pdf_from_byte_slice(bytes, None)
        .map_err(|e| AppError::Pdf(format!("could not open document: {e}")))?;

    let mut combined = String::new();
    let mut page_count = 0usize;
    for (idx, page) in doc.pages().iter().enumerate() {
        let text = page
            .text()
            .map_err(|e| AppError::Pdf(format!("text extraction (page {idx}): {e}")))?;
        combined.push_str(&text.all());
        combined.push('\n');
        page_count = idx + 1;
    }

    Ok(ExtractedPdf {
        sha256_hex,
        byte_count: bytes.len(),
        combined_text: combined,
        page_count,
    })
}

pub fn render_pages_for_ocr(path: &Path) -> AppResult<Vec<RenderedPdfPageImage>> {
    let bytes = std::fs::read(path)?;
    render_pages_for_ocr_bytes(&bytes)
}

pub fn render_pages_for_ocr_bytes(bytes: &[u8]) -> AppResult<Vec<RenderedPdfPageImage>> {
    const TARGET_WIDTH_PX: i32 = 2480;
    const MAX_HEIGHT_PX: i32 = 3508;
    const SOURCE_RESOLUTION_DPI: i32 = 300;

    let pdfium = build_pdfium()?;
    let doc = pdfium
        .load_pdf_from_byte_slice(bytes, None)
        .map_err(|e| AppError::Pdf(format!("could not open document for OCR rendering: {e}")))?;

    let render_config = PdfRenderConfig::new()
        .set_target_width(TARGET_WIDTH_PX)
        .set_maximum_height(MAX_HEIGHT_PX)
        .use_grayscale_rendering(true)
        .use_print_quality(true)
        .render_annotations(false)
        .render_form_data(true);

    let mut pages = Vec::new();
    for (idx, page) in doc.pages().iter().enumerate() {
        let image = page
            .render_with_config(&render_config)
            .map_err(|e| AppError::Pdf(format!("OCR render (page {idx}): {e}")))?
            .as_image()
            .into_luma8();
        let width = i32::try_from(image.width())
            .map_err(|_| AppError::Pdf(format!("OCR render (page {idx}) width exceeds i32")))?;
        let height = i32::try_from(image.height())
            .map_err(|_| AppError::Pdf(format!("OCR render (page {idx}) height exceeds i32")))?;
        pages.push(RenderedPdfPageImage {
            page_index: idx,
            width,
            height,
            bytes_per_pixel: 1,
            bytes_per_line: width,
            source_resolution: SOURCE_RESOLUTION_DPI,
            bytes: image.into_raw(),
        });
    }

    Ok(pages)
}

fn hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{:02x}", b);
    }
    s
}
