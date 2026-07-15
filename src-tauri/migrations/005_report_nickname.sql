-- 005_report_nickname.sql
-- Optional human-friendly label for a report — e.g. "Annual checkup",
-- "Pre-surgery panel", "Post-treatment 3-month follow-up". Surfaces in the
-- Records list, report detail header, and report links so users can find
-- a report without remembering its collection date.

ALTER TABLE reports ADD COLUMN nickname TEXT;
