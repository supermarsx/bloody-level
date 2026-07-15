-- 003_categorical_tiers.sql
-- Categorical reference data — tier tables (Vit D, Ferritina) and cycle-phase
-- tables (Estradiol). Stored as JSON so the structure stays flexible. The UI
-- uses these to show a meaningful reference + flag when the row's printed
-- ref_low/ref_high are null (which is the case whenever the lab prints the
-- reference as a sub-table rather than a single a-b range).

ALTER TABLE analytes ADD COLUMN categorical_tiers_json TEXT;
ALTER TABLE analytes ADD COLUMN cycle_phases_json TEXT;
