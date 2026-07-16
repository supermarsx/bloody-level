use std::path::Path;

use rusqlite::Connection;
use secrecy::{ExposeSecret, SecretBox};

use crate::error::{AppError, AppResult};
use crate::ontology::{install_into_db, OntologySeed};

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn open_encrypted(path: &Path, key: &SecretBox<[u8; 32]>) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        let pragma = format!("PRAGMA key = \"x'{}'\";", hex_lower(key.expose_secret()));
        conn.execute_batch(&pragma)?;
        conn.execute_batch(
            "PRAGMA cipher_memory_security = ON;\n\
             PRAGMA cipher_page_size = 4096;\n\
             PRAGMA kdf_iter = 256000;\n\
             PRAGMA cipher_default_kdf_iter = 256000;\n\
             PRAGMA cipher_hmac_algorithm = HMAC_SHA512;\n\
             PRAGMA cipher_kdf_algorithm = PBKDF2_HMAC_SHA512;\n\
             PRAGMA foreign_keys = ON;\n\
             PRAGMA journal_mode = WAL;\n\
             PRAGMA synchronous = NORMAL;",
        )?;
        match conn.query_row("SELECT count(*) FROM sqlite_master", [], |r| {
            r.get::<_, i64>(0)
        }) {
            Ok(_) => Ok(Self { conn }),
            Err(_) => Err(AppError::Crypto("wrong key — database not unlocked".into())),
        }
    }

    pub fn migrate(&mut self) -> AppResult<()> {
        super::migrations::run(&mut self.conn)
    }

    pub fn ensure_ontology(&self, seed_path: Option<&Path>) -> AppResult<usize> {
        let Some(path) = seed_path else {
            return Ok(0);
        };
        if !path.exists() {
            return Ok(0);
        }
        let seed: OntologySeed = crate::ontology::load(path)?;
        match install_into_db(&seed, &self.conn) {
            Ok(n) => Ok(n),
            Err(e) => {
                // Defensive cleanup if anything left a tx open.
                if !self.conn.is_autocommit() {
                    let _ = self.conn.execute_batch("ROLLBACK");
                }
                Err(e)
            }
        }
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{:02x}", b);
    }
    s
}
