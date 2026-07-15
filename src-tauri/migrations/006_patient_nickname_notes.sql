-- 006_patient_nickname_notes.sql
-- Add a friendly label and a free-form notes field on patients. The nickname
-- shows up alongside the canonical name in tables and titles; notes are for
-- clinical context that isn't captured by any structured field (allergies,
-- relevant family history, treatment plan, etc).

ALTER TABLE patients ADD COLUMN nickname TEXT;
ALTER TABLE patients ADD COLUMN notes    TEXT;
