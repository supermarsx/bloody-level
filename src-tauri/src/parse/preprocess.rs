// Pre-processing of pdfium-extracted text before line-by-line parsing.
//
// pdfium splits text at PDF line boundaries, which often fall inside what is
// logically one row in the source layout. Two recurring patterns:
//
//   1. Unit wrap. "x 10³/" or "x 10⁶/µl" gets split as `x 103\n/` or
//      `x 106\n/µl` because the superscript and slash often have slightly
//      different y-positions in the source PDF.
//
//   2. Continuation. The "/", "/µl", or "µl" tail of a unit can land on its
//      own line. We glue those onto the previous line.
//
// Joining is deliberately conservative — we only merge if the next line LOOKS
// like a unit fragment (starts with `/`, `µl`, or `uL`). Genuine analyte rows
// always start with a letter, so this can't accidentally collapse two separate
// rows.

use once_cell::sync::Lazy;
use regex::Regex;

/// Apply all preprocessing in order. Returns text safe to feed line-by-line
/// to the row parser.
pub fn normalize(text: &str) -> String {
    let s = join_unit_breaks(text);
    let s = merge_continuation_lines(&s);
    fix_misordered_values(&s)
}

/// pdfium sometimes orders the value cell BEFORE the analyte cell, producing
/// lines like:
///   "0Creatininémia .83 mg/dL 0.50 - 1.10 …"
///   "94TFGe [CKD-EPI 2009] ml/min/1,73 m2 >= 60"
/// where `0` and `.83` together are the real value (0.83), or where `94` is
/// the value with no decimal fragment.
///
/// This pre-processor looks for a leading `(\d+)(uppercase letter)` boundary
/// and either (a) merges the leading digits with a decimal fragment found
/// later in the line, or (b) inserts the leading digits right before the
/// first unit token, in their canonical position.
pub fn fix_misordered_values(text: &str) -> String {
    text.lines()
        .map(fix_misordered_value_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn fix_misordered_value_line(line: &str) -> String {
    static RE_PREFIX: Lazy<Regex> = Lazy::new(|| {
        // Leading digits glued to an uppercase letter (Latin + PT diacritics).
        Regex::new(r"^(\d+)([A-ZÀ-Ý])").unwrap()
    });
    let Some(cap) = RE_PREFIX.captures(line) else {
        return line.to_string();
    };
    let leading = cap[1].to_string();
    let prefix_end = cap.get(1).unwrap().end();
    let rest = &line[prefix_end..];

    // (a) decimal-fragment case: "0Creatininémia .83 …" → "Creatininémia 0.83 …"
    static RE_DEC_FRAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s(\.\d+)(\s|$)").unwrap());
    if let Some(dec_cap) = RE_DEC_FRAG.captures(rest) {
        let decimal = &dec_cap[1];
        let dec_start = dec_cap.get(1).unwrap().start();
        let dec_end = dec_cap.get(1).unwrap().end();
        let merged = format!("{leading}{decimal}");
        let mut out = String::with_capacity(rest.len());
        out.push_str(&rest[..dec_start]);
        out.push_str(&merged);
        out.push_str(&rest[dec_end..]);
        return out;
    }

    // (b) no decimal fragment: insert leading right before the first unit.
    static RE_UNIT_HINT: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"\s(mg/d[lL]|g/d[lL]|ng/m[lL]|ng/d[lL]|ng/[lL]|pg/m[lL]|pg|f[lL]|U/[lL]|UI/m[lL]|mUI/[mlL]+|mmol/[lL]|mmol/mol|nmol/[lL]|µ?g/d[lL]|ug/d[lL]|µ[lL]|%|x\s*10\^?\d+(?:/[µu]?[lL]?)?|ml/min/1,?73\s*m²?2?)\b").unwrap()
    });
    if let Some(unit_match) = RE_UNIT_HINT.find(rest) {
        let pos = unit_match.start();
        let before = rest[..pos].trim_end();
        let after = &rest[pos..];
        return format!("{before} {leading}{after}");
    }

    // No unit found — append at end (will be picked as the trailing value).
    format!("{rest} {leading}")
}

/// Joins `x N\n/` patterns into `x N/`, handling whitespace on either side.
pub fn join_unit_breaks(text: &str) -> String {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(x\s*10\^?\d+)\s*\n\s*(/)").unwrap());
    RE.replace_all(text, "$1$2").into_owned()
}

