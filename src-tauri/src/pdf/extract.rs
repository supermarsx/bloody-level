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

fn hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{:02x}", b);
    }
    s
}
