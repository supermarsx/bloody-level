use std::path::PathBuf;
use std::time::Instant;

use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Emitter, State};

use crate::commands::{audit, parse_audit};
use crate::error::{AppError, AppErrorPayload, AppResult, ContextExt};
use crate::parse::{
    canonical::AnalyteRegistry, header, preprocess, rows::parse_row, sections::SectionContext,
    ParsedRow,
};
use crate::pdf;
use crate::state::AppState;

const EVT_PROGRESS: &str = "ingest:progress";
const EVT_BATCH: &str = "ingest:batch";

const STAGE_STARTED: &str = "started";
const STAGE_HASHING: &str = "hashing";
const STAGE_DUPLICATE: &str = "duplicate";
const STAGE_EXTRACTING: &str = "extracting";
const STAGE_EXTRACTED: &str = "extracted";
const STAGE_HEADER: &str = "parsing_header";
const STAGE_PARSING: &str = "parsing_rows";
const STAGE_WRITING: &str = "writing";
const STAGE_DONE: &str = "completed";
const STAGE_ERROR: &str = "error";

#[derive(Serialize, Clone)]
pub struct IngestProgress {
    pub path: String,
    pub stage: &'static str,
    pub progress: f32, // 0.0 .. 1.0 within this file
    pub message: Option<String>,
    pub elapsed_ms: u64,
    pub pages: Option<usize>,
    pub bytes: Option<usize>,
    pub rows_parsed: Option<usize>,
    pub rows_unmatched: Option<usize>,
    pub inline_priors: Option<usize>,
    pub doc_confidence: Option<f32>,
    pub already_ingested: Option<bool>,
    pub error: Option<AppErrorPayload>,
}

