-- 002_analyte_descriptions.sql
-- Add clinical context fields to the analytes registry. These are populated
-- from the ontology seed (re-installed on every unlock) and surfaced on the
-- analyte detail page so users understand what each parameter measures and
-- how to interpret high / low values.

ALTER TABLE analytes ADD COLUMN description TEXT;
ALTER TABLE analytes ADD COLUMN high_means  TEXT;
ALTER TABLE analytes ADD COLUMN low_means   TEXT;
ALTER TABLE analytes ADD COLUMN unit_notes  TEXT;
