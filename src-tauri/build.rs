use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Expose target triple to the binary for diagnostics (app_info command).
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".into());
    println!("cargo:rustc-env=TARGET_TRIPLE={target}");

    // Fetch pdfium FIRST so tauri-build can package it into bundle.resources.
    // Soft-fail: print warnings but let the build continue. Runtime discovery
    // gives the user a clear, actionable error if no pdfium is found anywhere.
    if let Err(e) = ensure_pdfium() {
        println!("cargo:warning=pdfium download failed: {e}");
        println!(
            "cargo:warning=Manual install: https://github.com/bblanchon/pdfium-binaries/releases"
        );
    }

    if let Err(e) = bundle_tesseract() {
        println!("cargo:warning=tesseract bundle skipped: {e}");
    }

    tauri_build::build();
}

/// Copy a prepared native Tesseract distribution into the application
/// resources when the release environment provides `TESSERACT_BUNDLE_DIR`.
/// We do not silently fetch executable code from an unpinned third-party
/// installer during builds; platform packaging can provide a reviewed,
/// licensed directory and the runtime resolves it automatically.
fn bundle_tesseract() -> Result<(), String> {
    let source = match env::var("TESSERACT_BUNDLE_DIR") {
        Ok(value) if !value.trim().is_empty() => PathBuf::from(value),
        _ => return Ok(()),
    };
    if !source.is_dir() {
        return Err(format!(
            "TESSERACT_BUNDLE_DIR is not a directory: {source:?}"
        ));
    }
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").map_err(|e| format!("CARGO_MANIFEST_DIR: {e}"))?,
    );
    let destination = manifest_dir.join("binaries").join("tesseract");
    copy_dir_recursive(&source, &destination)?;
    println!("cargo:rerun-if-env-changed=TESSERACT_BUNDLE_DIR");
    println!("cargo:rerun-if-changed={}", source.display());
    Ok(())
}

fn copy_dir_recursive(
    source: &std::path::Path,
    destination: &std::path::Path,
) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|e| format!("mkdir {destination:?}: {e}"))?;
    for entry in fs::read_dir(source).map_err(|e| format!("read {source:?}: {e}"))? {
        let entry = entry.map_err(|e| format!("read bundle entry: {e}"))?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &destination_path)?;
        } else if source_path.is_file() {
            fs::copy(&source_path, &destination_path)
                .map_err(|e| format!("copy {source_path:?}: {e}"))?;
        }
    }
    Ok(())
}