impl IngestProgress {
    fn new(path: &str, stage: &'static str, progress: f32, started: Instant) -> Self {
        Self {
            path: path.to_string(),
            stage,
            progress,
            message: None,
            elapsed_ms: started.elapsed().as_millis() as u64,
            pages: None,
            bytes: None,
            rows_parsed: None,
            rows_unmatched: None,
            inline_priors: None,
            doc_confidence: None,
            already_ingested: None,
            error: None,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct BatchProgress {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub current_path: Option<String>,
    pub elapsed_ms: u64,
}

#[derive(Serialize)]
pub struct IngestResult {
    pub report_id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub collection_date_iso: String,
    pub rows_parsed: usize,
    pub rows_unmatched: usize,
    pub inline_priors_emitted: usize,
    pub doc_confidence: f32,
    pub already_ingested: bool,
}

#[tauri::command]
pub async fn ingest_pdf(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> AppResult<IngestResult> {
    let started = Instant::now();
    match ingest_one(&app, &state, &path, started).await {
        Ok(r) => Ok(r),
        Err(e) => {
            let mut p = IngestProgress::new(&path, STAGE_ERROR, 1.0, started);
            p.error = Some(e.to_payload());
            let _ = app.emit(EVT_PROGRESS, &p);
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn ingest_pdfs(
    app: AppHandle,
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> AppResult<Vec<IngestOutcome>> {
    let total = paths.len();
    let batch_started = Instant::now();
    let mut completed = 0usize;
    let mut failed = 0usize;
    let mut outcomes: Vec<IngestOutcome> = Vec::with_capacity(total);

    for (i, path) in paths.iter().enumerate() {
        let _ = app.emit(
            EVT_BATCH,
            &BatchProgress {
                total,
                completed,
                failed,
                current_path: Some(path.clone()),
                elapsed_ms: batch_started.elapsed().as_millis() as u64,
            },
        );

        let started = Instant::now();
        match ingest_one(&app, &state, path, started).await {
            Ok(r) => {
                completed += 1;
                outcomes.push(IngestOutcome {
                    path: path.clone(),
                    index: i,
                    ok: true,
                    error: None,
                    result: Some(r),
                });
            }
            Err(e) => {
                failed += 1;
                let payload = e.to_payload();
                let mut p = IngestProgress::new(path, STAGE_ERROR, 1.0, started);
                p.error = Some(payload.clone());
                let _ = app.emit(EVT_PROGRESS, &p);
                outcomes.push(IngestOutcome {
                    path: path.clone(),
                    index: i,
                    ok: false,
                    error: Some(payload),
                    result: None,
                });
            }
        }
    }

    let _ = app.emit(
        EVT_BATCH,
        &BatchProgress {
            total,
            completed,
            failed,
            current_path: None,
            elapsed_ms: batch_started.elapsed().as_millis() as u64,
        },
    );
    Ok(outcomes)
}

#[derive(Serialize)]
pub struct IngestOutcome {
    pub path: String,
    pub index: usize,
    pub ok: bool,
    pub error: Option<AppErrorPayload>,
    pub result: Option<IngestResult>,
}

async fn ingest_one(
    app: &AppHandle,
    state: &State<'_, AppState>,
    path_str: &str,
    started: Instant,
) -> AppResult<IngestResult> {
    let pdf_path = PathBuf::from(path_str);
    if !pdf_path.exists() {
        return Err(AppError::NotFound(format!("file not found: {path_str}")))
            .stage("opening_file")
            .path(path_str)
            .hint("Verify the path is reachable and the file wasn't moved or deleted.");
    }

    emit_simple(app, path_str, STAGE_STARTED, 0.02, started);

    // ── Hashing + extraction (10% → 40%) ──────────────────────────────────
    emit_simple(app, path_str, STAGE_HASHING, 0.05, started);
    // pdfium is FFI; an unexpected library issue can panic. spawn_blocking
    // turns a panic into a typed error rather than killing the IPC channel.
    let pdf_path_for_blocking = pdf_path.clone();
    let mut extracted = tokio::task::spawn_blocking(move || pdf::extract(&pdf_path_for_blocking))
        .await
        .map_err(|join_err| {
            if join_err.is_panic() {
                AppError::Pdf(format!(
                    "PDF extraction panicked: {:?}",
                    join_err.into_panic()
                ))
            } else {
                AppError::Internal(format!("pdf task: {join_err}"))
            }
        })?
        .stage("extracting_pdf")
        .path(path_str)
        .hint("Confirm the file is a valid PDF (not corrupted or password-protected).")
        .hint(
            "If pdfium reports it's missing, rebuild — `build.rs` downloads it on first build.",
        )?;

    {
        let guard = state.db.lock().await;
        let db = guard.as_ref().ok_or(AppError::Locked)?;
        if let Ok(existing) = db.conn.query_row(
            "SELECT id FROM reports WHERE source_sha256 = ?1",
            [&extracted.sha256_hex],
            |r| r.get::<_, String>(0),
        ) {
            let (patient_id, patient_name): (String, String) = db.conn.query_row(
                "SELECT p.id, p.display_name FROM patients p
                 JOIN reports r ON r.patient_id = p.id WHERE r.id = ?1",
                [&existing],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            let collection_date_iso: String = db.conn.query_row(
                "SELECT collection_date_iso FROM reports WHERE id = ?1",
                [&existing],
                |r| r.get(0),
            )?;

            audit::log(
                &db.conn,
                "duplicate",
                "report",
                Some(&existing),
                &format!("Skipped duplicate ingest of {existing} ({patient_name})"),
                Some(&json!({
                    "source_path": path_str,
                    "sha256": extracted.sha256_hex,
                })),
            );

            let mut p = IngestProgress::new(path_str, STAGE_DUPLICATE, 1.0, started);
            p.message = Some(format!("already ingested as {existing}"));
            p.already_ingested = Some(true);
            let _ = app.emit(EVT_PROGRESS, &p);

            return Ok(IngestResult {
                report_id: existing,
                patient_id,
                patient_name,
                collection_date_iso,
                rows_parsed: 0,
                rows_unmatched: 0,
                inline_priors_emitted: 0,
                doc_confidence: 1.0,
                already_ingested: true,
            });
        }
    }

    {
        let mut p = IngestProgress::new(path_str, STAGE_EXTRACTING, 0.20, started);
        p.bytes = Some(extracted.byte_count);
        p.pages = Some(extracted.page_count);
        let _ = app.emit(EVT_PROGRESS, &p);
    }

    let mut ingest_tier = 1_i64;
    let mut ocr_mean_confidence: Option<f32> = None;
    let tesseract_setting = read_json_setting(state, "tesseract")
        .await
        .stage("loading_tesseract_settings")
        .path(path_str)?;
    let tesseract_config = crate::ocr::config_from_settings(&tesseract_setting);

    if crate::ocr::should_fallback_to_ocr(&extracted.combined_text, extracted.page_count) {
        if tesseract_config.enabled && crate::ocr::is_available() {
            {
                let mut p = IngestProgress::new(path_str, STAGE_EXTRACTING, 0.30, started);
                p.bytes = Some(extracted.byte_count);
                p.pages = Some(extracted.page_count);
                p.message = Some("embedded PDF text is sparse; running Tesseract OCR".to_string());
                let _ = app.emit(EVT_PROGRESS, &p);
            }

            let pdf_path_for_ocr = pdf_path.clone();
            let ocr_config_for_blocking = tesseract_config.clone();
            let ocr_output = tokio::task::spawn_blocking(move || {
                let pages = pdf::render_pages_for_ocr(&pdf_path_for_ocr)?;
                crate::ocr::recognize_pages(&pages, &ocr_config_for_blocking)
            })
            .await
            .map_err(|join_err| {
                if join_err.is_panic() {
                    AppError::Pdf(format!(
                        "Tesseract OCR task panicked: {:?}",
                        join_err.into_panic()
                    ))
                } else {
                    AppError::Internal(format!("tesseract OCR task: {join_err}"))
                }
            })?
            .stage("tesseract_ocr")
            .path(path_str)
            .hint("Tesseract OCR requires the tesseract executable and tessdata for the configured languages at runtime.")?;

            if !ocr_output.text.trim().is_empty() {
                extracted.combined_text = ocr_output.text;
                ingest_tier = 2;
                ocr_mean_confidence = ocr_output.mean_confidence;

                let mut p = IngestProgress::new(path_str, STAGE_EXTRACTING, 0.38, started);
                p.bytes = Some(extracted.byte_count);
                p.pages = Some(extracted.page_count);
                p.doc_confidence = ocr_mean_confidence;
                p.message = Some(format!(
                    "Tesseract OCR extracted {} chars from {} pages{}",
                    extracted.combined_text.len(),
                    ocr_output.page_count,
                    format_confidence_suffix(ocr_mean_confidence)
                ));
                let _ = app.emit(EVT_PROGRESS, &p);
            } else {
                tracing::warn!(
                    "Tesseract OCR returned no text for sparse-text PDF {}",
                    path_str
                );
            }
        } else {
            let mut p = IngestProgress::new(path_str, STAGE_EXTRACTING, 0.30, started);
            p.bytes = Some(extracted.byte_count);
            p.pages = Some(extracted.page_count);
            p.message = Some(
                "embedded PDF text is sparse; Tesseract OCR is not compiled or enabled".to_string(),
            );
            let _ = app.emit(EVT_PROGRESS, &p);
        }
    }

    {
        let mut p = IngestProgress::new(path_str, STAGE_EXTRACTED, 0.40, started);
        p.bytes = Some(extracted.byte_count);
        p.pages = Some(extracted.page_count);
        p.message = Some(format!(
            "{} pages, {} chars{}",
            extracted.page_count,
            extracted.combined_text.len(),
            if ingest_tier == 2 {
                " (Tesseract OCR)"
            } else {
                ""
            }
        ));
        let _ = app.emit(EVT_PROGRESS, &p);
    }

    // ── Header extraction (45%) ───────────────────────────────────────────
    emit_simple(app, path_str, STAGE_HEADER, 0.45, started);
    // Run inside spawn_blocking so any panic in the regex / slice path becomes
    // a typed error rather than killing the tokio worker thread. We've fixed
    // the known UTF-8 boundary panics, but defense-in-depth means a future
    // bug is recoverable.
    let header_text = extracted.combined_text.clone();
    let header = tokio::task::spawn_blocking(move || header::extract(&header_text))
        .await
        .map_err(|join_err| {
            if join_err.is_panic() {
                AppError::Internal(format!(
                    "header extraction panicked: {:?}",
                    join_err.into_panic()
                ))
            } else {
                AppError::Internal(format!("header task: {join_err}"))
            }
        })
        .stage("parsing_header")
        .path(path_str)?;
    let collection_date_iso = header
        .collection_date_iso
        .clone()
        .ok_or_else(|| AppError::BadRequest("could not extract collection date".into()))
        .stage("parsing_header")
        .path(path_str)
        .hint("Expected pattern: 'Data de colheita DD/MM/YYYY' (CUF / Germano de Sousa format).")
        .hint("If this is a different lab format, you may need a custom header parser.")?;
    let patient_name = header
        .patient_name
        .clone()
        .ok_or_else(|| AppError::BadRequest("could not extract patient name".into()))
        .stage("parsing_header")
        .path(path_str)
        .hint(
            "Expected the patient name on the line immediately after 'Exmo Sr.' / 'Exma Sra.'.",
        )?;
    let patient_id = patient_slug(&patient_name);

    // ── Row parsing with incremental progress (50% → 85%) ─────────────────
    let registry = {
        let guard = state.db.lock().await;
        let db = guard
            .as_ref()
            .ok_or(AppError::Locked)
            .stage("loading_registry")?;
        AnalyteRegistry::load_from_db(&db.conn)
            .stage("loading_registry")
            .hint("If the analytes table is empty, restart the app to re-seed the ontology.")?
    };

    // Stitch up unit-line breaks and continuation fragments before the
    // line-by-line parse loop. This is what lets leucograma rows whose unit
    // was split across PDF lines (e.g., `x 103\n/ 40.00 - 80.00`) parse with
    // a real reference range instead of falling through to Unparsed.
    // catch_unwind protects the worker if a future regex/slice bug panics on
    // unusual PDF text — the whole ingest fails cleanly rather than dying.
    let raw_text_for_preprocess = extracted.combined_text.clone();
    let normalized_text = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        preprocess::normalize(&raw_text_for_preprocess)
    }))
    .map_err(|panic| AppError::Internal(format!("preprocess panicked: {}", describe_panic(&panic))))
    .stage("preprocess")
    .path(path_str)?;
    let lines: Vec<&str> = normalized_text.lines().collect();
    let total_lines = lines.len().max(1);
    let mut ctx = SectionContext::default();
    let mut rows: Vec<ParsedRow> = Vec::new();
    let mut row_panics: usize = 0;
    let report_size_emit_every = (total_lines / 12).max(50);

    for (i, line) in lines.iter().enumerate() {
        ctx.update(line);
        // Per-line catch_unwind: one bad line can't kill the whole ingest.
        let parsed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            parse_row(line, &ctx, &registry)
        }));
        match parsed {
            Ok(Some(row)) => rows.push(row),
            Ok(None) => {}
            Err(panic) => {
                row_panics += 1;
                tracing::warn!(
                    "parse_row panicked on line {} ({}): {}",
                    i,
                    line,
                    describe_panic(&panic)
                );
            }
        }
        if i % report_size_emit_every == 0 {
            let progress = 0.50 + 0.35 * (i as f32 / total_lines as f32);
            let mut p = IngestProgress::new(path_str, STAGE_PARSING, progress, started);
            p.rows_parsed = Some(rows.len());
            p.message = Some(format!("line {} / {}", i, total_lines));
            let _ = app.emit(EVT_PROGRESS, &p);
        }
    }
    if row_panics > 0 {
        tracing::warn!(
            "ingest of {} skipped {} line(s) due to parser panics",
            path_str,
            row_panics
        );
    }

    let rows_parsed = rows.len();
    let rows_unmatched = rows.iter().filter(|r| r.analyte_id.is_none()).count();
    let inline_priors_emitted: usize = rows.iter().map(|r| r.inline_priors.len()).sum();
    let doc_confidence = if rows.is_empty() {
        0.0
    } else {
        rows.iter()
            .map(|r| r.confidence)
            .fold(f32::INFINITY, f32::min)
    };

    {
        let mut p = IngestProgress::new(path_str, STAGE_PARSING, 0.85, started);
        p.rows_parsed = Some(rows_parsed);
        p.rows_unmatched = Some(rows_unmatched);
        p.inline_priors = Some(inline_priors_emitted);
        p.doc_confidence = Some(doc_confidence);
        p.message = Some(format!(
            "{} rows ({} matched, {} unmatched), {} priors",
            rows_parsed,
            rows_parsed - rows_unmatched,
            rows_unmatched,
            inline_priors_emitted
        ));
        let _ = app.emit(EVT_PROGRESS, &p);
    }

    // ── DB write (85% → 99%) ──────────────────────────────────────────────
    let report_id = format!("{}-{}", collection_date_iso, &extracted.sha256_hex[..8]);
    let now = now_secs();

    let pdf_dest_dir = state.pdf_dir();
    std::fs::create_dir_all(&pdf_dest_dir)?;
    let pdf_dest = pdf_dest_dir.join(format!("{}.pdf", extracted.sha256_hex));
    std::fs::copy(&pdf_path, &pdf_dest)?;
    let pdf_dest_str = pdf_dest.to_string_lossy().to_string();

    let raw_text_blob = extracted.combined_text.clone().into_bytes();
    let inferred_sex = header.inferred_sex.unwrap_or("?");
    let age = header.age_years.map(|a| a as i64);
    let inferred_dob = header.dob_iso.clone();
    let inferred_dob_is_exact = header.dob_confidence == Some("exact");
    let h_clinic = header.clinic_entity.clone();
    let h_phys = header.requesting_physician.clone();
    let h_insc = header.inscription_id.clone();
    let h_proc = header.process_id.clone();
    let h_origin = header.origin_id.clone();
    let h_emit = header.emission_date_iso.clone();

    emit_simple(app, path_str, STAGE_WRITING, 0.90, started);

    let guard = state.db.lock().await;
    let db = guard
        .as_ref()
        .ok_or(AppError::Locked)
        .stage("writing_db")
        .path(path_str)?;

    // Defensive: clear any leaked transaction state from a prior failure.
    // `unchecked_transaction` will fail with "cannot start a transaction within
    // a transaction" if the connection still has one open from somewhere else.
    if !db.conn.is_autocommit() {
        let _ = db.conn.execute_batch("ROLLBACK");
        tracing::warn!("rolled back leaked transaction before ingest write");
    }

    let tx = db
        .conn
        .unchecked_transaction()
        .map_err(AppError::from)
        .stage("writing_db")
        .path(path_str)
        .patient(&patient_name)?;

    // The ON CONFLICT clause used to skip `sex` entirely, which meant the very
    // first ingest's inferred_sex was sticky forever. If that first PDF lacked
    // a recognizable salutation we'd store `?` and every later flag derivation
    // for this patient would fall through to the (often wrong) parser-stored
    // flag because `defaultRefFor()` returns null for `?`. Now: only refresh
    // `sex` when the new value is more specific than what's stored. A
    // user-set `m`/`f` is preserved against a future ambiguous ingest.
    // DOB seed precedence (per UPSERT below):
    //   - If `patients.dob_iso` is null → take whatever the new ingest gave.
    //   - If existing is set and the new value is `exact` → upgrade to it
    //     (a literal "Data de Nascimento" line beats an age-derived guess).
    //   - Otherwise keep the existing value (don't downgrade exact → approx,
    //     and don't churn the column on every ingest).
    //
    // We pass `inferred_dob_is_exact` as a separate bind param because the
    // CASE expression needs to know the new row's confidence level.
    tx.execute(
        "INSERT INTO patients(id, display_name, sex, dob_iso, created_at, updated_at)
         VALUES(?1, ?2, ?3, ?4, ?6, ?6)
         ON CONFLICT(id) DO UPDATE SET
            display_name = excluded.display_name,
            sex = CASE
                    WHEN excluded.sex IN ('m','f') AND patients.sex NOT IN ('m','f')
                      THEN excluded.sex
                    ELSE patients.sex
                  END,
            dob_iso = CASE
                        WHEN patients.dob_iso IS NULL
                          THEN excluded.dob_iso
                        WHEN ?5 = 1
                          THEN excluded.dob_iso
                        ELSE patients.dob_iso
                      END,
            updated_at = excluded.updated_at",
        rusqlite::params![
            patient_id,
            patient_name,
            inferred_sex,
            inferred_dob,
            inferred_dob_is_exact as i32,
            now
        ],
    )?;

    tx.execute(
        "INSERT INTO reports(id, patient_id, source_path, source_sha256, collection_date_iso,
                emission_date_iso, age_at_collection, lab_entity, requesting_physician,
                inscription_id, process_id, origin_id, ingest_tier, parse_version,
                doc_confidence, raw_text, raw_pdf_path, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        rusqlite::params![
            report_id,
            patient_id,
            pdf_path.to_string_lossy().to_string(),
            extracted.sha256_hex,
            collection_date_iso,
            h_emit,
            age,
            h_clinic,
            h_phys,
            h_insc,
            h_proc,
            h_origin,
            ingest_tier,
            "0.1.0",
            doc_confidence as f64,
            raw_text_blob,
            pdf_dest_str,
            now
        ],
    )?;

    {
        let mut stmt = tx.prepare(
            "INSERT INTO results(
                report_id, analyte_id, raw_analyte_text, value_numeric, value_qualitative,
                value_titer, value_raw_text, unit, unit_raw, ref_low, ref_high,
                ref_grammar, ref_raw_text, flag, method_annotation, parse_method,
                confidence, inline_prior_pdf, collection_date_iso
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
        )?;
        let total_rows = rows.len();
        for (idx, r) in rows.iter().enumerate() {
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

            if total_rows > 0 && idx % 32 == 0 {
                let progress = 0.90 + 0.09 * (idx as f32 / total_rows as f32);
                emit_simple(app, path_str, STAGE_WRITING, progress, started);
            }
        }
    }

    let parse_audit_entries = parse_audit::replace_report_diagnostics(
        &tx,
        &report_id,
        &rows,
        ingest_tier,
        parse_audit::DEFAULT_LOW_CONFIDENCE_THRESHOLD,
    )?;

    // Explicit rollback on commit failure: if commit fails, the Transaction is
    // consumed and Drop won't run, so we must clean up manually so the next
    // ingest call doesn't see a leaked transaction.
    let conn_ref = &db.conn;
    if let Err(e) = tx.commit() {
        let _ = conn_ref.execute_batch("ROLLBACK");
        return Err(AppError::from(e))
            .stage("writing_db")
            .path(path_str)
            .patient(&patient_name)
            .hint("Commit failed; transaction was rolled back and DB is clean.");
    }

    audit_model_tier_escalation_candidates(&db.conn, &report_id, path_str, doc_confidence);

    audit::log(
        &db.conn,
        "ingest",
        "report",
        Some(&report_id),
        &format!(
            "Ingested {patient_name} ({collection_date_iso}) — {rows_parsed} rows, {rows_unmatched} unmatched"
        ),
        Some(&json!({
            "patient_id": patient_id,
            "patient_name": patient_name,
            "collection_date_iso": collection_date_iso,
            "rows_parsed": rows_parsed,
            "rows_unmatched": rows_unmatched,
            "inline_priors": inline_priors_emitted,
            "doc_confidence": doc_confidence,
            "ingest_tier": ingest_tier,
            "ocr_mean_confidence": ocr_mean_confidence,
            "parse_audit_entries": parse_audit_entries,
            "source_path": path_str,
            "sha256": extracted.sha256_hex,
        })),
    );

    {
        let mut p = IngestProgress::new(path_str, STAGE_DONE, 1.0, started);
        p.rows_parsed = Some(rows_parsed);
        p.rows_unmatched = Some(rows_unmatched);
        p.inline_priors = Some(inline_priors_emitted);
        p.doc_confidence = Some(doc_confidence);
        p.already_ingested = Some(false);
        p.message = Some(format!("ingested as {report_id}"));
        let _ = app.emit(EVT_PROGRESS, &p);
    }

    Ok(IngestResult {
        report_id,
        patient_id,
        patient_name,
        collection_date_iso,
        rows_parsed,
        rows_unmatched,
        inline_priors_emitted,
        doc_confidence,
        already_ingested: false,
    })
}

fn emit_simple(app: &AppHandle, path: &str, stage: &'static str, progress: f32, started: Instant) {
    let p = IngestProgress::new(path, stage, progress, started);
    let _ = app.emit(EVT_PROGRESS, &p);
}

async fn read_json_setting(state: &State<'_, AppState>, key: &str) -> AppResult<serde_json::Value> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let raw: Option<String> = db
        .conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            [key],
            |r| r.get::<_, String>(0),
        )
        .ok();
    match raw {
        Some(s) => Ok(serde_json::from_str(&s)?),
        None => Ok(serde_json::json!({})),
    }
}

fn format_confidence_suffix(confidence: Option<f32>) -> String {
    confidence
        .map(|c| format!(" (mean confidence {:.0}%)", c * 100.0))
        .unwrap_or_default()
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

/// Best-effort panic-payload stringifier. Panics carry an `Any` payload
/// which is usually a `&'static str` or a `String`; for anything else we
/// emit a placeholder so the error message is still useful.
fn describe_panic(panic: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic.downcast_ref::<&'static str>() {
        (*s).to_string()
    } else if let Some(s) = panic.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_string()
    }
}

fn now_secs() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn audit_model_tier_escalation_candidates(
    conn: &rusqlite::Connection,
    report_id: &str,
    source_path: &str,
    doc_confidence: f32,
) {
    let thresholds = read_setting_json(conn, "tier_thresholds");
    let olmocr = read_setting_json(conn, "olmocr");
    let llm = read_setting_json(conn, "llm");

    let olmocr_threshold = setting_f32(
        &olmocr,
        "trigger_below_tesseract_confidence",
        setting_f32(&thresholds, "escalate_to_olmocr_below", 0.55),
    );
    let llm_threshold = setting_f32(
        &llm,
        "trigger_below_confidence",
        setting_f32(&thresholds, "escalate_to_llm_repair_below", 0.7),
    );

    let olmocr_path = olmocr.get("model_path").and_then(|v| v.as_str());
    let llm_path = llm.get("model_path").and_then(|v| v.as_str());
    let mut candidates = Vec::new();

    if setting_enabled(&olmocr) && doc_confidence < olmocr_threshold {
        candidates.push(json!({
            "tier": "embedded-ocr-vision",
            "threshold": olmocr_threshold,
            "model_present": crate::ocr_vision::model_present(olmocr_path),
            "loading": crate::ocr_vision::is_loading(),
            "loaded": crate::ocr_vision::is_loaded(),
        }));
    }

    if setting_enabled(&llm) && doc_confidence < llm_threshold {
        candidates.push(json!({
            "tier": "embedded-llm",
            "threshold": llm_threshold,
            "model_present": crate::llm::model_present(llm_path),
            "loading": crate::llm::is_loading(),
            "loaded": crate::llm::is_loaded(),
        }));
    }

    if candidates.is_empty() {
        return;
    }

    audit::log(
        conn,
        "tier_escalation_candidate",
        "report",
        Some(report_id),
        &format!("Low-confidence ingest ({doc_confidence:.2}) met model-tier escalation criteria"),
        Some(&json!({
            "report_id": report_id,
            "source_path": source_path,
            "doc_confidence": doc_confidence,
            "candidates": candidates,
            "output_modified": false,
        })),
    );
}

fn read_setting_json(conn: &rusqlite::Connection, key: &str) -> serde_json::Value {
    let raw: Option<String> = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            [key],
            |r| r.get::<_, String>(0),
        )
        .ok();
    raw.and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}))
}

fn setting_enabled(setting: &serde_json::Value) -> bool {
    setting
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn setting_f32(setting: &serde_json::Value, key: &str, fallback: f32) -> f32 {
    setting
        .get(key)
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(fallback)
}
