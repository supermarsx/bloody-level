use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

use crate::commands::audit;
use crate::error::{AppError, AppResult};
use crate::parse::header;
use crate::state::AppState;

#[derive(Serialize)]
pub struct BackfillSexResult {
    pub patients_scanned: usize,
    pub patients_updated: usize,
    pub patients_still_unknown: usize,
}

/// Re-infer `patients.sex` from each patient's reports' stored raw_text.
/// Fixes legacy data ingested before the UPSERT learned to refresh `sex`.
/// Skips patients whose sex is already `m` or `f` (don't override user-set
/// or already-confident values).
#[tauri::command]
pub async fn backfill_patient_sex(state: State<'_, AppState>) -> AppResult<BackfillSexResult> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;

    let mut patients_scanned = 0usize;
    let mut patients_updated = 0usize;
    let mut patients_still_unknown = 0usize;

    let mut select = db.conn.prepare(
        "SELECT id FROM patients WHERE sex NOT IN ('m','f')",
    )?;
    let patient_ids: Vec<String> = select
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(select);

    for pid in patient_ids {
        patients_scanned += 1;

        let mut reports = db.conn.prepare(
            "SELECT raw_text FROM reports WHERE patient_id = ?1 ORDER BY collection_date_iso ASC",
        )?;
        let texts: Vec<Vec<u8>> = reports
            .query_map([&pid], |r| r.get::<_, Vec<u8>>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        drop(reports);

        let mut found: Option<&'static str> = None;
        for blob in &texts {
            if let Ok(s) = std::str::from_utf8(blob) {
                let h = header::extract(s);
                if let Some(sex) = h.inferred_sex {
                    found = Some(sex);
                    break;
                }
            }
        }

        if let Some(sex) = found {
            db.conn.execute(
                "UPDATE patients SET sex = ?1, updated_at = strftime('%s','now') WHERE id = ?2",
                rusqlite::params![sex, pid],
            )?;
            patients_updated += 1;
        } else {
            patients_still_unknown += 1;
        }
    }

    audit::log(
        &db.conn,
        "backfill",
        "patient",
        None,
        &format!(
            "Backfill sex: {patients_updated}/{patients_scanned} patients updated"
        ),
        Some(&json!({
            "scanned": patients_scanned,
            "updated": patients_updated,
            "still_unknown": patients_still_unknown,
            "field": "sex"
        })),
    );
    Ok(BackfillSexResult {
        patients_scanned,
        patients_updated,
        patients_still_unknown,
    })
}

#[derive(Serialize)]
pub struct BackfillDobResult {
    pub patients_scanned: usize,
    pub patients_updated: usize,
    pub patients_still_unknown: usize,
    /// How many of the updates were exact (literal DOB anchor in PDF) vs
    /// approximate (derived from age + collection date).
    pub exact_updates: usize,
    pub approximate_updates: usize,
}

/// Re-derive `patients.dob_iso` for every patient currently null. Reuses the
/// header parser on each patient's report raw_text — first hit with an exact
/// "Data de Nascimento" anchor wins; otherwise the highest-confidence
/// age-derived approximation across that patient's reports is taken.
#[tauri::command]
pub async fn backfill_patient_dob(state: State<'_, AppState>) -> AppResult<BackfillDobResult> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;

    let mut patients_scanned = 0usize;
    let mut patients_updated = 0usize;
    let mut patients_still_unknown = 0usize;
    let mut exact_updates = 0usize;
    let mut approximate_updates = 0usize;

    let mut select = db.conn.prepare("SELECT id FROM patients WHERE dob_iso IS NULL")?;
    let patient_ids: Vec<String> = select
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(select);

    for pid in patient_ids {
        patients_scanned += 1;

        let mut reports = db.conn.prepare(
            "SELECT raw_text FROM reports WHERE patient_id = ?1 ORDER BY collection_date_iso ASC",
        )?;
        let texts: Vec<Vec<u8>> = reports
            .query_map([&pid], |r| r.get::<_, Vec<u8>>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        drop(reports);

        // Pass 1: prefer an exact hit if any report has one.
        // Pass 2: take the first approximate value otherwise.
        let mut best: Option<(String, &'static str)> = None;
        for blob in &texts {
            if let Ok(s) = std::str::from_utf8(blob) {
                let h = header::extract(s);
                if let (Some(dob), Some(conf)) = (h.dob_iso.clone(), h.dob_confidence) {
                    if conf == "exact" {
                        best = Some((dob, "exact"));
                        break;
                    } else if best.is_none() {
                        best = Some((dob, "approximate"));
                    }
                }
            }
        }

        if let Some((dob, conf)) = best {
            db.conn.execute(
                "UPDATE patients SET dob_iso = ?1, updated_at = strftime('%s','now') WHERE id = ?2",
                rusqlite::params![dob, pid],
            )?;
            patients_updated += 1;
            if conf == "exact" {
                exact_updates += 1;
            } else {
                approximate_updates += 1;
            }
        } else {
            patients_still_unknown += 1;
        }
    }

    audit::log(
        &db.conn,
        "backfill",
        "patient",
        None,
        &format!(
            "Backfill DOB: {patients_updated}/{patients_scanned} patients updated"
        ),
        Some(&json!({
            "scanned": patients_scanned,
            "updated": patients_updated,
            "still_unknown": patients_still_unknown,
            "exact_updates": exact_updates,
            "approximate_updates": approximate_updates,
            "field": "dob_iso"
        })),
    );
    Ok(BackfillDobResult {
        patients_scanned,
        patients_updated,
        patients_still_unknown,
        exact_updates,
        approximate_updates,
    })
}

// ────────────────────────────────────────────────────────────────────────────
// Delete
// ────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn delete_report(state: State<'_, AppState>, report_id: String) -> AppResult<()> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;

    let snapshot: Option<(Option<String>, Option<String>, Option<String>, Option<String>)> = db
        .conn
        .query_row(
            "SELECT raw_pdf_path, patient_id, collection_date_iso, lab_entity FROM reports WHERE id = ?1",
            [&report_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;

    let affected = db.conn.execute("DELETE FROM reports WHERE id = ?1", [&report_id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("report {report_id}")));
    }

    let (pdf_path, patient_id, collection_date_iso, lab_entity) = snapshot
        .map(|(a, b, c, d)| (a, b, c, d))
        .unwrap_or((None, None, None, None));
    audit::log(
        &db.conn,
        "delete",
        "report",
        Some(&report_id),
        &format!(
            "Deleted report {report_id} ({})",
            collection_date_iso.as_deref().unwrap_or("?")
        ),
        Some(&json!({
            "patient_id": patient_id,
            "collection_date_iso": collection_date_iso,
            "lab_entity": lab_entity,
        })),
    );

    // Best-effort: remove cached PDF file. Failure is non-fatal — DB row is gone.
    if let Some(p) = pdf_path {
        let _ = std::fs::remove_file(p);
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct MergePatientsArgs {
    pub source_id: String,
    pub target_id: String,
}

#[derive(Serialize)]
pub struct MergePatientsResult {
    pub reports_moved: usize,
    pub source_deleted: bool,
}

#[tauri::command]
pub async fn merge_patients(
    state: State<'_, AppState>,
    args: MergePatientsArgs,
) -> AppResult<MergePatientsResult> {
    if args.source_id == args.target_id {
        return Err(AppError::BadRequest(
            "source and target are the same patient".into(),
        ));
    }
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;

    // Verify both exist before mutating.
    let target_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM patients WHERE id = ?1",
        [&args.target_id],
        |r| r.get(0),
    )?;
    if target_count == 0 {
        return Err(AppError::NotFound(format!("target patient {}", args.target_id)));
    }
    let source_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM patients WHERE id = ?1",
        [&args.source_id],
        |r| r.get(0),
    )?;
    if source_count == 0 {
        return Err(AppError::NotFound(format!("source patient {}", args.source_id)));
    }

    if !db.conn.is_autocommit() {
        let _ = db.conn.execute_batch("ROLLBACK");
    }
    let tx = db.conn.unchecked_transaction()?;
    let reports_moved = tx.execute(
        "UPDATE reports SET patient_id = ?1 WHERE patient_id = ?2",
        [&args.target_id, &args.source_id],
    )?;
    tx.execute("DELETE FROM patients WHERE id = ?1", [&args.source_id])?;
    if let Err(e) = tx.commit() {
        let _ = db.conn.execute_batch("ROLLBACK");
        return Err(AppError::from(e));
    }
    audit::log(
        &db.conn,
        "merge",
        "patient",
        Some(&args.target_id),
        &format!(
            "Merged patient {} → {} ({reports_moved} reports moved)",
            args.source_id, args.target_id
        ),
        Some(&json!({
            "source_id": args.source_id,
            "target_id": args.target_id,
            "reports_moved": reports_moved,
        })),
    );
    Ok(MergePatientsResult {
        reports_moved,
        source_deleted: true,
    })
}

#[tauri::command]
pub async fn delete_patient(state: State<'_, AppState>, patient_id: String) -> AppResult<()> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;

    let display_name: Option<String> = db
        .conn
        .query_row(
            "SELECT display_name FROM patients WHERE id = ?1",
            [&patient_id],
            |r| r.get(0),
        )
        .optional()?;

    let mut pdfs: Vec<String> = Vec::new();
    {
        let mut stmt = db
            .conn
            .prepare("SELECT raw_pdf_path FROM reports WHERE patient_id = ?1")?;
        let rows = stmt.query_map([&patient_id], |r| r.get::<_, String>(0))?;
        for r in rows {
            pdfs.push(r?);
        }
    }
    let report_count = pdfs.len();

    let affected = db.conn.execute("DELETE FROM patients WHERE id = ?1", [&patient_id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("patient {patient_id}")));
    }
    audit::log(
        &db.conn,
        "delete",
        "patient",
        Some(&patient_id),
        &format!(
            "Deleted patient {} ({report_count} reports cascaded)",
            display_name.as_deref().unwrap_or(&patient_id)
        ),
        Some(&json!({
            "display_name": display_name,
            "reports_cascaded": report_count,
        })),
    );
    for p in pdfs {
        let _ = std::fs::remove_file(p);
    }
    Ok(())
}

#[derive(Serialize)]
pub struct BulkDeleteResult {
    pub deleted: usize,
    pub failed: Vec<String>,
}

#[tauri::command]
pub async fn bulk_delete_reports(
    state: State<'_, AppState>,
    report_ids: Vec<String>,
) -> AppResult<BulkDeleteResult> {
    if report_ids.is_empty() {
        return Ok(BulkDeleteResult { deleted: 0, failed: vec![] });
    }
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;

    if !db.conn.is_autocommit() {
        let _ = db.conn.execute_batch("ROLLBACK");
    }

    // Collect the cached PDF paths first so we can remove the files after the
    // SQL delete commits.
    let mut pdfs: Vec<String> = Vec::new();
    {
        let placeholders = (1..=report_ids.len())
            .map(|i| format!("?{i}"))
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!("SELECT raw_pdf_path FROM reports WHERE id IN ({placeholders})");
        let mut stmt = db.conn.prepare(&sql)?;
        let params = rusqlite::params_from_iter(report_ids.iter());
        let rows = stmt.query_map(params, |r| r.get::<_, String>(0))?;
        for r in rows {
            pdfs.push(r?);
        }
    }

    let tx = db.conn.unchecked_transaction()?;
    let mut deleted = 0usize;
    let mut failed = Vec::new();
    for id in &report_ids {
        match tx.execute("DELETE FROM reports WHERE id = ?1", [id]) {
            Ok(n) if n > 0 => deleted += 1,
            Ok(_) => failed.push(id.clone()),
            Err(_) => failed.push(id.clone()),
        }
    }
    if let Err(e) = tx.commit() {
        let _ = db.conn.execute_batch("ROLLBACK");
        return Err(AppError::from(e));
    }

    for p in pdfs {
        let _ = std::fs::remove_file(p);
    }
    audit::log(
        &db.conn,
        "bulk_delete",
        "report",
        None,
        &format!("Bulk deleted {deleted} reports ({} failed)", failed.len()),
        Some(&json!({
            "requested": report_ids,
            "deleted": deleted,
            "failed": failed,
        })),
    );
    Ok(BulkDeleteResult { deleted, failed })
}

#[tauri::command]
pub async fn delete_result(state: State<'_, AppState>, result_id: i64) -> AppResult<()> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let snapshot: Option<(Option<String>, Option<String>, Option<f64>)> = db
        .conn
        .query_row(
            "SELECT report_id, analyte_id, value_numeric FROM results WHERE id = ?1",
            [result_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let affected = db.conn.execute("DELETE FROM results WHERE id = ?1", [result_id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("result {result_id}")));
    }
    let (report_id, analyte_id, value_numeric) = snapshot
        .map(|(a, b, c)| (a, b, c))
        .unwrap_or((None, None, None));
    audit::log(
        &db.conn,
        "delete",
        "result",
        Some(&result_id.to_string()),
        &format!("Deleted result {result_id}"),
        Some(&json!({
            "report_id": report_id,
            "analyte_id": analyte_id,
            "value_numeric": value_numeric,
        })),
    );
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// Update
// ────────────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct UpdatePatientArgs {
    pub id: String,
    pub display_name: String,
    pub sex: String,
    pub dob_iso: Option<String>,
    /// Optional friendly label (e.g. "Mom", "Dad", a known initial).
    /// `serde(default)` keeps older callers source-compatible.
    #[serde(default)]
    pub nickname: Option<String>,
    /// Optional clinical free-form notes (allergies, family hx, etc).
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct SetPatientNicknameArgs {
    pub id: String,
    pub nickname: Option<String>,
}

#[tauri::command]
pub async fn set_patient_nickname(
    state: State<'_, AppState>,
    args: SetPatientNicknameArgs,
) -> AppResult<()> {
    let cleaned: Option<String> = args
        .nickname
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let affected = db.conn.execute(
        "UPDATE patients SET nickname = ?1, updated_at = strftime('%s','now') WHERE id = ?2",
        rusqlite::params![cleaned, args.id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("patient {}", args.id)));
    }
    audit::log(
        &db.conn,
        "update",
        "patient",
        Some(&args.id),
        &match &cleaned {
            Some(n) => format!("Set nickname for {} → \"{n}\"", args.id),
            None => format!("Cleared nickname for {}", args.id),
        },
        Some(&json!({ "field": "nickname", "value": cleaned })),
    );
    Ok(())
}

#[derive(Deserialize)]
pub struct SetPatientNotesArgs {
    pub id: String,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct SetPatientHrtStartArgs {
    pub id: String,
    /// ISO YYYY-MM-DD or `None` to clear.
    pub hrt_start_iso: Option<String>,
}

/// Set a patient's HRT start date — the reference anchor every report
/// gets compared against to compute its "Day N" / "Month N" milestone.
#[tauri::command]
pub async fn set_patient_hrt_start(
    state: State<'_, AppState>,
    args: SetPatientHrtStartArgs,
) -> AppResult<()> {
    let cleaned = args.hrt_start_iso
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    // Cheap shape validation — we don't need to be a full date parser, but
    // we should reject obviously-broken values so the UI's milestone math
    // doesn't crash on Date.parse(undefined).
    if let Some(ref s) = cleaned {
        if !looks_like_iso_date(s) {
            return Err(AppError::BadRequest(
                "hrt_start_iso must be YYYY-MM-DD".into(),
            ));
        }
    }
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let affected = db.conn.execute(
        "UPDATE patients SET hrt_start_iso = ?1, updated_at = strftime('%s','now') WHERE id = ?2",
        rusqlite::params![cleaned, args.id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("patient {}", args.id)));
    }
    audit::log(
        &db.conn,
        "update",
        "patient",
        Some(&args.id),
        &match &cleaned {
            Some(d) => format!("Set HRT start for {} → {d}", args.id),
            None => format!("Cleared HRT start for {}", args.id),
        },
        Some(&json!({ "field": "hrt_start_iso", "value": cleaned })),
    );
    Ok(())
}

fn looks_like_iso_date(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() != 10 { return false; }
    bytes.iter().enumerate().all(|(i, &b)| match i {
        4 | 7 => b == b'-',
        _ => b.is_ascii_digit(),
    })
}

#[derive(Deserialize)]
pub struct SetReportAnnotationsArgs {
    pub id: String,
    pub annotations: Option<String>,
}

/// Free-form annotations per report — visible at the top of the report
/// detail page. Distinct from `nickname` (short label) and from
/// `patients.notes` (patient-level clinical context).
#[tauri::command]
pub async fn set_report_annotations(
    state: State<'_, AppState>,
    args: SetReportAnnotationsArgs,
) -> AppResult<()> {
    let cleaned = args.annotations.filter(|s| !s.trim().is_empty());
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let affected = db.conn.execute(
        "UPDATE reports SET annotations = ?1 WHERE id = ?2",
        rusqlite::params![cleaned, args.id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("report {}", args.id)));
    }
    audit::log(
        &db.conn,
        "update",
        "report",
        Some(&args.id),
        &match &cleaned {
            Some(_) => format!("Set annotations on report {}", args.id),
            None => format!("Cleared annotations on report {}", args.id),
        },
        Some(&json!({
            "field": "annotations",
            "len": cleaned.as_ref().map(|s| s.len()),
        })),
    );
    Ok(())
}

#[tauri::command]
pub async fn set_patient_notes(
    state: State<'_, AppState>,
    args: SetPatientNotesArgs,
) -> AppResult<()> {
    // Notes intentionally keep leading/trailing whitespace inside the body
    // (paragraphs etc.) but treat a fully-blank string as cleared.
    let cleaned: Option<String> = args.notes.filter(|s| !s.trim().is_empty());
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let affected = db.conn.execute(
        "UPDATE patients SET notes = ?1, updated_at = strftime('%s','now') WHERE id = ?2",
        rusqlite::params![cleaned, args.id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("patient {}", args.id)));
    }
    audit::log(
        &db.conn,
        "update",
        "patient",
        Some(&args.id),
        &match &cleaned {
            Some(_) => format!("Updated notes for {}", args.id),
            None => format!("Cleared notes for {}", args.id),
        },
        Some(&json!({
            "field": "notes",
            "len": cleaned.as_ref().map(|s| s.len()),
        })),
    );
    Ok(())
}

#[derive(Deserialize)]
pub struct CreatePatientArgs {
    pub display_name: String,
    pub sex: String,
    pub dob_iso: Option<String>,
}

#[derive(Serialize)]
pub struct CreatePatientResult {
    pub id: String,
    pub created: bool, // false if a patient with this slug already existed
}

/// Manually create a patient row (no ingested PDF). Useful for entering a
/// known patient before any reports arrive. The id is derived from the
/// display name with the same slug rule the ingest path uses, so the next
/// PDF for this patient will UPSERT against the existing row.
#[tauri::command]
pub async fn create_patient(
    state: State<'_, AppState>,
    args: CreatePatientArgs,
) -> AppResult<CreatePatientResult> {
    let trimmed = args.display_name.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest("display_name cannot be empty".into()));
    }
    if !["m", "f", "x", "?"].contains(&args.sex.as_str()) {
        return Err(AppError::BadRequest("sex must be one of m / f / x / ?".into()));
    }
    let id = patient_slug(trimmed);
    if id.is_empty() {
        return Err(AppError::BadRequest(
            "display_name produces an empty id — needs at least one alphanumeric character".into(),
        ));
    }

    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let now = now_secs();

    let existed: bool = db
        .conn
        .query_row::<i64, _, _>("SELECT 1 FROM patients WHERE id = ?1", [&id], |r| r.get(0))
        .optional()?
        .is_some();

    db.conn.execute(
        "INSERT INTO patients(id, display_name, sex, dob_iso, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)
         ON CONFLICT(id) DO UPDATE SET
            display_name = excluded.display_name,
            sex = CASE
                    WHEN excluded.sex IN ('m','f') AND patients.sex NOT IN ('m','f')
                      THEN excluded.sex
                    ELSE patients.sex
                  END,
            dob_iso = COALESCE(excluded.dob_iso, patients.dob_iso),
            updated_at = excluded.updated_at",
        rusqlite::params![id, trimmed, args.sex, args.dob_iso, now],
    )?;

    audit::log(
        &db.conn,
        if existed { "update" } else { "create" },
        "patient",
        Some(&id),
        &if existed {
            format!("Upserted existing patient {trimmed} ({id})")
        } else {
            format!("Created patient {trimmed} ({id})")
        },
        Some(&json!({
            "display_name": trimmed,
            "sex": args.sex,
            "dob_iso": args.dob_iso,
            "existed_before": existed,
        })),
    );

    Ok(CreatePatientResult { id, created: !existed })
}

