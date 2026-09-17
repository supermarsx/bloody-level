-- 001_initial_schema.sql
-- bloody-level — initial encrypted schema

CREATE TABLE patients (
    id              TEXT PRIMARY KEY,
    display_name    TEXT NOT NULL,
    sex             TEXT NOT NULL DEFAULT '?' CHECK (sex IN ('m','f','x','?')),
    dob_iso         TEXT,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
);

CREATE TABLE reports (
    id                      TEXT PRIMARY KEY,
    patient_id              TEXT NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
    source_path             TEXT NOT NULL,
    source_sha256           TEXT NOT NULL UNIQUE,
    collection_date_iso     TEXT NOT NULL,
    emission_date_iso       TEXT,
    age_at_collection       INTEGER,
    lab_entity              TEXT,
    requesting_physician    TEXT,
    inscription_id          TEXT,
    process_id              TEXT,
    origin_id               TEXT,
    ingest_tier             INTEGER NOT NULL,
    parse_version           TEXT NOT NULL,
    doc_confidence          REAL NOT NULL,
    raw_text                BLOB NOT NULL,
    raw_pdf_path            TEXT NOT NULL,
    created_at              INTEGER NOT NULL
);
CREATE INDEX idx_reports_patient_date ON reports(patient_id, collection_date_iso);

CREATE TABLE analytes (
    id                      TEXT PRIMARY KEY,
    pt_name                 TEXT NOT NULL,
    loinc                   TEXT,
    section                 TEXT NOT NULL,
    subsection              TEXT,
    panel                   TEXT,
    is_qualitative          INTEGER NOT NULL DEFAULT 0,
    is_derived              INTEGER NOT NULL DEFAULT 0,
    paired_value            INTEGER NOT NULL DEFAULT 0,
    is_panel_header         INTEGER NOT NULL DEFAULT 0,
    expected_units_json     TEXT NOT NULL DEFAULT '[]',
    default_ref_json        TEXT,
    sex_dependent           INTEGER NOT NULL DEFAULT 0,
    age_dependent           INTEGER NOT NULL DEFAULT 0,
    cycle_dependent         INTEGER NOT NULL DEFAULT 0,
    method_annotation       TEXT
);

CREATE TABLE analyte_aliases (
    alias       TEXT PRIMARY KEY,
    analyte_id  TEXT NOT NULL REFERENCES analytes(id) ON DELETE CASCADE,
    source      TEXT NOT NULL DEFAULT 'seed' CHECK (source IN ('seed','user'))
);

CREATE TABLE results (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    report_id               TEXT NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
    analyte_id              TEXT REFERENCES analytes(id),
    raw_analyte_text        TEXT NOT NULL,
    value_numeric           REAL,
    value_qualitative       TEXT,
    value_titer             TEXT,
    value_raw_text          TEXT NOT NULL,
    unit                    TEXT,
    unit_raw                TEXT NOT NULL,
    ref_low                 REAL,
    ref_high                REAL,
    ref_grammar             TEXT NOT NULL,
    ref_raw_text            TEXT,
    flag                    TEXT CHECK (flag IS NULL OR flag IN
                              ('low','normal','high','critical_low','critical_high','abnormal_qual','?')),
    method_annotation       TEXT,
    parse_method            TEXT NOT NULL,
    confidence              REAL NOT NULL,
    inline_prior_pdf        INTEGER NOT NULL DEFAULT 0,
    collection_date_iso     TEXT NOT NULL
);
CREATE INDEX idx_results_analyte_date ON results(analyte_id, collection_date_iso);
CREATE INDEX idx_results_report ON results(report_id);

-- Canonical view: prefer original-report rows over inline-prior duplicates.
CREATE VIEW results_canonical AS
SELECT r.*
FROM results r
JOIN reports rep ON rep.id = r.report_id
WHERE NOT EXISTS (
    SELECT 1
    FROM results r2
    JOIN reports rep2 ON rep2.id = r2.report_id
    WHERE r2.analyte_id          = r.analyte_id
      AND r2.collection_date_iso = r.collection_date_iso
      AND rep2.patient_id        = rep.patient_id
      AND r2.inline_prior_pdf    = 0
      AND r.inline_prior_pdf     = 1
);

CREATE TABLE parse_audit (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    report_id           TEXT NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
    row_index           INTEGER NOT NULL,
    diagnostic          TEXT NOT NULL,
    parse_method        TEXT,
    confidence          REAL,
    llm_repaired        INTEGER NOT NULL DEFAULT 0,
    llm_prompt_hash     TEXT,
    ocr_tier            INTEGER NOT NULL
);
CREATE INDEX idx_parse_audit_report ON parse_audit(report_id);

CREATE TABLE settings (
    key         TEXT PRIMARY KEY,
    value_json  TEXT NOT NULL
);

-- Seed defaults
INSERT INTO settings(key, value_json) VALUES
    ('theme', '"system"'),
    ('llm', '{"enabled":false,"model_path":null,"n_ctx":8192,"trigger_below_confidence":0.7}'),
    ('olmocr', '{"enabled":false,"model_path":null,"trigger_below_tesseract_confidence":0.55}'),
    ('tesseract', '{"enabled":true,"languages":["eng","por"]}'),
    ('tier_thresholds', '{"escalate_to_tesseract_below":0.4,"escalate_to_olmocr_below":0.55,"escalate_to_llm_repair_below":0.7}'),
    ('units_preference', '{"glucose":"mg/dL","cholesterol":"mg/dL"}');
