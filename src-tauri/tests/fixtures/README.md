# SQLCipher compatibility fixture

`sqlcipher-v4.sqlite3` is the unmodified public SQLCipher 4 test database from
[SQLite3 Multiple Ciphers](https://github.com/utelle/SQLite3MultipleCiphers/blob/1d31c6c7321b9e88db0bc036523efe851f609eae/test/sqlcipher-4.0-testkey.db),
distributed under that project's [MIT license](https://github.com/utelle/SQLite3MultipleCiphers/blob/1d31c6c7321b9e88db0bc036523efe851f609eae/LICENSE).
It contains synthetic test rows, not patient data. The upstream
`test/sqlciphertest.sql` documents the password `testkey` and expected 78,536
rows in `t1`.

SHA-256: `ff3a28e3bf644cc7b543260d8b698ba72b71a59f2883bd2a8b979a3655d92128`.

The regression test derives the equivalent raw key with
PBKDF2-HMAC-SHA512(`testkey`, first 16 file bytes, 256000 iterations, 32 bytes)
and passes it through the application's normal vault opener. This checks
compatibility with independently generated SQLCipher files, including raw
keys, database reads, schema migrations, writes, and reopening.