/// Slug rule shared with the ingest path so manual + ingested IDs converge.
/// Strips diacritics, lowercases, replaces non-alphanumerics with `-`, and
/// collapses runs.
fn patient_slug(name: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    name.nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .collect::<String>()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[derive(Serialize)]
pub struct PatientOverview {
    pub id: String,
    pub display_name: String,
    pub nickname: Option<String>,
    pub notes: Option<String>,
    pub sex: String,
    pub dob_iso: Option<String>,
    pub report_count: i64,
    pub latest_collection_date_iso: Option<String>,
    pub earliest_collection_date_iso: Option<String>,
    pub abnormal_row_count: i64,
    pub critical_row_count: i64,
    pub distinct_analyte_count: i64,
    /// Same metrics, restricted to reports with `collection_date_iso` within
    /// the last 12 months. Drives the page-level KPI cards which prefer
    /// recency over lifetime cumulative counts.
    pub recent_report_count: i64,
    pub recent_abnormal_row_count: i64,
    pub recent_critical_row_count: i64,
}

/// Richer per-patient summary for the dedicated Patients tab — counts
/// abnormal / critical results and distinct analytes seen, alongside the
/// usual report counts and date span. Single round-trip so the page can
/// render a sortable table without N+1 queries from the frontend.
#[tauri::command]
pub async fn patient_overviews(state: State<'_, AppState>) -> AppResult<Vec<PatientOverview>> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    // 12-month cutoff in ISO date form — comparisons are lexicographic on
    // YYYY-MM-DD, which matches numeric ordering exactly.
    let cutoff_iso: String = db.conn.query_row(
        "SELECT date('now','-12 months')",
        [],
        |r| r.get(0),
    )?;

    let mut stmt = db.conn.prepare(
        "SELECT
            p.id, p.display_name, p.nickname, p.notes, p.sex, p.dob_iso,
            (SELECT COUNT(*)        FROM reports r WHERE r.patient_id = p.id) AS report_count,
            (SELECT MAX(r.collection_date_iso) FROM reports r WHERE r.patient_id = p.id) AS latest,
            (SELECT MIN(r.collection_date_iso) FROM reports r WHERE r.patient_id = p.id) AS earliest,
            (SELECT COUNT(*) FROM results res
                JOIN reports r ON r.id = res.report_id
                WHERE r.patient_id = p.id
                  AND res.flag IN ('low','high','abnormal_qual')
                  AND res.inline_prior_pdf = 0) AS abnormal_count,
            (SELECT COUNT(*) FROM results res
                JOIN reports r ON r.id = res.report_id
                WHERE r.patient_id = p.id
                  AND res.flag IN ('critical_low','critical_high')
                  AND res.inline_prior_pdf = 0) AS critical_count,
            (SELECT COUNT(DISTINCT res.analyte_id) FROM results res
                JOIN reports r ON r.id = res.report_id
                WHERE r.patient_id = p.id
                  AND res.analyte_id IS NOT NULL
                  AND res.inline_prior_pdf = 0) AS distinct_analytes,
            (SELECT COUNT(*) FROM reports r
                WHERE r.patient_id = p.id AND r.collection_date_iso >= ?1) AS recent_reports,
            (SELECT COUNT(*) FROM results res
                JOIN reports r ON r.id = res.report_id
                WHERE r.patient_id = p.id
                  AND res.flag IN ('low','high','abnormal_qual')
                  AND res.inline_prior_pdf = 0
                  AND r.collection_date_iso >= ?1) AS recent_abnormal,
            (SELECT COUNT(*) FROM results res
                JOIN reports r ON r.id = res.report_id
                WHERE r.patient_id = p.id
                  AND res.flag IN ('critical_low','critical_high')
                  AND res.inline_prior_pdf = 0
                  AND r.collection_date_iso >= ?1) AS recent_critical
         FROM patients p
         ORDER BY latest DESC NULLS LAST, p.display_name COLLATE NOCASE",
    )?;
    let rows: Vec<PatientOverview> = stmt
        .query_map([&cutoff_iso], |r| {
            Ok(PatientOverview {
                id: r.get(0)?,
                display_name: r.get(1)?,
                nickname: r.get(2)?,
                notes: r.get(3)?,
                sex: r.get(4)?,
                dob_iso: r.get(5)?,
                report_count: r.get(6)?,
                latest_collection_date_iso: r.get(7)?,
                earliest_collection_date_iso: r.get(8)?,
                abnormal_row_count: r.get(9)?,
                critical_row_count: r.get(10)?,
                distinct_analyte_count: r.get(11)?,
                recent_report_count: r.get(12)?,
                recent_abnormal_row_count: r.get(13)?,
                recent_critical_row_count: r.get(14)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub async fn update_patient(state: State<'_, AppState>, args: UpdatePatientArgs) -> AppResult<()> {
    if !["m", "f", "x", "?"].contains(&args.sex.as_str()) {
        return Err(AppError::BadRequest("sex must be one of m / f / x / ?".into()));
    }
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let now = now_secs();
    // Empty strings on optional metadata clear the column rather than
    // saving a literal "". Notes preserve internal whitespace.
    let nickname = args.nickname
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let notes = args.notes.as_ref().filter(|s| !s.trim().is_empty()).cloned();
    let affected = db.conn.execute(
        "UPDATE patients SET display_name = ?1, sex = ?2, dob_iso = ?3,
                              nickname = ?4, notes = ?5,
                              updated_at = ?6
         WHERE id = ?7",
        rusqlite::params![
            args.display_name, args.sex, args.dob_iso,
            nickname, notes, now, args.id
        ],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("patient {}", args.id)));
    }
    audit::log(
        &db.conn,
        "update",
        "patient",
        Some(&args.id),
        &format!("Updated patient {} ({})", args.display_name, args.id),
        Some(&json!({
            "display_name": args.display_name,
            "sex": args.sex,
            "dob_iso": args.dob_iso,
            "nickname": nickname,
            "notes_set": notes.is_some(),
        })),
    );
    Ok(())
}

#[derive(Deserialize)]
pub struct UpdateReportArgs {
    pub id: String,
    pub collection_date_iso: String,
    pub emission_date_iso: Option<String>,
    pub lab_entity: Option<String>,
    pub requesting_physician: Option<String>,
    /// Menstrual cycle phase at collection. Only meaningful for female
    /// patients; for male patients we still accept (and ignore) it. Free-form
    /// string so labs with custom buckets aren't blocked, but the UI
    /// constrains the selector to the canonical labels.
    #[serde(default)]
    pub cycle_phase: Option<String>,
}

#[derive(Deserialize)]
pub struct SetReportCyclePhaseArgs {
    pub id: String,
    /// `None` clears any prior tag.
    pub cycle_phase: Option<String>,
}

#[derive(Deserialize)]
pub struct SetReportNicknameArgs {
    pub id: String,
    /// `None` or empty string clears the nickname.
    pub nickname: Option<String>,
}

/// Set a friendly label for a report. The frontend stores empty strings as
/// `None` so the column round-trips cleanly between "no nickname" states.
#[tauri::command]
pub async fn set_report_nickname(
    state: State<'_, AppState>,
    args: SetReportNicknameArgs,
) -> AppResult<()> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    // Trim whitespace and treat empty as cleared so users don't end up with
    // invisible-whitespace nicknames.
    let cleaned: Option<String> = args
        .nickname
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let affected = db.conn.execute(
        "UPDATE reports SET nickname = ?1 WHERE id = ?2",
        rusqlite::params![cleaned, args.id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("report {}", args.id)));
    }
    audit::log(
        &db.conn,
        "update",
        "report",
        Some(&args.id),
        &match &cleaned {
            Some(n) => format!("Set nickname for report {} → \"{n}\"", args.id),
            None => format!("Cleared nickname for report {}", args.id),
        },
        Some(&json!({ "field": "nickname", "value": cleaned })),
    );
    Ok(())
}

/// Quick endpoint for the report-detail dropdown so the user can change
/// just the cycle phase without re-typing the entire `update_report` payload.
#[tauri::command]
pub async fn set_report_cycle_phase(
    state: State<'_, AppState>,
    args: SetReportCyclePhaseArgs,
) -> AppResult<()> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let affected = db.conn.execute(
        "UPDATE reports SET cycle_phase = ?1 WHERE id = ?2",
        rusqlite::params![args.cycle_phase, args.id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("report {}", args.id)));
    }
    audit::log(
        &db.conn,
        "update",
        "report",
        Some(&args.id),
        &match &args.cycle_phase {
            Some(p) => format!("Set cycle phase for report {} → {p}", args.id),
            None => format!("Cleared cycle phase for report {}", args.id),
        },
        Some(&json!({ "field": "cycle_phase", "value": args.cycle_phase })),
    );
    Ok(())
}

