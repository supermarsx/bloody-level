// Global search across patients, analytes (canonical + aliases + raw text),
// and reports (by date / source path / process id). Designed for fast typeahead
// from the navbar, so we cap each section at a small `limit`.

use rusqlite::OptionalExtension;
use serde::Serialize;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Serialize)]
pub struct SearchHit {
    pub kind: &'static str, // "patient" | "analyte" | "report"
    pub id: String,
    pub label: String,
    pub sub: Option<String>,
    /// route (relative) the frontend should navigate to.
    pub href: String,
}

#[derive(Serialize)]
pub struct SearchResults {
    pub patients: Vec<SearchHit>,
    pub analytes: Vec<SearchHit>,
    pub reports: Vec<SearchHit>,
}

#[tauri::command]
pub async fn global_search(state: State<'_, AppState>, query: String) -> AppResult<SearchResults> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(SearchResults {
            patients: vec![],
            analytes: vec![],
            reports: vec![],
        });
    }

    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;

    let like = format!("%{}%", escape_like(q));
    let limit = 8i64;

    // Patients ----------------------------------------------------------------
    let patients: Vec<SearchHit> = {
        let mut stmt = db.conn.prepare(
            "SELECT p.id, p.display_name,
                    (SELECT MAX(r.collection_date_iso) FROM reports r WHERE r.patient_id = p.id) AS latest,
                    (SELECT COUNT(*) FROM reports r WHERE r.patient_id = p.id) AS rcount
             FROM patients p
             WHERE p.display_name LIKE ?1 ESCAPE '\\' OR p.id LIKE ?1 ESCAPE '\\'
             ORDER BY p.display_name COLLATE NOCASE
             LIMIT ?2",
        )?;
        let rows: Vec<SearchHit> = stmt
            .query_map(rusqlite::params![&like, limit], |r| {
                let id: String = r.get(0)?;
                let name: String = r.get(1)?;
                let latest: Option<String> = r.get(2)?;
                let rcount: i64 = r.get(3)?;
                Ok(SearchHit {
                    kind: "patient",
                    href: format!("/patient/{id}"),
                    id,
                    label: name,
                    sub: Some(format!(
                        "{} report{} · latest {}",
                        rcount,
                        if rcount == 1 { "" } else { "s" },
                        latest.unwrap_or_else(|| "—".into())
                    )),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };

    // Analytes (canonical pt_name + aliases + raw analyte text observed) -----
    let analytes: Vec<SearchHit> = {
        let mut stmt = db.conn.prepare(
            "SELECT a.id, a.pt_name, a.section
             FROM analytes a
             WHERE a.pt_name LIKE ?1 ESCAPE '\\'
                OR a.id LIKE ?1 ESCAPE '\\'
                OR EXISTS (
                     SELECT 1 FROM analyte_aliases al
                     WHERE al.analyte_id = a.id AND al.alias LIKE ?1 ESCAPE '\\')
             ORDER BY a.pt_name COLLATE NOCASE
             LIMIT ?2",
        )?;
        let rows: Vec<SearchHit> = stmt
            .query_map(rusqlite::params![&like, limit], |r| {
                let id: String = r.get(0)?;
                let name: String = r.get(1)?;
                let section: Option<String> = r.get(2)?;
                Ok(SearchHit {
                    kind: "analyte",
                    href: format!("/analyte/{id}"),
                    id,
                    label: name,
                    sub: section,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };

    // Reports (by date / process_id / inscription_id / patient name / nickname)
    let reports: Vec<SearchHit> = {
        let mut stmt = db.conn.prepare(
            "SELECT r.id, p.display_name, r.collection_date_iso,
                    r.lab_entity, r.process_id, r.inscription_id, r.nickname
             FROM reports r JOIN patients p ON p.id = r.patient_id
             WHERE r.collection_date_iso LIKE ?1 ESCAPE '\\'
                OR p.display_name        LIKE ?1 ESCAPE '\\'
                OR r.process_id          LIKE ?1 ESCAPE '\\'
                OR r.inscription_id      LIKE ?1 ESCAPE '\\'
                OR r.id                  LIKE ?1 ESCAPE '\\'
                OR r.nickname            LIKE ?1 ESCAPE '\\'
             ORDER BY r.collection_date_iso DESC
             LIMIT ?2",
        )?;
        let rows: Vec<SearchHit> = stmt
            .query_map(rusqlite::params![&like, limit], |r| {
                let id: String = r.get(0)?;
                let name: String = r.get(1)?;
                let date: String = r.get(2)?;
                let lab: Option<String> = r.get(3)?;
                let proc_id: Option<String> = r.get(4)?;
                let insc: Option<String> = r.get(5)?;
                let nickname: Option<String> = r.get(6)?;
                let mut sub_parts: Vec<String> = vec![];
                if let Some(l) = lab {
                    sub_parts.push(l);
                }
                if let Some(p) = proc_id {
                    sub_parts.push(format!("Proc {p}"));
                }
                if let Some(i) = insc {
                    sub_parts.push(format!("Insc {i}"));
                }
                let sub = if sub_parts.is_empty() {
                    None
                } else {
                    Some(sub_parts.join(" · "))
                };
                // When the report has a nickname, lead with it — the user is far
                // more likely to remember "Annual checkup" than the date.
                let label = match nickname {
                    Some(n) => format!("{n} · {date} · {name}"),
                    None => format!("{date} · {name}"),
                };
                Ok(SearchHit {
                    kind: "report",
                    href: format!("/report/{id}"),
                    id,
                    label,
                    sub,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };

    // Defensive: probe the alias table once so a missing schema produces a
    // clean error rather than a silent empty list.
    let _ = db
        .conn
        .query_row::<i64, _, _>("SELECT COUNT(*) FROM analyte_aliases LIMIT 1", [], |r| {
            r.get(0)
        })
        .optional()?;

    Ok(SearchResults {
        patients,
        analytes,
        reports,
    })
}

/// Escape `%` and `_` so an arbitrary user query doesn't behave as a wildcard.
/// We use `\` as the escape char (matches `ESCAPE '\\'` in the SQL above).
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

#[cfg(test)]
mod tests {
    use super::escape_like;

    #[test]
    fn escapes_percent_and_underscore() {
        assert_eq!(escape_like("100%"), "100\\%");
        assert_eq!(escape_like("a_b"), "a\\_b");
        assert_eq!(escape_like("plain"), "plain");
    }

    #[test]
    fn escapes_backslash() {
        assert_eq!(escape_like("a\\b"), "a\\\\b");
    }
}
