// Cross-platform "open this file in the OS default app" command.
//
// Confines the request to our managed pdf_dir / data_dir so a compromised
// frontend can't ask us to launch arbitrary files.

use secrecy::{ExposeSecret, SecretBox};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// Opens `path` in the OS default application. The path must resolve into
/// the app's data directory — anything outside is rejected.
#[tauri::command]
pub async fn open_file_external(state: State<'_, AppState>, path: String) -> AppResult<()> {
    let target = PathBuf::from(&path);
    if !target.exists() {
        return Err(AppError::NotFound(format!("file not found: {path}")));
    }

    let canon = target
        .canonicalize()
        .map_err(|e| AppError::Internal(format!("canonicalize {path}: {e}")))?;
    let allowed_root = state
        .data_dir
        .canonicalize()
        .map_err(|e| AppError::Internal(format!("canonicalize data_dir: {e}")))?;

    if !is_within(&canon, &allowed_root) {
        return Err(AppError::BadRequest(format!(
            "refusing to open path outside app data dir: {}",
            canon.display()
        )));
    }

    let pdf_root = state
        .pdf_dir()
        .canonicalize()
        .map_err(|e| AppError::Internal(format!("canonicalize pdf_dir: {e}")))?;
    let is_managed_pdf = canon.is_file()
        && canon
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
        && is_within(&canon, &pdf_root);

    let viewer_path = if is_managed_pdf {
        let key = {
            let guard = state.pdf_key.lock().await;
            let key = guard.as_ref().ok_or(AppError::Locked)?;
            SecretBox::new(Box::new(*key.expose_secret()))
        };
        let plaintext = crate::crypto::file::decrypt_file(&canon, &key)?;
        let temp_dir = std::env::temp_dir().join("bloody-level-open");
        std::fs::create_dir_all(&temp_dir)?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let temporary = temp_dir.join(format!("report-{}-{stamp}.pdf", std::process::id()));
        std::fs::write(&temporary, plaintext)?;

        // External PDF viewers need a plaintext hand-off. Keep that hand-off
        // outside the vault and remove it after a short review window.
        let cleanup_path = temporary.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(15 * 60));
            let _ = std::fs::remove_file(cleanup_path);
        });
        temporary
    } else {
        canon
    };

    spawn_opener(&viewer_path).map_err(|e| AppError::Internal(format!("opener failed: {e}")))?;
    Ok(())
}

/// Opens an http(s) URL in the OS default browser. Restricted to those two
/// schemes so a compromised frontend can't trigger a `file://` or
/// command-injection style URI through us.
#[tauri::command]
pub async fn open_url(url: String) -> AppResult<()> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err(AppError::BadRequest("Only http(s) URLs are allowed".into()));
    }
    if trimmed.len() > 2048 {
        return Err(AppError::BadRequest("URL is too long".into()));
    }
    spawn_url_opener(trimmed).map_err(|e| AppError::Internal(format!("url opener failed: {e}")))?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn spawn_url_opener(url: &str) -> std::io::Result<()> {
    std::process::Command::new("cmd")
        .args(["/C", "start", ""])
        .arg(url)
        .spawn()?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn spawn_url_opener(url: &str) -> std::io::Result<()> {
    std::process::Command::new("open").arg(url).spawn()?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn spawn_url_opener(url: &str) -> std::io::Result<()> {
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    Ok(())
}

fn is_within(child: &Path, ancestor: &Path) -> bool {
    child.starts_with(ancestor)
}

#[cfg(target_os = "windows")]
fn spawn_opener(path: &Path) -> std::io::Result<()> {
    // `cmd /C start "" "<path>"` is the canonical way to invoke ShellExecute.
    // The empty quoted "" is the window title — required so paths with spaces
    // aren't misread as the title.
    std::process::Command::new("cmd")
        .args(["/C", "start", ""])
        .arg(path)
        .spawn()?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn spawn_opener(path: &Path) -> std::io::Result<()> {
    std::process::Command::new("open").arg(path).spawn()?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn spawn_opener(path: &Path) -> std::io::Result<()> {
    std::process::Command::new("xdg-open").arg(path).spawn()?;
    Ok(())
}
