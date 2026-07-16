use rusqlite::OptionalExtension;
use serde::Serialize;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Serialize)]
pub struct ReportDetail {
    pub report: ReportMeta,
    pub rows: Vec<ReportRow>,
    pub parse_audit: Vec<ReportParseAudit>,
    pub unmatched_analytes: Vec<String>,
    pub stats: ReportStats,
    /// Adjacent reports for the same patient, oldest→newest, so the UI can
    /// offer prev/next navigation without an extra round-trip.
    pub prev_report_id: Option<String>,
    pub next_report_id: Option<String>,
}

#[derive(Serialize)]
pub struct ReportMeta {
    pub id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub patient_sex: String,
    pub collection_date_iso: String,
    pub emission_date_iso: Option<String>,
    pub age_at_collection: Option<i64>,
    pub lab_entity: Option<String>,
    pub requesting_physician: Option<String>,
    pub inscription_id: Option<String>,
    pub process_id: Option<String>,
    pub origin_id: Option<String>,
    pub ingest_tier: i64,
    pub parse_version: String,
    pub doc_confidence: f64,
    pub raw_pdf_path: String,
    pub source_path: String,
    /// Menstrual cycle phase at collection (female patients only).
    pub cycle_phase: Option<String>,
    /// User-supplied friendly label (e.g. "Annual checkup"). Optional.
    pub nickname: Option<String>,
    /// Free-form per-report annotations. Optional.
    pub annotations: Option<String>,
    /// Patient's HRT start date (YYYY-MM-DD). Optional. Used to compute the
    /// "Day N / Month N HRT" milestone for this report.
    pub hrt_start_iso: Option<String>,
}

#[derive(Serialize)]
pub struct ReportRow {
    pub id: i64,
    pub analyte_id: Option<String>,
    /// Canonical display name from the ontology (e.g., "TFGe" for what was
    /// printed as "TFGe [CKD-EPI 2009]"). None when analyte didn't resolve.
    pub analyte_pt_name: Option<String>,
    /// Methodology / equation pulled from the ontology (e.g., "CKD-EPI 2009").
    pub analyte_method_annotation: Option<String>,
    /// Categorical tier table (Vit D, Ferritina, …) if the analyte uses one.
    /// JSON-encoded array of `{label, min?, max?}`.
    pub analyte_categorical_tiers_json: Option<String>,
    /// Cycle-phase ref table (Estradiol female) if applicable.
    pub analyte_cycle_phases_json: Option<String>,
    /// Sex- or universal-stratified default reference range (e.g.,
    /// `{m:[13,17], f:[12,15]}` or `{all:[0,120]}`). Used as a fallback for
    /// the Ref column when the row has no printed range and no tier match.
    pub analyte_default_ref_json: Option<String>,
    /// True for analytes whose female reference depends on cycle phase
    /// (Estradiol, FSH, LH, Progesterone). Frontend uses this to prefer the
    /// broad sex-specific default_ref over phase-specific categorical_tiers
    /// for female patients when the cycle phase isn't known.
    pub analyte_cycle_dependent: bool,
    pub raw_analyte_text: String,
    pub value_numeric: Option<f64>,
    pub value_qualitative: Option<String>,
    pub value_raw_text: String,
    pub unit: Option<String>,
    pub unit_raw: String,
    pub ref_low: Option<f64>,
    pub ref_high: Option<f64>,
    pub ref_grammar: String,
    pub ref_raw_text: Option<String>,
    pub flag: Option<String>,
    pub method_annotation: Option<String>,
    pub parse_method: String,
    pub confidence: f64,
    pub inline_prior_pdf: bool,
    pub collection_date_iso: String,
}

#[derive(Serialize)]
pub struct ReportParseAudit {
    pub row_index: i64,
    pub diagnostic: String,
    pub parse_method: Option<String>,
    pub confidence: Option<f64>,
    pub llm_repaired: bool,
    pub ocr_tier: i64,
}

#[derive(Serialize)]
pub struct ReportStats {
    pub total_rows: i64,
    pub matched_rows: i64,
    pub unmatched_rows: i64,
    pub abnormal_rows: i64,
    pub critical_rows: i64,
    pub inline_prior_rows: i64,
    pub min_confidence: f64,
    pub avg_confidence: f64,
}

