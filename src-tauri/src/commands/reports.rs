use serde::Serialize;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Serialize)]
pub struct PatientSummary {
    pub id: String,
    pub display_name: String,
    pub sex: String,
    pub report_count: i64,
    pub latest_collection_date_iso: Option<String>,
    pub nickname: Option<String>,
    pub notes: Option<String>,
    pub dob_iso: Option<String>,
    pub hrt_start_iso: Option<String>,
}

#[derive(Serialize)]
pub struct ReportSummary {
    pub id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub collection_date_iso: String,
    pub doc_confidence: f64,
    pub ingest_tier: i64,
    pub row_count: i64,
    /// User-supplied friendly label (e.g. "Annual checkup"). Optional.
    pub nickname: Option<String>,
    /// Free-form per-report annotations. Optional.
    pub annotations: Option<String>,
}

#[derive(Serialize)]
pub struct AnalyteReading {
    pub date: String,
    pub value: Option<f64>,
    pub qualitative: Option<String>,
    pub unit: Option<String>,
    pub ref_low: Option<f64>,
    pub ref_high: Option<f64>,
    pub flag: Option<String>,
    pub patient_id: String,
    pub patient_name: String,
    /// 'm', 'f', or '?' — needed so the UI can pick the right side of a
    /// sex-keyed ontology default_ref when re-deriving the flag.
    pub patient_sex: String,
    pub method: Option<String>,
    pub source_report_id: String,
    /// Friendly label set by the user on the source report, if any.
    /// Used by the chart's "labels" toggle to put `Annual checkup` on
    /// the axis instead of the raw date.
    pub source_report_nickname: Option<String>,
    pub inline_prior: bool,
}

#[tauri::command]
pub async fn list_patients(state: State<'_, AppState>) -> AppResult<Vec<PatientSummary>> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let mut stmt = db.conn.prepare(
        "SELECT p.id, p.display_name, p.sex,
                COUNT(r.id) AS report_count,
                MAX(r.collection_date_iso) AS latest,
                p.nickname, p.notes, p.dob_iso, p.hrt_start_iso
         FROM patients p
         LEFT JOIN reports r ON r.patient_id = p.id
         GROUP BY p.id
         ORDER BY p.display_name COLLATE NOCASE",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(PatientSummary {
                id: r.get(0)?,
                display_name: r.get(1)?,
                sex: r.get(2)?,
                report_count: r.get(3)?,
                latest_collection_date_iso: r.get(4)?,
                nickname: r.get(5)?,
                notes: r.get(6)?,
                dob_iso: r.get(7)?,
                hrt_start_iso: r.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub async fn list_reports(
    state: State<'_, AppState>,
    patient_id: Option<String>,
) -> AppResult<Vec<ReportSummary>> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let sql = if patient_id.is_some() {
        "SELECT r.id, r.patient_id, p.display_name, r.collection_date_iso,
                r.doc_confidence, r.ingest_tier,
                (SELECT COUNT(*) FROM results res WHERE res.report_id = r.id) AS row_count,
                r.nickname, r.annotations
         FROM reports r JOIN patients p ON p.id = r.patient_id
         WHERE r.patient_id = ?1
         ORDER BY r.collection_date_iso DESC"
    } else {
        "SELECT r.id, r.patient_id, p.display_name, r.collection_date_iso,
                r.doc_confidence, r.ingest_tier,
                (SELECT COUNT(*) FROM results res WHERE res.report_id = r.id) AS row_count,
                r.nickname, r.annotations
         FROM reports r JOIN patients p ON p.id = r.patient_id
         ORDER BY r.collection_date_iso DESC"
    };

    let mut stmt = db.conn.prepare(sql)?;
    let map = |r: &rusqlite::Row<'_>| {
        Ok(ReportSummary {
            id: r.get(0)?,
            patient_id: r.get(1)?,
            patient_name: r.get(2)?,
            collection_date_iso: r.get(3)?,
            doc_confidence: r.get(4)?,
            ingest_tier: r.get(5)?,
            row_count: r.get(6)?,
            nickname: r.get(7)?,
            annotations: r.get(8)?,
        })
    };
    let rows: Vec<ReportSummary> = if let Some(pid) = patient_id {
        stmt.query_map([pid], map)?.collect::<Result<_, _>>()?
    } else {
        stmt.query_map([], map)?.collect::<Result<_, _>>()?
    };
    Ok(rows)
}