fn ensure_pdfium() -> Result<(), String> {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    // bblanchon's archive naming is {plat}-{arch}; the older naming used full
    // names like "pdfium-windows-x64". Try both. The first that downloads wins.
    let (lib_filename, archive_match, candidates) = match (target_os.as_str(), target_arch.as_str())
    {
        ("windows", "x86_64") => (
            "pdfium.dll",
            "pdfium.dll",
            vec!["pdfium-win-x64.tgz", "pdfium-windows-x64.tgz"],
        ),
        ("windows", "aarch64") => (
            "pdfium.dll",
            "pdfium.dll",
            vec!["pdfium-win-arm64.tgz", "pdfium-windows-arm64.tgz"],
        ),
        ("linux", "x86_64") => ("libpdfium.so", "libpdfium.so", vec!["pdfium-linux-x64.tgz"]),
        ("linux", "aarch64") => (
            "libpdfium.so",
            "libpdfium.so",
            vec!["pdfium-linux-arm64.tgz"],
        ),
        ("macos", "x86_64") => (
            "libpdfium.dylib",
            "libpdfium.dylib",
            vec!["pdfium-mac-x64.tgz"],
        ),
        ("macos", "aarch64") => (
            "libpdfium.dylib",
            "libpdfium.dylib",
            vec!["pdfium-mac-arm64.tgz"],
        ),
        (os, arch) => return Err(format!("unsupported target: {os}-{arch}")),
    };

    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").map_err(|e| format!("CARGO_MANIFEST_DIR: {e}"))?,
    );
    let bundle_dir = manifest_dir.join("binaries");
    let bundled = bundle_dir.join(lib_filename);

    fs::create_dir_all(&bundle_dir).map_err(|e| format!("mkdir {bundle_dir:?}: {e}"))?;
    // Always create .keep so tauri-build's `binaries/*` glob always matches —
    // even when pdfium download fails the bundle config stays valid.
    let _ = fs::write(bundle_dir.join(".keep"), "");

    println!("cargo:rerun-if-env-changed=PDFIUM_FORCE_DOWNLOAD");
    println!("cargo:rerun-if-changed={}", bundled.display());

    let needs_download = !bundled.exists() || env::var("PDFIUM_FORCE_DOWNLOAD").is_ok();

    if needs_download {
        let mut last_err = String::new();
        let mut got_bytes: Option<Vec<u8>> = None;
        let mut got_archive: &str = "";

        for archive_name in &candidates {
            let url = format!(
                "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/{archive_name}"
            );
            println!("cargo:warning=pdfium: trying {url}");
            match download(&url) {
                Ok(b) => {
                    got_bytes = Some(b);
                    got_archive = archive_name;
                    break;
                }
                Err(e) => {
                    last_err = format!("{archive_name}: {e}");
                }
            }
        }

        let bytes = got_bytes.ok_or_else(|| format!("all candidates failed; last: {last_err}"))?;
        let lib_bytes = extract_lib_from_tgz(&bytes, archive_match)
            .ok_or_else(|| format!("{lib_filename} not found in archive {got_archive}"))?;
        fs::write(&bundled, &lib_bytes).map_err(|e| format!("write {bundled:?}: {e}"))?;
        println!(
            "cargo:warning=pdfium: installed at {} ({} bytes from {})",
            bundled.display(),
            lib_bytes.len(),
            got_archive
        );
    }

    // Mirror to target/{profile}/ so `cargo run` finds it next to the binary.
    if let Some(target_dir) = compute_target_profile_dir() {
        let target_lib = target_dir.join(lib_filename);
        if target_dir.exists() && !target_lib.exists() {
            if let Err(e) = fs::copy(&bundled, &target_lib) {
                println!(
                    "cargo:warning=could not mirror pdfium to {}: {e}",
                    target_lib.display()
                );
            } else {
                println!("cargo:warning=pdfium: mirrored to {}", target_lib.display());
            }
        }
    }

    Ok(())
}

fn compute_target_profile_dir() -> Option<PathBuf> {
    // OUT_DIR = target/{profile}/build/{pkg}-{hash}/out → walk up 3 to get target/{profile}/
    let out_dir = env::var("OUT_DIR").ok()?;
    let p = PathBuf::from(out_dir);
    let p = p.parent()?.parent()?.parent()?.to_path_buf();
    Some(p)
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    let mut last_err = String::new();
    for attempt in 0..3u32 {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_millis(500 * (1 << attempt)));
        }
        match try_download(url) {
            Ok(b) => return Ok(b),
            Err(e) => last_err = e,
        }
    }
    Err(format!("after 3 attempts: {last_err}"))
}

fn try_download(url: &str) -> Result<Vec<u8>, String> {
    let resp = ureq::get(url).call().map_err(|e| format!("http: {e}"))?;
    if resp.status() != 200 {
        return Err(format!("http {}", resp.status()));
    }
    resp.into_body()
        .with_config()
        .limit(200 * 1024 * 1024)
        .read_to_vec()
        .map_err(|e| format!("read body: {e}"))
}

fn extract_lib_from_tgz(tgz_bytes: &[u8], lib_match: &str) -> Option<Vec<u8>> {
    let gz = flate2::read::GzDecoder::new(tgz_bytes);
    let mut archive = tar::Archive::new(gz);
    for entry in archive.entries().ok()? {
        let mut entry = entry.ok()?;
        let path_str = entry.path().ok()?.to_string_lossy().replace('\\', "/");
        if path_str.ends_with(lib_match) {
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut buf).ok()?;
            return Some(buf);
        }
    }
    None
}
