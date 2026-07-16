use std::path::PathBuf;

use serde::Serialize;

use crate::error::AppResult;

#[derive(Serialize)]
pub struct SamplePdf {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
}

/// Returns the bundled `data test/*.pdf` sample files when running in dev mode
/// (where they live alongside the manifest). Returns an empty list in installed
/// builds where the data folder isn't shipped.
#[tauri::command]
pub async fn sample_pdf_paths() -> AppResult<Vec<SamplePdf>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest_dir.parent().map(|p| p.join("data test")),
        manifest_dir.parent().map(|p| p.join("data-test")),
        manifest_dir.parent().map(|p| p.join("data")),
    ];

    for candidate in candidates.iter().flatten() {
        if !candidate.is_dir() {
            continue;
        }
        let mut out = Vec::new();
        let entries = match std::fs::read_dir(candidate) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.eq_ignore_ascii_case("pdf"))
                .unwrap_or(false)
            {
                let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                out.push(SamplePdf {
                    path: path.to_string_lossy().to_string(),
                    name,
                    size_bytes,
                });
            }
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        return Ok(out);
    }
    Ok(vec![])
}
