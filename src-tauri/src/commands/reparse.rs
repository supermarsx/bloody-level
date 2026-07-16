// Re-run the current parser against a report's stored raw_text and replace
// its results. Lets users benefit from parser improvements without having to
// delete + re-ingest the source PDFs.

use serde::Serialize;
use serde_json::json;
use tauri::State;

use crate::commands::{audit, parse_audit};
use crate::error::{AppError, AppResult};
use crate::parse::{
    canonical::AnalyteRegistry, preprocess, rows::parse_row, sections::SectionContext, ParsedRow,
};
use crate::state::AppState;

#[derive(Serialize)]
pub struct ReparseResult {
    pub report_id: String,
    pub patient_name: String,
    pub collection_date_iso: String,
    pub rows_before: usize,
    pub rows_after: usize,
    pub rows_unmatched: usize,
    pub inline_priors_emitted: usize,
    pub parse_audit_entries: usize,
    pub doc_confidence: f32,
    pub parse_version: &'static str,
}

#[derive(Serialize)]
pub struct ReparseBatchResult {
    pub total: usize,
    pub succeeded: usize,
    pub failed: Vec<(String, String)>,
    pub total_rows_after: usize,
    pub total_parse_audit_entries: usize,
}

const PARSE_VERSION: &str = "0.1.0";

#[tauri::command]
pub async fn reparse_report(
    state: State<'_, AppState>,
    report_id: String,
) -> AppResult<ReparseResult> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let result = reparse_one(&db.conn, &report_id)?;
    audit::log(
        &db.conn,
        "reparse",
        "report",
        Some(&report_id),
        &format!(
            "Reparsed {} ({}) — {} → {} rows",
            result.patient_name, result.collection_date_iso, result.rows_before, result.rows_after
        ),
        Some(&json!({
            "rows_before": result.rows_before,
            "rows_after": result.rows_after,
            "rows_unmatched": result.rows_unmatched,
            "inline_priors": result.inline_priors_emitted,
            "parse_audit_entries": result.parse_audit_entries,
            "doc_confidence": result.doc_confidence,
            "parse_version": result.parse_version,
        })),
    );
    Ok(result)
}

#[tauri::command]
pub async fn reparse_all_reports(state: State<'_, AppState>) -> AppResult<ReparseBatchResult> {
    // Refresh ontology BEFORE re-parsing so any new categorical_tiers /
    // descriptions / aliases the user added (or that came in a seed update)
    // are picked up by the new parse + the JOINed display fields.
    let seed_path = state.ontology_seed_path();
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let _ = db.ensure_ontology(seed_path.as_deref());

    let report_ids: Vec<String> = {
        let mut stmt = db
            .conn
            .prepare("SELECT id FROM reports ORDER BY collection_date_iso DESC")?;
        let v: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        v
    };

    let total = report_ids.len();
    let mut succeeded = 0usize;
    let mut total_rows_after = 0usize;
    let mut total_parse_audit_entries = 0usize;
    let mut failed = Vec::new();

    for id in report_ids {
        match reparse_one(&db.conn, &id) {
            Ok(r) => {
                succeeded += 1;
                total_rows_after += r.rows_after;
                total_parse_audit_entries += r.parse_audit_entries;
            }
            Err(e) => {
                failed.push((id, e.to_string()));
            }
        }
    }

    audit::log(
        &db.conn,
        "reparse_all",
        "system",
        None,
        &format!(
            "Reparsed all reports — {succeeded}/{total} succeeded, {} failed, {total_rows_after} total rows after",
            failed.len()
        ),
        Some(&json!({
            "total": total,
            "succeeded": succeeded,
            "failed_count": failed.len(),
            "total_rows_after": total_rows_after,
            "total_parse_audit_entries": total_parse_audit_entries,
            "parse_version": PARSE_VERSION,
        })),
    );
    Ok(ReparseBatchResult {
        total,
        succeeded,
        failed,
        total_rows_after,
        total_parse_audit_entries,
    })
}

