//! User-initiated, streamed downloads for optional local resources.
//!
//! Nothing in this module runs during startup or ingestion. Downloads are
//! explicit settings actions and land under the app's managed models folder
//! through a temporary file followed by an atomic rename.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize)]
pub struct DownloadResult {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

pub fn download_file(url: &str, destination: &Path) -> AppResult<DownloadResult> {
    if !is_allowed_url(url) {
        return Err(AppError::BadRequest(
            "resource URL must use HTTPS and an approved upstream host".into(),
        ));
    }
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if destination.is_file() {
        return describe_file(destination);
    }

    let tmp = destination.with_extension(format!(
        "{}.part",
        destination
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("download")
    ));
    let _ = std::fs::remove_file(&tmp);

    let result = (|| {
        let response = ureq::get(url)
            .header("User-Agent", "bloody-level-resource-manager/1")
            .call()
            .map_err(|error| AppError::Internal(format!("resource download failed: {error}")))?;
        if response.status() != 200 {
            return Err(AppError::Internal(format!(
                "resource download returned HTTP {}",
                response.status()
            )));
        }

        let mut reader = response.into_body().into_reader();
        let mut file = std::fs::File::create(&tmp)?;
        let mut hash = Sha256::new();
        let mut buffer = [0u8; 1024 * 1024];
        let mut bytes = 0u64;
        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            file.write_all(&buffer[..read])?;
            hash.update(&buffer[..read]);
            bytes = bytes.saturating_add(read as u64);
        }
        file.flush()?;
        drop(file);
        std::fs::rename(&tmp, destination)?;

        Ok(DownloadResult {
            path: destination.to_string_lossy().into_owned(),
            bytes,
            sha256: hex_lower(&hash.finalize()),
        })
    })();

    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

pub fn describe_file(path: &Path) -> AppResult<DownloadResult> {
    let mut file = std::fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
        bytes = bytes.saturating_add(read as u64);
    }
    Ok(DownloadResult {
        path: path.to_string_lossy().into_owned(),
        bytes,
        sha256: hex_lower(&hash.finalize()),
    })
}

pub fn safe_child(root: &Path, relative: &str) -> AppResult<PathBuf> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(AppError::BadRequest(
            "resource path escapes the models folder".into(),
        ));
    }
    Ok(root.join(relative_path))
}

fn is_allowed_url(url: &str) -> bool {
    url.starts_with("https://huggingface.co/")
        || url.starts_with("https://raw.githubusercontent.com/tesseract-ocr/")
}

fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
