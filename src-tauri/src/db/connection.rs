use std::path::Path;

use rusqlite::Connection;
use secrecy::{ExposeSecret, SecretBox};
use zeroize::Zeroizing;

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
        // Configure the cipher BEFORE applying the key. `legacy = 4` keeps the
        // existing SQLCipher v4 file format (4096-byte pages, SHA-512 HMAC/KDF).
        // SQLite silently ignores unknown pragmas, so require the cipher query
        // to succeed before allowing any database writes.
        conn.execute_batch(
            "PRAGMA cipher = 'sqlcipher';\n\
             PRAGMA legacy = 4;\n\
             PRAGMA memory_security = 1;",
        )?;
        let cipher: String = conn.query_row("PRAGMA cipher", [], |r| r.get(0))?;
        if cipher != "sqlcipher" {
            return Err(AppError::Crypto(
                "database encryption is unavailable".into(),
            ));
        }
        let raw_key = Zeroizing::new(format!("x'{}'", hex_lower(key.expose_secret()).as_str()));
        conn.pragma_update(None, "key", raw_key.as_str())?;
        // Check the key before enabling WAL, which itself reads the schema.
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |r| {
            r.get::<_, i64>(0)
        })
        .map_err(|_| AppError::Crypto("wrong key — database not unlocked".into()))?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;\n\
             PRAGMA journal_mode = WAL;\n\
             PRAGMA synchronous = NORMAL;\n\
             PRAGMA temp_store = MEMORY;",
        )?;
        Ok(Self { conn })
    }

    pub fn migrate(&mut self) -> AppResult<()> {
        super::migrations::run(&mut self.conn)
    }

    pub fn rekey(&self, key: &SecretBox<[u8; 32]>) -> AppResult<()> {
        let raw_key = Zeroizing::new(format!("x'{}'", hex_lower(key.expose_secret()).as_str()));
        self.conn.pragma_update(None, "rekey", raw_key.as_str())?;
        self.conn
            .query_row("SELECT count(*) FROM sqlite_master", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|_| AppError::Crypto("database rekey verification failed".into()))?;
        Ok(())
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

fn hex_lower(bytes: &[u8]) -> Zeroizing<String> {
    use std::fmt::Write;
    let mut s = Zeroizing::new(String::with_capacity(bytes.len() * 2));
    for b in bytes {
        let _ = write!(s, "{:02x}", b);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    const MARKER: &str = "private lab result: encryption regression marker";

    fn key() -> SecretBox<[u8; 32]> {
        SecretBox::new(Box::new([0x42; 32]))
    }

    fn write_marker(db: &Database) {
        db.conn
            .execute_batch("CREATE TABLE encryption_probe (value TEXT NOT NULL)")
            .unwrap();
        db.conn
            .execute("INSERT INTO encryption_probe VALUES (?1)", [MARKER])
            .unwrap();
    }

    fn read_marker(db: &Database) -> String {
        db.conn
            .query_row("SELECT value FROM encryption_probe", [], |r| r.get(0))
            .unwrap()
    }

    fn assert_no_plaintext(path: &Path) {
        let bytes = std::fs::read(path).unwrap();
        assert!(!bytes.is_empty());
        assert!(!bytes.starts_with(b"SQLite format 3\0"));
        assert!(!bytes.windows(MARKER.len()).any(|w| w == MARKER.as_bytes()));
    }

    #[test]
    fn encrypted_vault_migrates_and_reopens() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.db");
        let mut db = Database::open_encrypted(&path, &key()).unwrap();
        db.migrate().unwrap();
        write_marker(&db);
        assert_eq!(read_marker(&db), MARKER);
        for (pragma, expected) in [
            ("legacy", 4),
            ("page_size", 4096),
            ("kdf_iter", 256000),
            ("hmac_algorithm", 2),
            ("hmac_use", 1),
            ("hmac_check", 1),
            ("memory_security", 1),
            ("foreign_keys", 1),
            ("temp_store", 2),
            ("user_version", 9),
        ] {
            let value: rusqlite::types::Value = db
                .conn
                .pragma_query_value(None, pragma, |r| r.get(0))
                .unwrap();
            let actual = match value {
                rusqlite::types::Value::Integer(n) => n,
                rusqlite::types::Value::Text(s) => s.parse::<i64>().unwrap(),
                other => panic!("unexpected PRAGMA {pragma} value: {other:?}"),
            };
            assert_eq!(actual, expected, "PRAGMA {pragma}");
        }
        drop(db);
        assert_no_plaintext(&path);

        let mut reopened = Database::open_encrypted(&path, &key()).unwrap();
        reopened.migrate().unwrap();
        assert_eq!(read_marker(&reopened), MARKER);
    }

    #[test]
    fn wrong_or_missing_key_cannot_read_vault() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.db");
        let db = Database::open_encrypted(&path, &key()).unwrap();
        write_marker(&db);
        drop(db);
        let before = std::fs::read(&path).unwrap();

        let wrong_key = SecretBox::new(Box::new([0x24; 32]));
        assert!(matches!(
            Database::open_encrypted(&path, &wrong_key),
            Err(AppError::Crypto(_))
        ));
        let unkeyed = Connection::open(&path).unwrap();
        assert!(unkeyed
            .query_row("SELECT count(*) FROM sqlite_master", [], |r| r
                .get::<_, i64>(0))
            .is_err());
        drop(unkeyed);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(
            read_marker(&Database::open_encrypted(&path, &key()).unwrap()),
            MARKER
        );
    }

    #[test]
    fn rekey_changes_the_database_key_without_losing_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.db");
        let old_key = key();
        let new_key = SecretBox::new(Box::new([0x18; 32]));
        let db = Database::open_encrypted(&path, &old_key).unwrap();
        write_marker(&db);
        db.rekey(&new_key).unwrap();
        drop(db);

        assert!(Database::open_encrypted(&path, &old_key).is_err());
        assert_eq!(
            read_marker(&Database::open_encrypted(&path, &new_key).unwrap()),
            MARKER
        );
    }

    #[test]
    fn wal_is_encrypted_and_visible_to_another_connection() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.db");
        let db = Database::open_encrypted(&path, &key()).unwrap();
        db.conn
            .pragma_update(None, "wal_autocheckpoint", 0)
            .unwrap();
        write_marker(&db);
        assert_no_plaintext(&dir.path().join("vault.db-wal"));
        let reader = Database::open_encrypted(&path, &key()).unwrap();
        assert_eq!(read_marker(&reader), MARKER);
    }

    #[test]
    fn tampered_vault_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.db");
        let db = Database::open_encrypted(&path, &key()).unwrap();
        write_marker(&db);
        drop(db);
        let mut bytes = std::fs::read(&path).unwrap();
        bytes[100] ^= 1;
        std::fs::write(&path, &bytes).unwrap();
        assert!(Database::open_encrypted(&path, &key()).is_err());
    }

    #[test]
    fn plaintext_database_is_rejected_without_modification() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("plaintext.db");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE plaintext (value TEXT)")
            .unwrap();
        drop(conn);
        let before = std::fs::read(&path).unwrap();
        assert!(Database::open_encrypted(&path, &key()).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[test]
    fn opens_existing_sqlcipher_v4_fixture_with_raw_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("legacy.db");
        std::fs::write(
            &path,
            include_bytes!("../../tests/fixtures/sqlcipher-v4.sqlite3"),
        )
        .unwrap();
        // PBKDF2-HMAC-SHA512("testkey", fixture's first 16 bytes, 256000, 32).
        // This exercises the same raw-key path as existing application vaults.
        let fixture_key = SecretBox::new(Box::new([
            0x48, 0x53, 0x46, 0xa6, 0xb4, 0x56, 0xf2, 0xdf, 0xa2, 0xd8, 0x2a, 0x94, 0x6f, 0xe6,
            0x9e, 0xfa, 0x15, 0xa7, 0xe6, 0x74, 0xda, 0xb9, 0xa3, 0x5e, 0xc6, 0x09, 0xe2, 0xa7,
            0xb4, 0x0b, 0xcc, 0xde,
        ]));
        let mut db = Database::open_encrypted(&path, &fixture_key).unwrap();
        let count: i64 = db
            .conn
            .query_row("SELECT count(*) FROM t1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 78536);
        db.migrate().unwrap();
        write_marker(&db);
        drop(db);
        assert_eq!(
            read_marker(&Database::open_encrypted(&path, &fixture_key).unwrap()),
            MARKER
        );
    }
}