#[tauri::command]
pub async fn update_report(state: State<'_, AppState>, args: UpdateReportArgs) -> AppResult<()> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let affected = db.conn.execute(
        "UPDATE reports SET collection_date_iso = ?1, emission_date_iso = ?2,
                            lab_entity = ?3, requesting_physician = ?4,
                            cycle_phase = ?5
         WHERE id = ?6",
        rusqlite::params![
            args.collection_date_iso,
            args.emission_date_iso,
            args.lab_entity,
            args.requesting_physician,
            args.cycle_phase,
            args.id
        ],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("report {}", args.id)));
    }
    // Keep results.collection_date_iso (denormalized) in sync for primary rows
    db.conn.execute(
        "UPDATE results SET collection_date_iso = ?1 WHERE report_id = ?2 AND inline_prior_pdf = 0",
        rusqlite::params![args.collection_date_iso, args.id],
    )?;
    audit::log(
        &db.conn,
        "update",
        "report",
        Some(&args.id),
        &format!(
            "Updated report {} ({})",
            args.id, args.collection_date_iso
        ),
        Some(&json!({
            "collection_date_iso": args.collection_date_iso,
            "emission_date_iso": args.emission_date_iso,
            "lab_entity": args.lab_entity,
            "requesting_physician": args.requesting_physician,
            "cycle_phase": args.cycle_phase,
        })),
    );
    Ok(())
}

