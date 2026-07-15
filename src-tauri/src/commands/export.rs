// CSV export commands. We return the CSV content as a String so the frontend
// can hand it to a save-file dialog; that keeps the path-picking flow inside
// the existing tauri_plugin_dialog UI, no new permissions to wire up.

use serde::Serialize;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Serialize)]
pub struct CsvOut {
    pub filename: String,
    pub content: String,
}

#[tauri::command]
pub async fn export_analyte_timeseries_csv(
    state: State<'_, AppState>,
    analyte_id: String,
    patient_id: Option<String>,
) -> AppResult<CsvOut> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;

    let pt_name: Option<String> = db
        .conn
        .query_row(
            "SELECT pt_name FROM analytes WHERE id = ?1",
            [&analyte_id],
            |r| r.get(0),
        )
        .ok();
    let display = pt_name.unwrap_or_else(|| analyte_id.clone());

    let sql = if patient_id.is_some() {
        "SELECT res.collection_date_iso, p.display_name, p.sex,
                res.value_numeric, res.value_qualitative, res.unit, res.unit_raw,
                res.ref_low, res.ref_high, res.ref_grammar, res.flag,
                res.method_annotation, res.report_id, res.inline_prior_pdf
         FROM results_canonical res
         JOIN reports r ON r.id = res.report_id
         JOIN patients p ON p.id = r.patient_id
         WHERE res.analyte_id = ?1 AND p.id = ?2
         ORDER BY res.collection_date_iso ASC, res.id ASC"
    } else {
        "SELECT res.collection_date_iso, p.display_name, p.sex,
                res.value_numeric, res.value_qualitative, res.unit, res.unit_raw,
                res.ref_low, res.ref_high, res.ref_grammar, res.flag,
                res.method_annotation, res.report_id, res.inline_prior_pdf
         FROM results_canonical res
         JOIN reports r ON r.id = res.report_id
         JOIN patients p ON p.id = r.patient_id
         WHERE res.analyte_id = ?1
         ORDER BY res.collection_date_iso ASC, res.id ASC"
    };

    let mut stmt = db.conn.prepare(sql)?;
    let mut buf = String::new();
    buf.push_str("date,patient,sex,value,qualitative,unit,unit_raw,ref_low,ref_high,ref_grammar,flag,method,report_id,is_prior\n");

    let mut map = |r: &rusqlite::Row<'_>| -> rusqlite::Result<()> {
        let date: String = r.get(0)?;
        let patient: String = r.get(1)?;
        let sex: String = r.get(2)?;
        let value: Option<f64> = r.get(3)?;
        let qual: Option<String> = r.get(4)?;
        let unit: Option<String> = r.get(5)?;
        let unit_raw: String = r.get(6)?;
        let ref_low: Option<f64> = r.get(7)?;
        let ref_high: Option<f64> = r.get(8)?;
        let grammar: String = r.get(9)?;
        let flag: Option<String> = r.get(10)?;
        let method: Option<String> = r.get(11)?;
        let report_id: String = r.get(12)?;
        let is_prior: i64 = r.get(13)?;
        push_row(
            &mut buf,
            &[
                date.as_str(),
                patient.as_str(),
                sex.as_str(),
                &opt_num(value),
                qual.as_deref().unwrap_or(""),
                unit.as_deref().unwrap_or(""),
                unit_raw.as_str(),
                &opt_num(ref_low),
                &opt_num(ref_high),
                grammar.as_str(),
                flag.as_deref().unwrap_or(""),
                method.as_deref().unwrap_or(""),
                report_id.as_str(),
                if is_prior != 0 { "1" } else { "0" },
            ],
        );
        Ok(())
    };

    if let Some(pid) = patient_id {
        for row in stmt.query_map([&analyte_id, &pid], |r| map(r))? {
            row?;
        }
    } else {
        for row in stmt.query_map([&analyte_id], |r| map(r))? {
            row?;
        }
    }

    let safe_name: String = display
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    Ok(CsvOut {
        filename: format!("{safe_name}.csv"),
        content: buf,
    })
}

