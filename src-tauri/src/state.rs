use secrecy::SecretBox;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;

use crate::db::Database;

pub struct AppState {
    pub data_dir: PathBuf,
    pub resource_dir: Option<PathBuf>,
    pub db: Mutex<Option<Database>>,
    pub pdf_key: Mutex<Option<SecretBox<[u8; 32]>>>,
}

impl AppState {
    pub fn new(data_dir: PathBuf, resource_dir: Option<PathBuf>) -> Self {
        Self {
            data_dir,
            resource_dir,
            db: Mutex::new(None),
            pdf_key: Mutex::new(None),
        }
    }

    pub fn keystore_path(&self) -> PathBuf {
        self.data_dir.join("keystore.json")
    }

    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("data.db")
    }

    pub fn pdf_dir(&self) -> PathBuf {
        self.data_dir.join("pdfs")
    }

    pub fn models_dir(&self) -> PathBuf {
        self.data_dir.join("models")
    }

    pub fn ontology_seed_path(&self) -> Option<PathBuf> {
        let bundled = self.data_dir.join("analytes.seed.json");
        if bundled.exists() {
            return Some(bundled);
        }
        if let Some(resource_dir) = &self.resource_dir {
            let bundled = resource_dir.join("analytes.seed.json");
            if bundled.exists() {
                return Some(bundled);
            }
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let bundled = parent.join("analytes.seed.json");
                if bundled.exists() {
                    return Some(bundled);
                }
            }
        }
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let dev = manifest
            .parent()
            .map(|p| p.join("ontology").join("analytes.seed.json"));
        dev.filter(|p| p.exists())
    }
}