/// Merges lines that start with a unit-fragment prefix into the previous line.
pub fn merge_continuation_lines(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < lines.len() {
        let mut current = lines[i].to_string();
        while i + 1 < lines.len() {
            let next = lines[i + 1].trim_start();
            let is_unit_continuation = next.starts_with('/')
                || next.starts_with("µl")
                || next.starts_with("µL")
                || next.starts_with("uL")
                || next.starts_with("ul");
            if is_unit_continuation {
                current.push(' ');
                current.push_str(next);
                i += 1;
            } else {
                break;
            }
        }
        out.push_str(&current);
        out.push('\n');
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_unit_break_with_caret() {
        let input = "Neutrófilos 64.0 % 5.54 x 10^3\n/ 40.00 - 80.00";
        let out = normalize(input);
        assert!(out.contains("x 10^3/ 40.00"), "got: {out}");
    }

    #[test]
    fn joins_unit_break_no_caret() {
        let input = "Eosinófilos 1.7 % 0.15 x 103\n/ 1.00 - 6.00";
        let out = normalize(input);
        assert!(out.contains("x 103/ 1.00"), "got: {out}");
    }

    #[test]
    fn merges_slash_ul_continuation() {
        let input = "Eritrócitos 4.19 x 106\n/µl 3.80 - 4.80 4.02 4.13 4.06";
        let out = normalize(input);
        assert!(
            out.contains("x 106 /µl 3.80") || out.contains("x 106/µl 3.80"),
            "got: {out}"
        );
    }

    #[test]
    fn does_not_merge_unrelated_lines() {
        let input = "Hemoglobina 13.4 g/dl 12.0 - 15.0\nEritrócitos 4.19 x 106/µl 3.80 - 4.80";
        let out = normalize(input);
        // The two analyte rows must stay separate.
        let line_count = out.trim_end().lines().count();
        assert_eq!(line_count, 2, "got: {out}");
    }

    #[test]
    fn fixes_creatinine_misordered_value_with_decimal_fragment() {
        // "0Creatininémia .83 mg/dL 0.50 - 1.10 …" → "Creatininémia 0.83 mg/dL 0.50 - 1.10 …"
        let input = "0Creatininémia .83 mg/dL 0.50 - 1.10 1.09 0.89 0.75";
        let out = fix_misordered_value_line(input);
        assert_eq!(out, "Creatininémia 0.83 mg/dL 0.50 - 1.10 1.09 0.89 0.75");
    }

    #[test]
    fn fixes_tfge_misordered_value_no_decimal() {
        // "94TFGe [CKD-EPI 2009] ml/min/1,73 m2 >= 60" → value moves before the unit
        let input = "94TFGe [CKD-EPI 2009] ml/min/1,73 m2 >= 60";
        let out = fix_misordered_value_line(input);
        assert_eq!(out, "TFGe [CKD-EPI 2009] 94 ml/min/1,73 m2 >= 60");
    }

    #[test]
    fn does_not_touch_well_ordered_lines() {
        let input = "Hemoglobina 13.4 g/dl 12.0 - 15.0";
        assert_eq!(fix_misordered_value_line(input), input);

        let input2 = "Glicémia 83 mg/dL 70 - 110";
        assert_eq!(fix_misordered_value_line(input2), input2);
    }

    #[test]
    fn full_normalize_covers_all_three() {
        let input =
            "0Creatininémia .83 mg/dL 0.50 - 1.10\n94TFGe [CKD-EPI 2009] ml/min/1,73 m2 >= 60";
        let out = normalize(input);
        assert!(out.contains("Creatininémia 0.83 mg/dL"), "got: {out}");
        assert!(
            out.contains("TFGe [CKD-EPI 2009] 94 ml/min/1,73 m2"),
            "got: {out}"
        );
    }

    // ───── Empty / pathological inputs ──────────────────────────────────

    #[test]
    fn normalize_empty_input() {
        // Empty input -> `.lines()` yields nothing, so no trailing newline.
        assert_eq!(normalize(""), "");
    }

    #[test]
    fn normalize_only_newlines() {
        let out = normalize("\n\n\n");
        // Each line iterates once; \n appended after each.
        assert_eq!(out.matches('\n').count(), out.len());
    }

    #[test]
    fn normalize_does_not_panic_on_only_multibyte() {
        // Each char is multi-byte; no analyte rows. Just ensure no panic.
        let _ = normalize("ãçõáéíóú\n");
    }

    #[test]
    fn normalize_preserves_unicode() {
        let input = "Glicémia 83 mg/dL 70 - 110\n";
        let out = normalize(input);
        assert!(out.contains("Glicémia"));
    }

    // ───── join_unit_breaks edge cases ───────────────────────────────────

    #[test]
    fn join_unit_break_does_not_join_normal_lines() {
        let input = "Foo bar\n/baz qux";
        // No `x \d+` prefix, so should NOT be joined.
        let out = join_unit_breaks(input);
        assert_eq!(out, input);
    }

    #[test]
    fn join_unit_break_handles_extra_whitespace() {
        let input = "x 103   \n   / 40.00";
        let out = join_unit_breaks(input);
        assert!(out.contains("x 103/ 40.00"), "got: {out}");
    }

    #[test]
    fn join_unit_break_idempotent() {
        let already_joined = "x 10^3/ 40.00 - 80.00";
        assert_eq!(join_unit_breaks(already_joined), already_joined);
    }

    // ───── merge_continuation_lines edge cases ──────────────────────────

    #[test]
    fn merge_continuation_handles_microliter_capital() {
        let input = "Eritrócitos 4.19 x 106\nµL 3.80 - 4.80";
        let out = merge_continuation_lines(input);
        assert!(out.contains("x 106 µL"), "got: {out}");
    }

    #[test]
    fn merge_continuation_handles_ul_ascii() {
        let input = "Some line\nuL trailer";
        let out = merge_continuation_lines(input);
        assert!(out.contains("Some line uL trailer"), "got: {out}");
    }

    #[test]
    fn merge_continuation_handles_multiple_consecutive() {
        // Two continuation lines in a row (rare but possible).
        let input = "Foo bar baz\n/µl\n/another";
        let out = merge_continuation_lines(input);
        assert!(out.contains("Foo bar baz /µl /another"), "got: {out}");
    }

    #[test]
    fn merge_continuation_does_not_merge_letter_starting_lines() {
        let input = "Hemoglobina 13.4 g/dl\nEritrócitos 4.19";
        let out = merge_continuation_lines(input);
        assert_eq!(out.trim_end().lines().count(), 2);
    }

    #[test]
    fn merge_continuation_empty_input() {
        let out = merge_continuation_lines("");
        // Empty -> "" (no iterations), which trims to empty.
        assert_eq!(out, "");
    }

    // ───── fix_misordered_values edge cases ─────────────────────────────

    #[test]
    fn fix_misordered_skips_rows_without_digit_prefix() {
        let line = "Just a normal sentence";
        assert_eq!(fix_misordered_value_line(line), line);
    }

    #[test]
    fn fix_misordered_does_not_eat_legit_value() {
        // "12 anos" — `12` is a digit but next char is space, so the regex
        // requires an immediate uppercase letter. Should NOT be touched.
        let line = "Adultos: 12 anos";
        assert_eq!(fix_misordered_value_line(line), line);
    }

    #[test]
    fn fix_misordered_handles_diacritic_uppercase() {
        // À-Ý in regex covers Á É Í Ó Ú Ã Õ Ç-cap (Ç is U+00C7, in range).
        let line = "94Ácido úrico mg/dL 3 - 7";
        let out = fix_misordered_value_line(line);
        assert!(out.contains("Ácido úrico"), "got: {out}");
        assert!(out.contains("94"), "got: {out}");
    }

    #[test]
    fn fix_misordered_no_unit_falls_back_to_appending() {
        let line = "100Foo bar baz";
        let out = fix_misordered_value_line(line);
        // Falls through to: "{rest} {leading}"
        assert_eq!(out, "Foo bar baz 100");
    }

    #[test]
    fn fix_misordered_multi_line_independent() {
        // Each line is processed independently; well-ordered ones pass through.
        let input = "Hemoglobina 13.4 g/dl 12.0 - 15.0\n94TFGe ml/min/1,73 m2 >= 60";
        let out = fix_misordered_values(input);
        assert!(out.contains("Hemoglobina 13.4 g/dl"));
        assert!(out.contains("TFGe 94 ml/min/1,73 m2"));
    }
}
