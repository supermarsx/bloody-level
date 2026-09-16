# Bundled encrypted SQLite

This local Cargo patch keeps the standard rusqlite API and migration library
while replacing SQLCipher/OpenSSL with SQLite3 Multiple Ciphers. No database
library, OpenSSL, Perl, NASM, bindgen, or download is needed to build this crate;
only the platform C compiler is required.

The application selects SQLCipher v4 compatibility before setting its raw key.
Allocation clearing is compiled in, and temporary databases stay in memory.
The database and WAL compatibility tests live in `src/db/connection.rs`.
SQLite3MC clears freed allocations but does not lock them against swapping.

## Provenance

- `src/lib.rs` and `src/error.rs`: rusqlite `libsqlite3-sys` 0.35.0, MIT.
  The unused OpenSSL linkage in `lib.rs` and unsupported loadable-extension
  initialization errors in `error.rs` are removed.
- `bindings.rs`: pregenerated SQLite3MC C API bindings from
  [rusqlite PR #1726](https://github.com/rusqlite/rusqlite/pull/1726), revision
  `fffd3e72f34cdd06ef42f18bfcf06c7e219beb75`. These are compatible with the newer
  SQLite C ABI; no libclang is required during builds.
- `sqlite3mc.tar.gz`: unmodified `sqlite3mc_amalgamation.c`,
  `sqlite3mc_amalgamation.h`, and `sqlite3ext.h` from the official
  [SQLite3 Multiple Ciphers 2.5.1 / SQLite 3.53.4 release](https://github.com/utelle/SQLite3MultipleCiphers/releases/tag/v2.5.1),
  renamed to `sqlite3.c`, `sqlite3.h`, and `sqlite3ext.h` and repacked as gzip.
  The downloaded `sqlite3mc-2.5.1-sqlite-3.53.4-amalgamation.zip` was verified
  against the release's SHA256SUMS:
  `4125f8ff275ea953dabb3289331b20a0e76d4fc060f57148f4a5df3bf3b0d5e0`.
- License notices are in `LICENSE-rusqlite`, `LICENSE-SQLite3MC`, and the
  amalgamation itself (including its bundled cryptographic implementations).

Only the Cargo features used by this application are exposed. Unsupported
features fail resolution instead of silently linking another database engine.
This is an application-local patch, not a general replacement sys crate.

## Updating

Download an official SQLite3MC amalgamation release and verify its published
checksum. Repack the same three source files, retain all license notices, and
update these version/checksum notes. If rusqlite needs additional C APIs,
regenerate the bindings against the new header. Run the vault compatibility,
wrong-key, corruption, WAL, and migration tests, then the full Rust checks and
platform build matrix. Confirm `cargo tree --locked --all-features --target all`
contains neither OpenSSL nor native-tls.
