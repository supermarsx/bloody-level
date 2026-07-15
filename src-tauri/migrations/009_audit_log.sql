-- 009_audit_log.sql
-- Append-only audit trail for every mutation that changes user-visible
-- state. Each row records WHO (always 'user' for now — single-tenant
-- desktop app), WHEN (unix epoch seconds), WHAT (action verb +
-- entity_type + entity_id), and the structured details_json blob with
-- whatever per-action fields are useful for forensic / undo display.

CREATE TABLE audit_log (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    ts            INTEGER NOT NULL,            -- unix seconds
    action        TEXT    NOT NULL,            -- 'create' | 'update' | 'delete' | 'merge' | 'reload' | …
    entity_type   TEXT    NOT NULL,            -- 'patient' | 'report' | 'result' | 'analyte' | 'alias' | 'ontology' | 'system'
    entity_id     TEXT,                        -- the id of the touched row (nullable for system-wide events)
    summary       TEXT    NOT NULL,            -- short human-readable line
    details_json  TEXT                         -- per-action structured payload
);

CREATE INDEX audit_log_ts_idx          ON audit_log(ts DESC);
CREATE INDEX audit_log_entity_idx      ON audit_log(entity_type, entity_id);
CREATE INDEX audit_log_action_idx      ON audit_log(action);
