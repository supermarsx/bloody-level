// Audit-trail helpers + the list command for the /audit page.
//
// Mutating commands call `audit::log(&conn, action, entity_type, entity_id,
// summary, details)` to append a single row. The helper takes a
// rusqlite::Connection (or transaction) so it integrates with whatever
// transaction the calling command already opened — failures during audit
// logging never roll back the user's actual mutation.

use rusqlite::OptionalExtension;
use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// Append-only writer. The connection can be a raw `Connection` or a
/// `Transaction` — both expose `execute`.
pub fn log(
    conn: &rusqlite::Connection,
    action: &str,
    entity_type: &str,
    entity_id: Option<&str>,
    summary: &str,
    details: Option<&Value>,
) {
    let now: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let details_json = details.map(|v| v.to_string());
    // Best-effort: if the audit insert fails we want to surface it to logs
    // but never abort the surrounding mutation. The trail is observability,
    // not durability-critical.
    if let Err(e) = conn.execute(
        "INSERT INTO audit_log(ts, action, entity_type, entity_id, summary, details_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![now, action, entity_type, entity_id, summary, details_json],
    ) {
        tracing::warn!("audit log write failed: {e}");
    }
}

#[derive(Serialize)]
pub struct AuditEntry {
    pub id: i64,
    pub ts: i64,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub summary: String,
    pub details_json: Option<String>,
}

#[derive(Serialize)]
pub struct AuditListResult {
    pub entries: Vec<AuditEntry>,
    /// Total matching rows ignoring limit/offset — drives pagination UI.
    pub total: i64,
    /// Distinct values currently present in `action` and `entity_type`,
    /// returned alongside so the filter dropdowns are populated in one
    /// round-trip.
    pub distinct_actions: Vec<String>,
    pub distinct_entity_types: Vec<String>,
}

/// Paginated + filterable audit query. Empty filters mean "no constraint".
#[tauri::command]
pub async fn list_audit_entries(
    state: State<'_, AppState>,
    action: Option<String>,
    entity_type: Option<String>,
    entity_id: Option<String>,
    text: Option<String>,
    since_ts: Option<i64>,
    until_ts: Option<i64>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> AppResult<AuditListResult> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;

    let limit  = limit.unwrap_or(100).clamp(1, 1000);
    let offset = offset.unwrap_or(0).max(0);

    // We compose dynamic WHERE clauses with bound params to keep the query
    // simple to reason about. Each clause adds (sql, value) to a small
    // accumulator that's threaded into both the rows query and the count
    // query so they stay in sync.
    let mut clauses: Vec<String> = Vec::new();
    let mut params: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(a) = action.as_deref().filter(|s| !s.is_empty()) {
        clauses.push(format!("action = ?{}", params.len() + 1));
        params.push(a.to_string().into());
    }
    if let Some(t) = entity_type.as_deref().filter(|s| !s.is_empty()) {
        clauses.push(format!("entity_type = ?{}", params.len() + 1));
        params.push(t.to_string().into());
    }
    if let Some(id) = entity_id.as_deref().filter(|s| !s.is_empty()) {
        clauses.push(format!("entity_id = ?{}", params.len() + 1));
        params.push(id.to_string().into());
    }
    if let Some(q) = text.as_deref().filter(|s| !s.trim().is_empty()) {
        let like = format!("%{}%", escape_like(q.trim()));
        clauses.push(format!(
            "(summary LIKE ?{0} ESCAPE '\\' OR details_json LIKE ?{0} ESCAPE '\\')",
            params.len() + 1
        ));
        params.push(like.into());
    }
    if let Some(s) = since_ts {
        clauses.push(format!("ts >= ?{}", params.len() + 1));
        params.push(s.into());
    }
    if let Some(u) = until_ts {
        clauses.push(format!("ts <= ?{}", params.len() + 1));
        params.push(u.into());
    }
    let where_sql = if clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", clauses.join(" AND "))
    };

    let total: i64 = {
        let sql = format!("SELECT COUNT(*) FROM audit_log{where_sql}");
        let mut stmt = db.conn.prepare(&sql)?;
        let bound = rusqlite::params_from_iter(params.iter());
        stmt.query_row(bound, |r| r.get(0))?
    };

    let entries: Vec<AuditEntry> = {
        let mut p_with_paging = params.clone();
        p_with_paging.push(limit.into());
        p_with_paging.push(offset.into());
        let sql = format!(
            "SELECT id, ts, action, entity_type, entity_id, summary, details_json
             FROM audit_log{where_sql}
             ORDER BY ts DESC, id DESC
             LIMIT ?{} OFFSET ?{}",
            params.len() + 1,
            params.len() + 2
        );
        let mut stmt = db.conn.prepare(&sql)?;
        let bound = rusqlite::params_from_iter(p_with_paging.iter());
        let rows: Vec<AuditEntry> = stmt
            .query_map(bound, |r| {
                Ok(AuditEntry {
                    id: r.get(0)?,
                    ts: r.get(1)?,
                    action: r.get(2)?,
                    entity_type: r.get(3)?,
                    entity_id: r.get(4)?,
                    summary: r.get(5)?,
                    details_json: r.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };

    let distinct_actions: Vec<String> = db
        .conn
        .prepare("SELECT DISTINCT action FROM audit_log ORDER BY action")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    let distinct_entity_types: Vec<String> = db
        .conn
        .prepare("SELECT DISTINCT entity_type FROM audit_log ORDER BY entity_type")?
        .query_map([], |r| r.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AuditListResult { entries, total, distinct_actions, distinct_entity_types })
}

#[derive(Serialize)]
pub struct AuditClearResult {
    pub deleted: i64,
}

/// Wipe the audit log. Confirmed at the UI; recorded as its own audit entry
/// (after the wipe — a single-row tombstone) so the deletion is auditable.
#[tauri::command]
pub async fn clear_audit_log(state: State<'_, AppState>) -> AppResult<AuditClearResult> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let deleted = db.conn.execute("DELETE FROM audit_log", [])? as i64;
    let after_count: Option<i64> = db
        .conn
        .query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))
        .optional()?;
    log(
        &db.conn,
        "wipe",
        "audit",
        None,
        &format!("Wiped {deleted} audit entries"),
        Some(&serde_json::json!({ "deleted": deleted, "after_count": after_count })),
    );
    Ok(AuditClearResult { deleted })
}

fn escape_like(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if matches!(ch, '\\' | '%' | '_') {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}