fn reparse_one(conn: &rusqlite::Connection, report_id: &str) -> AppResult<ReparseResult> {
    // Fetch raw text + headline metadata.
    let (patient_name, collection_date_iso, raw_text_blob, ingest_tier): (
        String,
        String,
        Vec<u8>,
        i64,
    ) = conn
        .query_row(
            "SELECT p.display_name, r.collection_date_iso, r.raw_text, r.ingest_tier
             FROM reports r JOIN patients p ON p.id = r.patient_id
             WHERE r.id = ?1",
            [report_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("report {report_id}"))
            }
            other => AppError::Sqlite(other),
        })?;
    let raw_text = String::from_utf8(raw_text_blob)
        .map_err(|e| AppError::Internal(format!("raw_text utf-8: {e}")))?;

    let rows_before: i64 = conn.query_row(
        "SELECT COUNT(*) FROM results WHERE report_id = ?1",
        [report_id],
        |r| r.get(0),
    )?;

    // Re-parse using the current pipeline.
    let registry = AnalyteRegistry::load_from_db(conn)?;
    let normalized = preprocess::normalize(&raw_text);
    let mut ctx = SectionContext::default();
    let mut rows: Vec<ParsedRow> = Vec::new();
    for line in normalized.lines() {
        ctx.update(line);
        if let Some(row) = parse_row(line, &ctx, &registry) {
            rows.push(row);
        }
    }

    let rows_unmatched = rows.iter().filter(|r| r.analyte_id.is_none()).count();
    let inline_priors_emitted: usize = rows.iter().map(|r| r.inline_priors.len()).sum();
    let doc_confidence = if rows.is_empty() {
        0.0
    } else {
        rows.iter()
            .map(|r| r.confidence)
            .fold(f32::INFINITY, f32::min)
    };

    if !conn.is_autocommit() {
        let _ = conn.execute_batch("ROLLBACK");
    }
    let tx = conn.unchecked_transaction()?;

    tx.execute("DELETE FROM results WHERE report_id = ?1", [report_id])?;

    {
        let mut stmt = tx.prepare(
            "INSERT INTO results(
                report_id, analyte_id, raw_analyte_text, value_numeric, value_qualitative,
                value_titer, value_raw_text, unit, unit_raw, ref_low, ref_high,
                ref_grammar, ref_raw_text, flag, method_annotation, parse_method,
                confidence, inline_prior_pdf, collection_date_iso
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
        )?;
        for r in &rows {
            stmt.execute(rusqlite::params![
                report_id,
                r.analyte_id,
                r.raw_analyte_text,
                r.value.numeric,
                r.value.qualitative,
                Option::<String>::None,
                r.value.raw,
                r.unit,
                r.unit_raw,
                r.ref_low,
                r.ref_high,
                r.ref_grammar.as_str(),
                r.ref_raw_text,
                r.flag,
                r.method_annotation,
                r.parse_method,
                r.confidence as f64,
                0_i32,
                collection_date_iso
            ])?;

            for (prior_date, prior_value) in &r.inline_priors {
                stmt.execute(rusqlite::params![
                    report_id,
                    r.analyte_id,
                    r.raw_analyte_text,
                    *prior_value,
                    Option::<String>::None,
                    Option::<String>::None,
                    format!("{}", prior_value),
                    r.unit,
                    r.unit_raw,
                    r.ref_low,
                    r.ref_high,
                    r.ref_grammar.as_str(),
                    r.ref_raw_text,
                    derive_flag(*prior_value, r.ref_low, r.ref_high),
                    r.method_annotation,
                    "parse_inline_prior",
                    (r.confidence as f64) * 0.95,
                    1_i32,
                    prior_date,
                ])?;
            }
        }
    }

    let parse_audit_entries = parse_audit::replace_report_diagnostics(
        &tx,
        report_id,
        &rows,
        ingest_tier,
        parse_audit::DEFAULT_LOW_CONFIDENCE_THRESHOLD,
    )?;

    tx.execute(
        "UPDATE reports SET parse_version = ?1, doc_confidence = ?2 WHERE id = ?3",
        rusqlite::params![PARSE_VERSION, doc_confidence as f64, report_id],
    )?;

    if let Err(e) = tx.commit() {
        let _ = conn.execute_batch("ROLLBACK");
        return Err(AppError::from(e));
    }

    let rows_after_total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM results WHERE report_id = ?1",
        [report_id],
        |r| r.get(0),
    )?;

    Ok(ReparseResult {
        report_id: report_id.into(),
        patient_name,
        collection_date_iso,
        rows_before: rows_before as usize,
        rows_after: rows_after_total as usize,
        rows_unmatched,
        inline_priors_emitted,
        parse_audit_entries,
        doc_confidence,
        parse_version: PARSE_VERSION,
    })
}

fn derive_flag(v: f64, low: Option<f64>, high: Option<f64>) -> Option<&'static str> {
    match (low, high) {
        (Some(lo), Some(hi)) => {
            if v < lo {
                Some("low")
            } else if v > hi {
                Some("high")
            } else {
                Some("normal")
            }
        }
        (Some(lo), None) => {
            if v < lo {
                Some("low")
            } else {
                Some("normal")
            }
        }
        (None, Some(hi)) => {
            if v > hi {
                Some("high")
            } else {
                Some("normal")
            }
        }
        _ => None,
    }
}
