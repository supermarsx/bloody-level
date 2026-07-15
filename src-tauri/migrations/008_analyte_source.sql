-- 008_analyte_source.sql
-- Distinguish bundled-seed rows from user-created entries on the analytes
-- table. Seed installs only UPSERT rows where source='seed' so user
-- additions/edits aren't clobbered the next time the seed is reloaded.

ALTER TABLE analytes ADD COLUMN source TEXT NOT NULL DEFAULT 'seed';
CREATE INDEX IF NOT EXISTS analytes_source_idx ON analytes(source);
