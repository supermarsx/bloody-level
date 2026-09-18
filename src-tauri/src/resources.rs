//! User-initiated, streamed downloads for optional local resources.
//!
//! Nothing in this module runs during startup or ingestion. Downloads are
//! explicit settings actions and land under the app's managed models folder
//! through a temporary file followed by an atomic rename.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize)]
pub struct DownloadResult {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub resource: String,
    pub file: String,
    pub file_index: usize,
    pub file_count: usize,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub progress: Option<f32>,
    pub done: bool,
    pub cancelled: bool,
}

#[derive(Clone)]
pub struct DownloadContext {
    pub resource: String,
    pub file_index: usize,
    pub file_count: usize,
    pub cancel: Arc<AtomicBool>,
    pub reporter: Arc<dyn Fn(DownloadProgress) + Send + Sync>,
}

impl DownloadContext {
    fn report(
        &self,
        file: &str,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        done: bool,
        cancelled: bool,
    ) {
        (self.reporter)(DownloadProgress {
            resource: self.resource.clone(),
            file: file.to_string(),
            file_index: self.file_index,
            file_count: self.file_count,
            downloaded_bytes,
            total_bytes,
            progress: total_bytes.map(|total| {
                if total == 0 {
                    1.0
                } else {
                    (downloaded_bytes as f64 / total as f64).min(1.0) as f32
                }
            }),
            done,
            cancelled,
        });
    }
}

pub fn download_file(
    url: &str,
    destination: &Path,
    context: &DownloadContext,
) -> AppResult<DownloadResult> {
    if !is_allowed_url(url) {
        return Err(AppError::BadRequest(
            "resource URL must use HTTPS and an approved upstream host".into(),
        ));
    }
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("download");

    if destination.is_file() {
        let result = describe_file(destination)?;
        context.report(file_name, result.bytes, Some(result.bytes), true, false);
        return Ok(result);
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

        let body = response.into_body();
        let total_bytes = body.content_length();
        let mut reader = body.into_reader();
        let mut file = std::fs::File::create(&tmp)?;
        let mut hash = Sha256::new();
        let mut buffer = [0u8; 1024 * 1024];
        let mut bytes = 0u64;
        let mut last_report = Instant::now() - Duration::from_secs(1);
        context.report(file_name, 0, total_bytes, false, false);
        loop {
            if context.cancel.load(Ordering::Relaxed) {
                context.report(file_name, bytes, total_bytes, true, true);
                return Err(AppError::Cancelled);
            }
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            file.write_all(&buffer[..read])?;
            hash.update(&buffer[..read]);
            bytes = bytes.saturating_add(read as u64);
            if last_report.elapsed() >= Duration::from_millis(100) {
                context.report(file_name, bytes, total_bytes, false, false);
                last_report = Instant::now();
            }
        }
        if context.cancel.load(Ordering::Relaxed) {
            context.report(file_name, bytes, total_bytes, true, true);
            return Err(AppError::Cancelled);
        }
        file.flush()?;
        drop(file);
        std::fs::rename(&tmp, destination)?;
        context.report(file_name, bytes, total_bytes, true, false);

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
