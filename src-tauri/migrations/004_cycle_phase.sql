-- 004_cycle_phase.sql
-- Allow tagging a female patient's report with the menstrual cycle phase that
-- was active at collection time. Used by the UI to pick the correct reference
-- range for cycle-dependent analytes (Estradiol, FSH, LH, Progesterone) from
-- the analyte ontology's `cycle_phases_json` table.
--
-- Allowed values (string, free-form so labs with custom buckets aren't blocked):
--   'follicular' | 'ovulation' | 'luteal' | 'postmenopause' | NULL (unknown)

ALTER TABLE reports ADD COLUMN cycle_phase TEXT;