#[derive(Deserialize)]
pub struct UpdateResultArgs {
    pub id: i64,
    pub analyte_id: Option<String>,
    pub value_numeric: Option<f64>,
    pub value_qualitative: Option<String>,
    pub unit: Option<String>,
    pub ref_low: Option<f64>,
    pub ref_high: Option<f64>,
    pub flag: Option<String>,
}

#[tauri::command]
pub async fn update_result(state: State<'_, AppState>, args: UpdateResultArgs) -> AppResult<()> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let affected = db.conn.execute(
        "UPDATE results SET analyte_id = ?2, value_numeric = ?3, value_qualitative = ?4,
                            unit = ?5, ref_low = ?6, ref_high = ?7, flag = ?8
         WHERE id = ?1",
        rusqlite::params![
            args.id,
            args.analyte_id,
            args.value_numeric,
            args.value_qualitative,
            args.unit,
            args.ref_low,
            args.ref_high,
            args.flag
        ],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("result {}", args.id)));
    }
    audit::log(
        &db.conn,
        "update",
        "result",
        Some(&args.id.to_string()),
        &format!("Updated result {}", args.id),
        Some(&json!({
            "analyte_id": args.analyte_id,
            "value_numeric": args.value_numeric,
            "value_qualitative": args.value_qualitative,
            "unit": args.unit,
            "ref_low": args.ref_low,
            "ref_high": args.ref_high,
            "flag": args.flag,
        })),
    );
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// Link unmatched analyte → ontology entry (adds alias and re-links)
// ────────────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LinkAnalyteArgs {
    pub raw_text: String,
    pub analyte_id: String,
}