#[tauri::command]
pub async fn export_report_rows_csv(
    state: State<'_, AppState>,
    report_id: String,
) -> AppResult<CsvOut> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;

    let (patient_name, date): (String, String) = db
        .conn
        .query_row(
            "SELECT p.display_name, r.collection_date_iso
             FROM reports r JOIN patients p ON p.id = r.patient_id
             WHERE r.id = ?1",
            [&report_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|_| AppError::NotFound(format!("report {report_id}")))?;

    let mut stmt = db.conn.prepare(
        "SELECT res.raw_analyte_text, COALESCE(a.pt_name, ''),
                res.value_numeric, res.value_qualitative, res.value_raw_text,
                res.unit, res.unit_raw, res.ref_low, res.ref_high, res.ref_grammar,
                res.flag, res.method_annotation, res.confidence, res.parse_method,
                res.inline_prior_pdf, res.collection_date_iso
         FROM results res
         LEFT JOIN analytes a ON a.id = res.analyte_id
         WHERE res.report_id = ?1
         ORDER BY res.id ASC",
    )?;

    let mut buf = String::new();
    buf.push_str(
        "raw_name,canonical_name,value,qualitative,value_raw,unit,unit_raw,\
         ref_low,ref_high,ref_grammar,flag,method,confidence,parse_method,is_prior,date\n",
    );

    for row in stmt.query_map([&report_id], |r| {
        let raw: String = r.get(0)?;
        let canon: String = r.get(1)?;
        let v: Option<f64> = r.get(2)?;
        let q: Option<String> = r.get(3)?;
        let v_raw: String = r.get(4)?;
        let unit: Option<String> = r.get(5)?;
        let unit_raw: String = r.get(6)?;
        let rl: Option<f64> = r.get(7)?;
        let rh: Option<f64> = r.get(8)?;
        let g: String = r.get(9)?;
        let f: Option<String> = r.get(10)?;
        let m: Option<String> = r.get(11)?;
        let c: f64 = r.get(12)?;
        let pm: String = r.get(13)?;
        let prior: i64 = r.get(14)?;
        let d: String = r.get(15)?;
        Ok((
            raw, canon, v, q, v_raw, unit, unit_raw, rl, rh, g, f, m, c, pm, prior, d,
        ))
    })? {
        let (raw, canon, v, q, v_raw, unit, unit_raw, rl, rh, g, f, m, c, pm, prior, d) = row?;
        push_row(
            &mut buf,
            &[
                &raw,
                &canon,
                &opt_num(v),
                q.as_deref().unwrap_or(""),
                &v_raw,
                unit.as_deref().unwrap_or(""),
                &unit_raw,
                &opt_num(rl),
                &opt_num(rh),
                &g,
                f.as_deref().unwrap_or(""),
                m.as_deref().unwrap_or(""),
                &format!("{:.3}", c),
                &pm,
                if prior != 0 { "1" } else { "0" },
                &d,
            ],
        );
    }

    let safe_patient: String = patient_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    Ok(CsvOut {
        filename: format!("{date}_{safe_patient}.csv"),
        content: buf,
    })
}

fn opt_num(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{x}"),
        None => String::new(),
    }
}

/// Quotes only when needed (RFC 4180 minimal): wrap in quotes when the field
/// contains comma, quote, or newline; double internal quotes.
fn push_row(out: &mut String, cells: &[&str]) {
    for (i, cell) in cells.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        if cell.contains([',', '"', '\n', '\r']) {
            out.push('"');
            for ch in cell.chars() {
                if ch == '"' {
                    out.push('"');
                }
                out.push(ch);
            }
            out.push('"');
        } else {
            out.push_str(cell);
        }
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::push_row;

    #[test]
    fn push_row_quotes_on_special_chars() {
        let mut s = String::new();
        push_row(&mut s, &["plain", "has,comma", "has\"quote", "multi\nline"]);
        assert_eq!(s, "plain,\"has,comma\",\"has\"\"quote\",\"multi\nline\"\n");
    }

    #[test]
    fn push_row_no_trailing_comma() {
        let mut s = String::new();
        push_row(&mut s, &["a", "b", "c"]);
        assert_eq!(s, "a,b,c\n");
    }

    #[test]
    fn push_row_handles_empty_cells() {
        let mut s = String::new();
        push_row(&mut s, &["", "x", ""]);
        assert_eq!(s, ",x,\n");
    }
}
