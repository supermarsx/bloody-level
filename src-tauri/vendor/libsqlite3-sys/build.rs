use std::{env, fs, path::PathBuf};

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let source = out.join("sqlite3mc");
    fs::create_dir_all(&source).expect("create SQLite source directory");
    let archive = fs::File::open("sqlite3mc.tar.gz").expect("bundled SQLite3MC source");
    tar::Archive::new(flate2::read::GzDecoder::new(archive))
        .unpack(&source)
        .expect("extract bundled SQLite3MC source");
    fs::copy("bindings.rs", out.join("bindgen.rs")).expect("copy SQLite bindings");

    println!("cargo:rerun-if-changed=sqlite3mc.tar.gz");
    println!("cargo:rerun-if-changed=bindings.rs");
    println!("cargo:include={}", source.display());

    let mut build = cc::Build::new();
    build
        .file(source.join("sqlite3.c"))
        .include(&source)
        .warnings(false);
    // Match rusqlite's bundled configuration and enable allocation clearing.
    // Temp databases must never spill decrypted values onto disk.
    for (name, value) in [
        ("SQLITE_CORE", "1"),
        ("SQLITE_THREADSAFE", "1"),
        ("SQLITE_TEMP_STORE", "3"),
        ("SQLITE_DEFAULT_FOREIGN_KEYS", "1"),
        ("SQLITE_ENABLE_API_ARMOR", "1"),
        ("SQLITE_ENABLE_COLUMN_METADATA", "1"),
        ("SQLITE_ENABLE_DBSTAT_VTAB", "1"),
        ("SQLITE_ENABLE_FTS3", "1"),
        ("SQLITE_ENABLE_FTS3_PARENTHESIS", "1"),
        ("SQLITE_ENABLE_FTS5", "1"),
        ("SQLITE_ENABLE_LOAD_EXTENSION", "1"),
        ("SQLITE_ENABLE_MEMORY_MANAGEMENT", "1"),
        ("SQLITE_ENABLE_RTREE", "1"),
        ("SQLITE_ENABLE_STAT4", "1"),
        ("SQLITE_SOUNDEX", "1"),
        ("SQLITE_USE_URI", "1"),
        ("SQLITE3MC_SECURE_MEMORY", "1"),
    ] {
        build.define(name, value);
    }
    if env::var("CARGO_CFG_TARGET_FAMILY").as_deref() == Ok("unix") {
        build.define("HAVE_USLEEP", "1");
        build.define("_POSIX_THREAD_SAFE_FUNCTIONS", None);
    }
    build.compile("sqlite3");
}