#[derive(Serialize)]
pub struct LinkAnalyteResult {
    pub rows_relinked: usize,
}

#[tauri::command]
pub async fn link_unmatched_analyte(
    state: State<'_, AppState>,
    args: LinkAnalyteArgs,
) -> AppResult<LinkAnalyteResult> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;

    let exists: i64 = db.conn.query_row(
        "SELECT count(*) FROM analytes WHERE id = ?1",
        [&args.analyte_id],
        |r| r.get(0),
    )?;
    if exists == 0 {
        return Err(AppError::NotFound(format!("analyte {}", args.analyte_id)));
    }

    db.conn.execute(
        "INSERT INTO analyte_aliases(alias, analyte_id, source) VALUES(?1, ?2, 'user')
         ON CONFLICT(alias) DO UPDATE SET analyte_id = excluded.analyte_id, source = 'user'",
        rusqlite::params![args.raw_text, args.analyte_id],
    )?;

    let updated = db.conn.execute(
        "UPDATE results SET analyte_id = ?1 WHERE raw_analyte_text = ?2 AND analyte_id IS NULL",
        rusqlite::params![args.analyte_id, args.raw_text],
    )?;
    audit::log(
        &db.conn,
        "link",
        "alias",
        Some(&args.analyte_id),
        &format!(
            "Linked alias \"{}\" → {} ({updated} rows relinked)",
            args.raw_text, args.analyte_id
        ),
        Some(&json!({
            "alias": args.raw_text,
            "analyte_id": args.analyte_id,
            "rows_relinked": updated,
        })),
    );
    Ok(LinkAnalyteResult { rows_relinked: updated })
}

