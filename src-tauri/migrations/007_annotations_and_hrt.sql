-- 007_annotations_and_hrt.sql
-- Two related additions:
--   1. Free-form annotations per report (separate from patient-level notes)
--      so users can capture context that's specific to a single draw, e.g.
--      "post-vacation, fasting violated", "first labs after starting Y med",
--      "redrawn after lab error".
--   2. A patient-level HRT (hormone-replacement therapy) start date. When
--      set, every report surfaces a milestone tag computed from
--      `(collection_date_iso - hrt_start_iso)` so users can map readings to
--      "Day 14 HRT", "Month 6 HRT", etc. Used heavily for monitoring
--      gender-affirming care, menopause therapy, post-transplant regimens.

ALTER TABLE reports  ADD COLUMN annotations  TEXT;
ALTER TABLE patients ADD COLUMN hrt_start_iso TEXT;
