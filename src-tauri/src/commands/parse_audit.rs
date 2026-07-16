use crate::error::AppResult;
use crate::parse::{ParsedRow, RangeGrammar};

pub(crate) const DEFAULT_LOW_CONFIDENCE_THRESHOLD: f32 = 0.7;

pub(crate) fn replace_report_diagnostics(
    tx: &rusqlite::Transaction<'_>,
    report_id: &str,
    rows: &[ParsedRow],
    ocr_tier: i64,
    low_confidence_threshold: f32,
) -> AppResult<usize> {
    tx.execute("DELETE FROM parse_audit WHERE report_id = ?1", [report_id])?;
    insert_report_diagnostics(tx, report_id, rows, ocr_tier, low_confidence_threshold)
}

fn insert_report_diagnostics(
    tx: &rusqlite::Transaction<'_>,
    report_id: &str,
    rows: &[ParsedRow],
    ocr_tier: i64,
    low_confidence_threshold: f32,
) -> AppResult<usize> {
    let mut inserted = 0usize;
    let mut stmt = tx.prepare(
        "INSERT INTO parse_audit(
            report_id, row_index, diagnostic, parse_method, confidence,
            llm_repaired, llm_prompt_hash, ocr_tier
         ) VALUES (?1, ?2, ?3, ?4, ?5, 0, NULL, ?6)",
    )?;

    for (idx, row) in rows.iter().enumerate() {
        for diagnostic in diagnostics_for_row(row, low_confidence_threshold) {
            stmt.execute(rusqlite::params![
                report_id,
                idx as i64,
                diagnostic,
                row.parse_method,
                row.confidence as f64,
                ocr_tier,
            ])?;
            inserted += 1;
        }
    }

    Ok(inserted)
}

fn diagnostics_for_row(row: &ParsedRow, low_confidence_threshold: f32) -> Vec<&'static str> {
    let mut diagnostics = Vec::new();

    if row.analyte_id.is_none() {
        diagnostics.push("analyte_unmatched");
    }
    if row.ref_grammar == RangeGrammar::Unparsed {
        diagnostics.push("reference_range_unparsed");
    }
    if row.unit.is_none() && !row.unit_raw.trim().is_empty() {
        diagnostics.push("unit_unrecognized");
    }
    if row.value.numeric.is_none() && row.value.qualitative.is_none() {
        diagnostics.push("value_missing");
    }
    if row.confidence < low_confidence_threshold {
        diagnostics.push("low_confidence");
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{ParsedValue, RangeGrammar};

    fn row() -> ParsedRow {
        ParsedRow {
            raw_analyte_text: "Unknown".into(),
            analyte_id: None,
            value: ParsedValue {
                numeric: Some(1.2),
                qualitative: None,
                raw: "1.2".into(),
            },
            unit: None,
            unit_raw: "bogus".into(),
            ref_low: None,
            ref_high: None,
            ref_grammar: RangeGrammar::Unparsed,
            ref_raw_text: Some("bad".into()),
            flag: None,
            method_annotation: None,
            parse_method: "parse_standard_row",
            confidence: 0.4,
            inline_priors: vec![],
        }
    }

    #[test]
    fn builds_expected_diagnostics_for_low_confidence_row() {
        let diagnostics = diagnostics_for_row(&row(), DEFAULT_LOW_CONFIDENCE_THRESHOLD);
        assert_eq!(
            diagnostics,
            vec![
                "analyte_unmatched",
                "reference_range_unparsed",
                "unit_unrecognized",
                "low_confidence"
            ]
        );
    }

    #[test]
    fn writes_and_replaces_report_diagnostics() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE parse_audit (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                report_id TEXT NOT NULL,
                row_index INTEGER NOT NULL,
                diagnostic TEXT NOT NULL,
                parse_method TEXT,
                confidence REAL,
                llm_repaired INTEGER NOT NULL DEFAULT 0,
                llm_prompt_hash TEXT,
                ocr_tier INTEGER NOT NULL
            );",
        )
        .unwrap();

        let tx = conn.transaction().unwrap();
        let inserted =
            replace_report_diagnostics(&tx, "r1", &[row()], 2, DEFAULT_LOW_CONFIDENCE_THRESHOLD)
                .unwrap();
        assert_eq!(inserted, 4);
        tx.commit().unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM parse_audit", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 4);

        let tx = conn.transaction().unwrap();
        let inserted =
            replace_report_diagnostics(&tx, "r1", &[], 2, DEFAULT_LOW_CONFIDENCE_THRESHOLD)
                .unwrap();
        assert_eq!(inserted, 0);
        tx.commit().unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM parse_audit", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