#[derive(Serialize)]
pub struct AnalyteOption {
    pub id: String,
    pub pt_name: String,
    pub section: String,
    pub subsection: Option<String>,
    pub panel: Option<String>,
}

#[derive(Serialize)]
pub struct ReloadOntologyResult {
    pub analytes_installed: usize,
}

/// Re-installs the ontology from the bundled seed file. Useful after the
/// app gains new ontology fields (description, categorical_tiers, …) and
/// the user wants those visible without locking + unlocking.
#[tauri::command]
pub async fn reload_ontology(state: State<'_, AppState>) -> AppResult<ReloadOntologyResult> {
    let seed_path = state.ontology_seed_path();
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let n = db.ensure_ontology(seed_path.as_deref())?;
    audit::log(
        &db.conn,
        "reload",
        "ontology",
        None,
        &format!("Reloaded ontology — {n} analytes installed"),
        Some(&json!({
            "analytes_installed": n,
            "seed_path": seed_path.as_ref().map(|p| p.display().to_string()),
        })),
    );
    Ok(ReloadOntologyResult { analytes_installed: n })
}

#[tauri::command]
pub async fn list_analytes(state: State<'_, AppState>) -> AppResult<Vec<AnalyteOption>> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let mut stmt = db.conn.prepare(
        "SELECT id, pt_name, section, subsection, panel FROM analytes ORDER BY pt_name COLLATE NOCASE",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(AnalyteOption {
                id: r.get(0)?,
                pt_name: r.get(1)?,
                section: r.get(2)?,
                subsection: r.get(3)?,
                panel: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn now_secs() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