#[tauri::command]
pub async fn analyte_timeseries(
    state: State<'_, AppState>,
    analyte_id: String,
    patient_id: Option<String>,
) -> AppResult<Vec<AnalyteReading>> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let sql = if patient_id.is_some() {
        "SELECT res.collection_date_iso, res.value_numeric, res.value_qualitative,
                res.unit, res.ref_low, res.ref_high, res.flag,
                p.id, p.display_name, p.sex, res.method_annotation,
                res.report_id, r.nickname, res.inline_prior_pdf
         FROM results_canonical res
         JOIN reports r ON r.id = res.report_id
         JOIN patients p ON p.id = r.patient_id
         WHERE res.analyte_id = ?1 AND p.id = ?2
         ORDER BY res.collection_date_iso ASC"
    } else {
        "SELECT res.collection_date_iso, res.value_numeric, res.value_qualitative,
                res.unit, res.ref_low, res.ref_high, res.flag,
                p.id, p.display_name, p.sex, res.method_annotation,
                res.report_id, r.nickname, res.inline_prior_pdf
         FROM results_canonical res
         JOIN reports r ON r.id = res.report_id
         JOIN patients p ON p.id = r.patient_id
         WHERE res.analyte_id = ?1
         ORDER BY res.collection_date_iso ASC"
    };
    let mut stmt = db.conn.prepare(sql)?;
    let map = |r: &rusqlite::Row<'_>| {
        Ok(AnalyteReading {
            date: r.get(0)?,
            value: r.get(1)?,
            qualitative: r.get(2)?,
            unit: r.get(3)?,
            ref_low: r.get(4)?,
            ref_high: r.get(5)?,
            flag: r.get(6)?,
            patient_id: r.get(7)?,
            patient_name: r.get(8)?,
            patient_sex: r.get(9)?,
            method: r.get(10)?,
            source_report_id: r.get(11)?,
            source_report_nickname: r.get(12)?,
            inline_prior: r.get::<_, i64>(13)? != 0,
        })
    };
    let rows: Vec<AnalyteReading> = if let Some(pid) = patient_id {
        stmt.query_map([analyte_id, pid], map)?
            .collect::<Result<_, _>>()?
    } else {
        stmt.query_map([analyte_id], map)?
            .collect::<Result<_, _>>()?
    };
    Ok(rows)
}

#[derive(Serialize)]
pub struct FlaggedAnalytesResult {
    /// Analytes with at least one explicitly-flagged reading (low / high /
    /// critical_low / critical_high / abnormal_qual). Sorted by latest
    /// collection date desc so the top entries are most recent.
    pub abnormal: Vec<String>,
    /// Analytes whose readings are *flagged normal* but the value sits in
    /// the bottom or top decile of the printed reference range — a useful
    /// "borderline / possibly subclinical" pre-flag the lab itself didn't
    /// raise. Restricted to the latest reading per analyte to avoid old
    /// borderline-then-normalised values cluttering the picker.
    pub subclinical: Vec<String>,
}

/// Return the analyte IDs for a given patient that fall into two clinically
/// useful buckets — explicitly-flagged abnormal/critical, and "subclinical"
/// (within ~10% of either reference boundary while still flagged normal).
/// Powers the Compare-page presets so the user can populate the picker
/// with everything that matters at a glance.
#[tauri::command]
pub async fn list_flagged_analytes(
    state: State<'_, AppState>,
    patient_id: String,
) -> AppResult<FlaggedAnalytesResult> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;

    let abnormal: Vec<String> = {
        let mut stmt = db.conn.prepare(
            "SELECT res.analyte_id, MAX(res.collection_date_iso) AS latest
             FROM results res
             JOIN reports r ON r.id = res.report_id
             WHERE r.patient_id = ?1
               AND res.analyte_id IS NOT NULL
               AND res.inline_prior_pdf = 0
               AND res.flag IN ('low','high','critical_low','critical_high','abnormal_qual')
             GROUP BY res.analyte_id
             ORDER BY latest DESC",
        )?;
        let rows = stmt.query_map([&patient_id], |r| r.get::<_, String>(0))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    // Subclinical: only consider the LATEST reading per analyte; if it's
    // flagged normal AND has a printed reference range AND sits in the
    // bottom or top decile of that range, count the analyte. This catches
    // the "marching towards abnormal" pattern that the lab itself didn't
    // flag.
    let subclinical: Vec<String> = {
        let mut stmt = db.conn.prepare(
            "WITH latest AS (
                SELECT res.analyte_id,
                       res.value_numeric,
                       res.ref_low,
                       res.ref_high,
                       res.flag,
                       res.collection_date_iso,
                       ROW_NUMBER() OVER (
                           PARTITION BY res.analyte_id
                           ORDER BY res.collection_date_iso DESC, res.id DESC
                       ) AS rn
                FROM results res
                JOIN reports r ON r.id = res.report_id
                WHERE r.patient_id = ?1
                  AND res.analyte_id IS NOT NULL
                  AND res.inline_prior_pdf = 0
                  AND res.value_numeric IS NOT NULL
            )
            SELECT analyte_id
            FROM latest
            WHERE rn = 1
              AND ref_low IS NOT NULL
              AND ref_high IS NOT NULL
              AND ref_high > ref_low
              AND COALESCE(flag, 'normal') NOT IN ('low','high','critical_low','critical_high','abnormal_qual')
              AND (
                    (value_numeric - ref_low) / (ref_high - ref_low) < 0.10
                 OR (value_numeric - ref_low) / (ref_high - ref_low) > 0.90
              )
            ORDER BY collection_date_iso DESC",
        )?;
        let rows = stmt.query_map([&patient_id], |r| r.get::<_, String>(0))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    Ok(FlaggedAnalytesResult {
        abnormal,
        subclinical,
    })
}