#[tauri::command]
pub async fn report_detail(
    state: State<'_, AppState>,
    report_id: String,
) -> AppResult<ReportDetail> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;

    let report = db
        .conn
        .query_row(
            "SELECT r.id, r.patient_id, p.display_name, p.sex,
                    r.collection_date_iso, r.emission_date_iso, r.age_at_collection,
                    r.lab_entity, r.requesting_physician, r.inscription_id,
                    r.process_id, r.origin_id, r.ingest_tier, r.parse_version,
                    r.doc_confidence, r.raw_pdf_path, r.source_path, r.cycle_phase,
                    r.nickname, r.annotations, p.hrt_start_iso
             FROM reports r JOIN patients p ON p.id = r.patient_id
             WHERE r.id = ?1",
            [&report_id],
            |row| {
                Ok(ReportMeta {
                    id: row.get(0)?,
                    patient_id: row.get(1)?,
                    patient_name: row.get(2)?,
                    patient_sex: row.get(3)?,
                    collection_date_iso: row.get(4)?,
                    emission_date_iso: row.get(5)?,
                    age_at_collection: row.get(6)?,
                    lab_entity: row.get(7)?,
                    requesting_physician: row.get(8)?,
                    inscription_id: row.get(9)?,
                    process_id: row.get(10)?,
                    origin_id: row.get(11)?,
                    ingest_tier: row.get(12)?,
                    parse_version: row.get(13)?,
                    doc_confidence: row.get(14)?,
                    raw_pdf_path: row.get(15)?,
                    source_path: row.get(16)?,
                    cycle_phase: row.get(17)?,
                    nickname: row.get(18)?,
                    annotations: row.get(19)?,
                    hrt_start_iso: row.get(20)?,
                })
            },
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("report {report_id}")))?;

    let mut stmt = db.conn.prepare(
        "SELECT res.id, res.analyte_id, a.pt_name, a.method_annotation,
                a.categorical_tiers_json, a.cycle_phases_json, a.default_ref_json,
                a.cycle_dependent,
                res.raw_analyte_text, res.value_numeric, res.value_qualitative,
                res.value_raw_text, res.unit, res.unit_raw, res.ref_low, res.ref_high,
                res.ref_grammar, res.ref_raw_text, res.flag, res.method_annotation,
                res.parse_method, res.confidence, res.inline_prior_pdf,
                res.collection_date_iso
         FROM results res
         LEFT JOIN analytes a ON a.id = res.analyte_id
         WHERE res.report_id = ?1
         ORDER BY res.inline_prior_pdf ASC, res.id ASC",
    )?;
    let rows: Vec<ReportRow> = stmt
        .query_map([&report_id], |r| {
            Ok(ReportRow {
                id: r.get(0)?,
                analyte_id: r.get(1)?,
                analyte_pt_name: r.get(2)?,
                analyte_method_annotation: r.get(3)?,
                analyte_categorical_tiers_json: r.get(4)?,
                analyte_cycle_phases_json: r.get(5)?,
                analyte_default_ref_json: r.get(6)?,
                analyte_cycle_dependent: r.get::<_, Option<i64>>(7)?.unwrap_or(0) != 0,
                raw_analyte_text: r.get(8)?,
                value_numeric: r.get(9)?,
                value_qualitative: r.get(10)?,
                value_raw_text: r.get(11)?,
                unit: r.get(12)?,
                unit_raw: r.get(13)?,
                ref_low: r.get(14)?,
                ref_high: r.get(15)?,
                ref_grammar: r.get(16)?,
                ref_raw_text: r.get(17)?,
                flag: r.get(18)?,
                method_annotation: r.get(19)?,
                parse_method: r.get(20)?,
                confidence: r.get(21)?,
                inline_prior_pdf: r.get::<_, i64>(22)? != 0,
                collection_date_iso: r.get(23)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let unmatched_analytes: Vec<String> = rows
        .iter()
        .filter(|r| r.analyte_id.is_none() && !r.inline_prior_pdf)
        .map(|r| r.raw_analyte_text.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    let parse_audit: Vec<ReportParseAudit> = db
        .conn
        .prepare(
            "SELECT row_index, diagnostic, parse_method, confidence, llm_repaired, ocr_tier
             FROM parse_audit
             WHERE report_id = ?1
             ORDER BY row_index ASC, id ASC",
        )?
        .query_map([&report_id], |row| {
            Ok(ReportParseAudit {
                row_index: row.get(0)?,
                diagnostic: row.get(1)?,
                parse_method: row.get(2)?,
                confidence: row.get(3)?,
                llm_repaired: row.get::<_, i64>(4)? != 0,
                ocr_tier: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let total_rows = rows.len() as i64;
    let matched_rows = rows.iter().filter(|r| r.analyte_id.is_some()).count() as i64;
    let unmatched_rows = total_rows - matched_rows;
    let abnormal_rows = rows
        .iter()
        .filter(|r| {
            matches!(
                r.flag.as_deref(),
                Some("low") | Some("high") | Some("abnormal_qual")
            )
        })
        .count() as i64;
    let critical_rows = rows
        .iter()
        .filter(|r| {
            matches!(
                r.flag.as_deref(),
                Some("critical_low") | Some("critical_high")
            )
        })
        .count() as i64;
    let inline_prior_rows = rows.iter().filter(|r| r.inline_prior_pdf).count() as i64;
    let min_confidence = rows
        .iter()
        .map(|r| r.confidence)
        .fold(f64::INFINITY, f64::min);
    let avg_confidence = if rows.is_empty() {
        0.0
    } else {
        rows.iter().map(|r| r.confidence).sum::<f64>() / total_rows as f64
    };

    // Find adjacent reports for the same patient by collection date. Tie-break
    // on report_id so two reports collected the same day still order stably.
    let prev_report_id: Option<String> = db
        .conn
        .query_row(
            "SELECT id FROM reports
             WHERE patient_id = ?1
               AND (collection_date_iso, id) < (?2, ?3)
             ORDER BY collection_date_iso DESC, id DESC
             LIMIT 1",
            rusqlite::params![&report.patient_id, &report.collection_date_iso, &report.id],
            |r| r.get(0),
        )
        .optional()?;
    let next_report_id: Option<String> = db
        .conn
        .query_row(
            "SELECT id FROM reports
             WHERE patient_id = ?1
               AND (collection_date_iso, id) > (?2, ?3)
             ORDER BY collection_date_iso ASC, id ASC
             LIMIT 1",
            rusqlite::params![&report.patient_id, &report.collection_date_iso, &report.id],
            |r| r.get(0),
        )
        .optional()?;

    Ok(ReportDetail {
        report,
        rows,
        parse_audit,
        unmatched_analytes,
        stats: ReportStats {
            total_rows,
            matched_rows,
            unmatched_rows,
            abnormal_rows,
            critical_rows,
            inline_prior_rows,
            min_confidence: if min_confidence.is_finite() {
                min_confidence
            } else {
                0.0
            },
            avg_confidence,
        },
        prev_report_id,
        next_report_id,
    })
}
