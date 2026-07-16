use once_cell::sync::Lazy;
use regex::Regex;

use super::canonical::AnalyteRegistry;
use super::ranges::{
    parse_range_a_b, parse_range_gt, parse_range_gte, parse_range_lt, parse_range_lte,
    try_parse_any, ParsedRange,
};
use super::sections::SectionContext;
use super::units::normalize_unit;
use super::values::{parse_inequality_value, parse_numeric_decimal, parse_qualitative_value};
use super::{ParsedRow, ParsedValue, RangeGrammar};

const KNOWN_UNITS: &[&str] = &[
    "g/dl",
    "g/dL",
    "mg/dl",
    "mg/dL",
    "mmol/l",
    "mmol/L",
    "mmol/mol",
    "U/l",
    "U/L",
    "UI/ml",
    "UI/mL",
    "ng/dl",
    "ng/dL",
    "ng/ml",
    "ng/mL",
    "ng/l",
    "ng/L",
    "pg/ml",
    "pg/mL",
    "pg",
    "fl",
    "fL",
    "%",
    "µl",
    "uL",
    "µL",
    "mUI/l",
    "mUI/ml",
    "mUI/L",
    "mUI/mL",
    "µg/dl",
    "µg/dL",
    "ug/dL",
    "nmol/l",
    "nmol/L",
    // x 10^N/µl variants — pdfium drops the caret, so we list both forms.
    // Order doesn't matter because find_first_unit picks the *earliest position*
    // in the line (with longer-wins on ties), not the longest registered unit.
    "x 10^3/µl",
    "x 103/µl",
    "x 10^6/µl",
    "x 106/µl",
    "x 10^3/",
    "x 103/",
    "ml/min/1,73 m2",
];

pub fn parse_row(line: &str, ctx: &SectionContext, reg: &AnalyteRegistry) -> Option<ParsedRow> {
    let raw = line.trim();
    if raw.is_empty() || is_layout_chrome(raw) {
        return None;
    }

    if let Some(q) = parse_qualitative_value(extract_last_token(raw)) {
        let name = strip_trailing_token(raw, q).trim().to_string();
        if name.is_empty() {
            return None;
        }
        let analyte_id = reg.resolve(&name).map(String::from);
        // Qualitative rows have no numeric value or unit — those slots aren't
        // applicable, not missing. Confidence reflects only what's relevant:
        // analyte resolved + qualitative term recognised. A row that hits
        // *this* branch already has a recognised qualitative term, so the
        // only quality signal left is whether the analyte resolved.
        let confidence = if analyte_id.is_some() { 1.0 } else { 0.7 };
        return Some(ParsedRow {
            raw_analyte_text: name,
            analyte_id,
            value: ParsedValue {
                qualitative: Some(q.into()),
                raw: q.into(),
                ..Default::default()
            },
            unit: None,
            unit_raw: String::new(),
            ref_low: None,
            ref_high: None,
            ref_grammar: RangeGrammar::Qualitative,
            ref_raw_text: None,
            flag: flag_for_qualitative(q),
            method_annotation: None,
            parse_method: "parse_qualitative_row",
            confidence,
            inline_priors: vec![],
        });
    }

    try_parse_numeric_row(raw, ctx, reg)
}

fn try_parse_numeric_row(
    raw: &str,
    ctx: &SectionContext,
    reg: &AnalyteRegistry,
) -> Option<ParsedRow> {
    let unit_match = find_first_unit(raw)?;
    let (left, after_unit_with) = split_at(raw, unit_match.start);
    let unit_raw = unit_match.token.to_string();
    let after_unit = &after_unit_with[unit_match.token.len()..];

    let value_str = trailing_number_or_inequality(left.trim_end()).map(|s| s.to_string())?;
    let analyte_text = left
        .trim_end()
        .trim_end_matches(&value_str)
        .trim()
        .to_string();
    if analyte_text.is_empty() {
        return None;
    }

    let value = parse_numeric_decimal(&value_str)
        .or_else(|| parse_inequality_value(&value_str).map(|(_, v)| v));

    let (range, after_range) = peel_range(after_unit.trim());

    let prior_values: Vec<f64> = after_range
        .split_whitespace()
        .filter_map(parse_numeric_decimal)
        .collect();
    let inline_priors: Vec<(String, f64)> = prior_values
        .into_iter()
        .zip(ctx.prior_dates.iter().cloned())
        .map(|(v, d)| (d, v))
        .collect();

    let analyte_id = reg.resolve(&analyte_text).map(String::from);
    let unit_canon = normalize_unit(&unit_raw).map(String::from);

    let flag = derive_flag(value, range.low, range.high);
    let confidence = compute_confidence(&analyte_id, value.is_some(), &range, unit_canon.is_some());

    Some(ParsedRow {
        raw_analyte_text: analyte_text,
        analyte_id,
        value: ParsedValue {
            numeric: value,
            raw: value_str,
            ..Default::default()
        },
        unit: unit_canon,
        unit_raw,
        ref_low: range.low,
        ref_high: range.high,
        ref_grammar: range.grammar,
        ref_raw_text: Some(range.raw),
        flag,
        method_annotation: None,
        parse_method: "parse_standard_row",
        confidence,
        inline_priors,
    })
}

fn peel_range(s: &str) -> (ParsedRange, String) {
    // Leucograma sub-rows: skip past the absolute-count column that sits
    // between the % unit and the actual reference range.
    let s = if let Some(m) = RE_ABS_COUNT_LEAD.find(s) {
        &s[m.end()..]
    } else {
        s
    };

    // Electroforese das Proteínas: skip past the absolute-concentration
    // column (e.g., "4.5 g/dl") between the % unit and the real range.
    let s = if let Some(m) = RE_PAIRED_ABSOLUTE_LEAD.find(s) {
        &s[m.end()..]
    } else {
        s
    };

    let tokens: Vec<&str> = s.split_whitespace().collect();
    if tokens.is_empty() {
        return (ParsedRange::none(""), String::new());
    }

    if tokens.len() >= 3 {
        let cand = format!("{} {} {}", tokens[0], tokens[1], tokens[2]);
        if let Some(r) = parse_range_a_b(&cand) {
            return (r, tokens[3..].join(" "));
        }
    }
    if tokens.len() >= 2 {
        let cand = format!("{} {}", tokens[0], tokens[1]);
        if let Some(r) = parse_range_lte(&cand)
            .or_else(|| parse_range_gte(&cand))
            .or_else(|| parse_range_lt(&cand))
            .or_else(|| parse_range_gt(&cand))
        {
            return (r, tokens[2..].join(" "));
        }
    }
    if let Some(r) = parse_range_lte(tokens[0])
        .or_else(|| parse_range_gte(tokens[0]))
        .or_else(|| parse_range_lt(tokens[0]))
        .or_else(|| parse_range_gt(tokens[0]))
    {
        return (r, tokens[1..].join(" "));
    }

    // Common case: this analyte's reference range is rendered as a separate
    // sub-table elsewhere (Vit D categorical tiers, Estradiol cycle-phase
    // table). The value-line tail then carries ONLY prior-result numbers.
    // Recognize that pattern as `grammar = None` (range absent) with the
    // numbers feeding the inline-priors column, instead of falling through
    // to Unparsed.
    if tokens.iter().all(|t| parse_numeric_decimal(t).is_some()) {
        return (ParsedRange::none(""), tokens.join(" "));
    }

    (try_parse_any(s), String::new())
}

#[derive(Debug, Clone, Copy)]
struct UnitHit {
    start: usize,
    token: &'static str,
}

// Pick the EARLIEST whitespace-bounded unit in the line, with ties broken by
// longest-wins. Earliest-position matters for leucograma rows like
//   "Neutrófilos 61.3 % 5.02 x 103/ 40.00 - 80.00 …"
// where both `%` and `x 103/` are valid units — `%` comes first, so the
// percent is the primary value (and the absolute count gets skipped by
// peel_range). Without this, the longest-match policy picks `x 103/` and
// strands the % column as part of the "analyte name."
fn find_first_unit(s: &str) -> Option<UnitHit> {
    let mut earliest: Option<UnitHit> = None;
    for u in KNOWN_UNITS.iter() {
        if let Some(idx) = s.find(u) {
            let before_ok = idx == 0
                || s.as_bytes()
                    .get(idx - 1)
                    .is_some_and(|b| (*b as char).is_whitespace());
            if !before_ok {
                continue;
            }
            let is_better = match earliest {
                None => true,
                Some(h) => idx < h.start || (idx == h.start && u.len() > h.token.len()),
            };
            if is_better {
                earliest = Some(UnitHit {
                    start: idx,
                    token: u,
                });
            }
        }
    }
    earliest
}

fn split_at(s: &str, idx: usize) -> (&str, &str) {
    (&s[..idx], &s[idx..])
}

// The trailing value can be:
//   - a plain number (`91.4`, `353`, `1.234,5`)
//   - an inequality (`< 38`, `>= 60`, `<=42`)
// Anchored to end-of-string + a whitespace boundary on the left so we don't eat
// the trailing dot of an acronym name like "V.G.M.".
static RE_TRAILING_VALUE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:^|\s)([<>]=?\s*-?\d+(?:[.,]\d+)?|-?\d+(?:[.,]\d+)?)\s*$").unwrap()
});

// Leucograma sub-rows print TWO columns of numbers: percentage (with `%`) AND
// absolute count (with `x 10^N/`). When we've already chosen `%` as the unit,
// the after-unit text starts with the absolute-count column followed by the
// real reference range. This regex matches (and lets us skip past) that
// absolute-count column so peel_range gets the actual range.
//
// Matches:    " 5.02 x 103/ "   " 0.15 x 10^3/µl "   " 1.5 x 10^6 "
// Doesn't:    " 40.0 - 80.0 "   " 12.6 12.9 12.7 "
static RE_ABS_COUNT_LEAD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*\d+(?:[.,]\d+)?\s+x\s*10\^?\d+\s*/?\s*(?:[µu][lL])?\s+").unwrap()
});

// Electroforese das Proteínas rows print TWO columns: percentage (`%`) AND
// absolute concentration (`<num> g/dl`). When `%` was picked as primary unit,
// the after-text starts with the absolute-concentration column.
//
// Matches:    " 4.5 g/dl "   " 0.55 g/dL "   " 1.2 mg/dL "
// Doesn't:    " 12.0 - 15.0 "  (no unit after the number)
//             " 4.5 5.0 "      (no unit between)
static RE_PAIRED_ABSOLUTE_LEAD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^\s*\d+(?:[.,]\d+)?\s+(?:g/d[lL]|mg/d[lL]|ng/m[lL]|ng/d[lL]|pg/m[lL]|U/[lL]|UI/m[lL]|mUI/[mlL]+|mmol/[lL]|nmol/[lL]|[µu]g/d[lL])\s+"
    ).unwrap()
});

fn trailing_number_or_inequality(s: &str) -> Option<&str> {
    let trimmed = s.trim_end();
    let cap = RE_TRAILING_VALUE.captures(trimmed)?;
    let m = cap.get(1)?;
    Some(&trimmed[m.start()..m.end()])
}

fn extract_last_token(s: &str) -> &str {
    s.split_whitespace().next_back().unwrap_or("")
}

fn strip_trailing_token<'a>(s: &'a str, token: &str) -> &'a str {
    let trimmed = s.trim_end();
    if trimmed.ends_with(token) {
        let new_len = trimmed.len() - token.len();
        trimmed[..new_len].trim_end()
    } else {
        trimmed
    }
}

/// Map a qualitative result string to a Flag the UI can color-code.
/// Positivo / Reactivo → abnormal (orange). Negativo / Não detectado → normal.
fn flag_for_qualitative(q: &str) -> Option<&'static str> {
    match q {
        "Positivo" | "Reactivo" | "Reativo" => Some("abnormal_qual"),
        "Negativo" | "Não Reactivo" | "Não Reativo" | "Não detectado" => Some("normal"),
        _ => None,
    }
}

fn derive_flag(value: Option<f64>, low: Option<f64>, high: Option<f64>) -> Option<&'static str> {
    let v = value?;
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

fn compute_confidence(
    analyte_id: &Option<String>,
    has_value: bool,
    range: &ParsedRange,
    has_unit: bool,
) -> f32 {
    let resolved = analyte_id.is_some();
    let s_name: f32 = if resolved { 1.0 } else { 0.5 };
    let s_value: f32 = if has_value { 1.0 } else { 0.0 };
    // RangeGrammar::None means the row contained no parseable range token —
    // not a parser failure for analytes that legitimately use a tier table
    // or cycle-phase table printed elsewhere on the page (Vit D, Ferritina,
    // Estradiol, FSH, LH, IgE Total). When the analyte resolved, the ontology
    // supplies the ref and this row is fully confident; only Unparsed (had
    // text but couldn't decode it) keeps the penalty. When the analyte did
    // NOT resolve, an empty range *is* a quality signal (we have nothing
    // anchoring the row), so we still discount it then.
    let s_range: f32 = match range.grammar {
        RangeGrammar::Unparsed => 0.0,
        RangeGrammar::None => {
            if resolved {
                1.0
            } else {
                0.6
            }
        }
        _ => 1.0,
    };
    let s_unit: f32 = if has_unit { 1.0 } else { 0.6 };
    let prod = (s_name * s_value * s_range * s_unit).max(0.0001_f32);
    prod.powf(0.25)
}

fn is_layout_chrome(s: &str) -> bool {
    // Lab-letterhead / page chrome.
    const CHROME: &[&str] = &[
        "Pólo Tecnológico",
        "GERMANO DE SOUSA",
        "CENTRO DE MEDICINA",
        "LABORATÓRIO CERTIFICADO",
        "Pág.",
        "Resultados anteriores",
        "Nº Inscrição",
        "Data de colheita",
        "Data de emissão",
        "Data ",
    ];
    if CHROME.iter().any(|n| s.contains(*n)) {
        return true;
    }

    let trimmed = s.trim_start();

    // Footnote markers — leading `*` or `**` (e.g., "*VPN: ...", "**VPP: ...").
    if trimmed.starts_with('*') {
        return true;
    }

    // For prefix matching, also strip leading bullet decorations like
    // "- Se Troponina …" so the prefix list catches the underlying form.
    let after_bullet =
        trimmed.trim_start_matches(|c: char| c == '-' || c == '·' || c == '•' || c.is_whitespace());

    // Categorical-tier sub-table labels — these are reference-range commentary
    // on the *previous* analyte (e.g., LDL risk tiers, Vit D deficiency tiers,
    // ferritin iron-deficiency tiers). Without this filter they get parsed as
    // bogus analyte rows like "Baixo ou moderado: 115 mg/dL".
    const TIER_LABEL_PREFIXES: &[&str] = &[
        "Baixo ou moderado",
        "Elevado:",
        "Muito elevado",
        "Deficiência:",
        "Insuficiência:",
        "Suficiência:",
        "Toxicidade:",
        "Ferropénia Absoluta",
        "Ferropénia Funcional",
        "Pacientes com",
        "Doentes com factores",
        "Doentes com",
        "Colesterol LDL recomendado",
        "Glicémia média estimada", // derived value, not a measured analyte
        "Nota:",
        "Nota :",
        "Notas:",
        "Ref. Bibliográfica",
        "[Ref. Bibliográfica",
        "Se aplicável",
        "Recomenda-se",
        // Clinical decision-rule prefixes — these are interpretation rules for
        // the *previous* analyte (e.g., Troponin algorithms), not measurements.
        "Se Troponina",
        "Se cTn",
        "Se Tn",
        "VPN:",
        "VPP:",
        "1.",
        "2.",
        "3.", // numbered footnote bullets
    ];
    if TIER_LABEL_PREFIXES
        .iter()
        .any(|p| after_bullet.starts_with(p))
    {
        return true;
    }

    // Threshold-definition lines — see is_threshold_definition_line for shape.
    if is_threshold_definition_line(trimmed) {
        return true;
    }

    // Age-stratified reference rows — "< 1 mês 1.5 - 31.0 pg/mL", "20-50 anos
    // 8.69 - 54.69 pg/mL", "1 - 3 meses 3.3 - 18.0 pg/mL". These belong to a
    // sub-table for analytes like Testosterona Livre, IgE Total etc.
    if is_age_tier_line(trimmed) {
        return true;
    }

    false
}

/// Recognize age-stratified reference-table rows (e.g., for Testosterona
/// Livre, IgE Total). The shape is "<age expr> <range> <unit>": an age
/// indicator (mês/meses/anos/dias/semanas) somewhere in the line, a numeric
/// range, and a unit. All three must be present so we don't accidentally
/// filter measurement lines that happen to contain "anos" elsewhere.
fn is_age_tier_line(s: &str) -> bool {
    static RE_AGE_WORD: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?i)\b(?:m[eê]s|meses|anos?|dias?|sem(?:anas?)?)\b").unwrap());
    static RE_RANGE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\d+(?:[.,]\d+)?\s*-\s*\d+(?:[.,]\d+)?").unwrap());
    static RE_UNIT: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"\b(?:pg/m[lL]|ng/m[lL]|ng/d[lL]|ng/[lL]|UI/m[lL]|mUI/m[lL]|[µu]g/d[lL]|nmol/[lL]|mmol/[lL]|mIU/m[lL]|U/[lL])\b"
        ).unwrap()
    });
    RE_AGE_WORD.is_match(s) && RE_RANGE.is_match(s) && RE_UNIT.is_match(s)
}

fn is_threshold_definition_line(s: &str) -> bool {
    static RE_THRESHOLD: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?x)
            ^
            [A-ZÀ-Ý][A-Za-zÀ-ÿ\s]{1,40}        # analyte-name-ish prefix
            \s* :? \s*                          # optional colon
            (?:
                [<>]=? \s* \d+(?:[.,]\d+)?       # comparator + number
              |
                \d+(?:[.,]\d+)? \s* [-–] \s* \d+(?:[.,]\d+)?  # range a - b
            )
            \s+
            (?:mg/d[lL]|g/d[lL]|ng/m[lL]|ng/d[lL]|ng/[lL]|pg/m[lL]
              |U/[lL]|UI/m[lL]|mUI/[mlL]+|mmol/[lL]|mmol/mol|nmol/[lL]
              |[µu]?g/d[lL]|µ[lL]|%|fL|fl|pg)
            ",
        )
        .unwrap()
    });
    RE_THRESHOLD.is_match(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_reg() -> AnalyteRegistry {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE analytes(id TEXT PRIMARY KEY, pt_name TEXT NOT NULL);
             CREATE TABLE analyte_aliases(alias TEXT PRIMARY KEY, analyte_id TEXT NOT NULL, source TEXT NOT NULL);
             INSERT INTO analytes(id, pt_name) VALUES('hemoglobina', 'Hemoglobina');
             INSERT INTO analyte_aliases(alias, analyte_id, source) VALUES('Hemoglobina', 'hemoglobina', 'seed');"
        ).unwrap();
        AnalyteRegistry::load_from_db(&conn).unwrap()
    }

    #[test]
    fn confidence_full_when_resolved_with_empty_range() {
        // Vit D / Estradiol / Ferritina rows often have no parseable range
        // token in the row text — the lab prints a tier table elsewhere.
        // When the analyte resolved AND value+unit are present, that row
        // should reach 1.0 confidence; ontology fills the range.
        let resolved = Some("vitamin_d_25oh".to_string());
        let none_range = ParsedRange::none("");
        let c = compute_confidence(&resolved, true, &none_range, true);
        assert!((c - 1.0).abs() < 1e-6, "expected 1.0, got {c}");
    }

    #[test]
    fn confidence_penalizes_unparsed_range_even_when_resolved() {
        // Unparsed != None: the row had range-shaped text but the parser
        // couldn't decode it. That's a real quality signal.
        let resolved = Some("hemoglobina".to_string());
        let bad = ParsedRange::unparsed("garbage");
        let c = compute_confidence(&resolved, true, &bad, true);
        assert!(c < 0.2, "Unparsed should still tank confidence; got {c}");
    }

    #[test]
    fn confidence_penalizes_empty_range_when_unresolved() {
        // No analyte resolution AND no printed range → low confidence.
        let unresolved = None;
        let none_range = ParsedRange::none("");
        let c = compute_confidence(&unresolved, true, &none_range, true);
        assert!(c < 0.85, "expected discounted confidence, got {c}");
    }

    #[test]
    fn peels_a_b_then_remainder() {
        let (range, rest) = peel_range("12.0 - 15.0 12.6 12.9 12.7");
        assert_eq!(range.grammar, RangeGrammar::AB);
        assert_eq!(range.low, Some(12.0));
        assert_eq!(range.high, Some(15.0));
        assert_eq!(rest, "12.6 12.9 12.7");
    }

    #[test]
    fn peels_lt_then_remainder() {
        let (range, rest) = peel_range("< 38 75 25 17");
        assert_eq!(range.grammar, RangeGrammar::Lt);
        assert_eq!(range.high, Some(38.0));
        assert_eq!(rest, "75 25 17");
    }

    #[test]
    fn captures_inline_priors() {
        let mut ctx = SectionContext::default();
        ctx.prior_dates = vec![
            "2025-07-01".into(),
            "2025-04-05".into(),
            "2025-01-13".into(),
        ];
        let reg = empty_reg();
        let row = parse_row(
            "Hemoglobina 13.4 g/dl 12.0 - 15.0 12.6 12.9 12.7",
            &ctx,
            &reg,
        )
        .unwrap();
        assert_eq!(row.value.numeric, Some(13.4));
        assert_eq!(row.ref_low, Some(12.0));
        assert_eq!(row.ref_high, Some(15.0));
        assert_eq!(row.inline_priors.len(), 3);
    }

    #[test]
    fn acronym_with_trailing_dot_parses_value_correctly() {
        // Regression: the parser used to swallow the trailing "." of "V.G.M."
        // into the value, producing value=".    91.4" (unparseable).
        let trimmed = "V.G.M.    91.4";
        let v = trailing_number_or_inequality(trimmed);
        assert_eq!(v, Some("91.4"));
    }

    #[test]
    fn lt_inequality_value_extracted() {
        let v = trailing_number_or_inequality("Colesterol Total    < 190");
        assert_eq!(v, Some("< 190"));
    }

    #[test]
    fn tier_label_lines_are_filtered() {
        let ctx = SectionContext::default();
        let reg = empty_reg();
        // These categorical-tier labels appear under LDL / Vit D / Ferritin
        // and would otherwise be parsed as standalone analyte rows.
        assert!(parse_row("Baixo ou moderado: < 115 mg/dL", &ctx, &reg).is_none());
        assert!(parse_row("Elevado: <100 mg/dL", &ctx, &reg).is_none());
        assert!(parse_row("Muito elevado: < 70 mg/dL", &ctx, &reg).is_none());
        assert!(parse_row("Deficiência: <10", &ctx, &reg).is_none());
        assert!(parse_row("Suficiência: 30 - 100", &ctx, &reg).is_none());
    }

    #[test]
    fn leucograma_paired_value_picks_percent() {
        // The leucograma rows have two units in one line: `%` (primary) and
        // `x 10^3/` (absolute count). We want the PERCENT value as the primary,
        // not the absolute count.
        let mut ctx = SectionContext::default();
        ctx.prior_dates = vec![
            "2025-04-05".into(),
            "2025-01-13".into(),
            "2024-10-04".into(),
        ];
        let reg = empty_reg();

        let row = parse_row(
            "Neutrófilos 61.3 % 5.02 x 103/ 40.00 - 80.00 60.5 61.4 60.0",
            &ctx,
            &reg,
        )
        .unwrap();
        assert_eq!(
            row.value.numeric,
            Some(61.3),
            "value should be the percent (61.3), not the absolute count (5.02)"
        );
        assert_eq!(row.unit.as_deref(), Some("%"));
        assert_eq!(row.ref_low, Some(40.0));
        assert_eq!(row.ref_high, Some(80.0));
        assert_eq!(row.inline_priors.len(), 3);
        assert_eq!(row.inline_priors[0].1, 60.5);
    }

    #[test]
    fn leucograma_paired_value_with_caret_unit() {
        let ctx = SectionContext::default();
        let reg = empty_reg();
        let row = parse_row("Eosinófilos 1.0 % 0.08 x 10^3/ 1.00 - 6.00", &ctx, &reg).unwrap();
        assert_eq!(row.value.numeric, Some(1.0));
        assert_eq!(row.unit.as_deref(), Some("%"));
        assert_eq!(row.ref_low, Some(1.0));
        assert_eq!(row.ref_high, Some(6.0));
    }

    #[test]
    fn eritrocytes_with_pdfium_no_caret_unit() {
        // Eritrócitos uses x 10^6/µl but pdfium drops the caret → "x 106/µl".
        let mut ctx = SectionContext::default();
        ctx.prior_dates = vec![
            "2025-07-01".into(),
            "2025-04-05".into(),
            "2025-01-13".into(),
        ];
        let reg = empty_reg();
        let row = parse_row(
            "Eritrócitos 4.19 x 106/µl 3.80 - 4.80 4.02 4.13 4.06",
            &ctx,
            &reg,
        )
        .unwrap();
        assert_eq!(row.value.numeric, Some(4.19));
        assert_eq!(row.unit.as_deref(), Some("x 10^6/µL"));
        assert_eq!(row.ref_low, Some(3.80));
        assert_eq!(row.ref_high, Some(4.80));
        assert_eq!(row.inline_priors.len(), 3);
    }

    #[test]
    fn standard_percent_row_unaffected() {
        // Hematócrito has only a % value (no absolute count). The peel_range
        // skip must NOT misfire here.
        let ctx = SectionContext::default();
        let reg = empty_reg();
        let row = parse_row("Hematócrito 38.3 % 36.0 - 46.0 36.3 37.2 37.9", &ctx, &reg).unwrap();
        assert_eq!(row.value.numeric, Some(38.3));
        assert_eq!(row.unit.as_deref(), Some("%"));
        assert_eq!(row.ref_low, Some(36.0));
        assert_eq!(row.ref_high, Some(46.0));
    }

    #[test]
    fn ferritina_threshold_lines_are_filtered() {
        let ctx = SectionContext::default();
        let reg = empty_reg();
        // These bleed through pdfium splitting of the "Ferropénia Absoluta no
        // Adulto: Ferritina <30 ng/mL" tier descriptions; previously they got
        // parsed as standalone measurements with value=30.
        assert!(parse_row("Ferritina: <30 ng/mL", &ctx, &reg).is_none());
        assert!(parse_row("Ferritina <30 ng/mL", &ctx, &reg).is_none());
        assert!(parse_row("Ferritina: 30 - 50 ng/mL", &ctx, &reg).is_none());
        assert!(parse_row("Ferritina 30 - 50 ng/mL", &ctx, &reg).is_none());
        assert!(parse_row("Ferritina: <70 ng/mL", &ctx, &reg).is_none());
        assert!(parse_row("Ferritina: 100 - 300 ng/mL", &ctx, &reg).is_none());

        // BUT the real measurement form must NOT be filtered:
        //   "Ferritina 215 ng/mL 30 - 340 …"
        // Single value before the unit, range follows. Different shape.
        let row = parse_row("Ferritina 215 ng/ml 30 - 340", &ctx, &reg);
        assert!(row.is_some(), "real measurement was filtered as threshold");
    }

    #[test]
    fn footnote_and_decision_rule_lines_are_filtered() {
        let ctx = SectionContext::default();
        let reg = empty_reg();
        // Troponin decision-rule lines.
        assert!(parse_row("Se Troponina I (hs) T0 < 100 ng/L", &ctx, &reg).is_none());
        assert!(parse_row("Se Troponina I (hs) T0 100 ng/L", &ctx, &reg).is_none());
        assert!(parse_row("- Se Troponina I (hs) T0 2.00 ng/L", &ctx, &reg).is_none());
        // Asterisk-prefixed footnote markers.
        assert!(parse_row("*VPN: Valor Preditivo Negativo", &ctx, &reg).is_none());
        assert!(parse_row("**VPP: Valor Preditivo Positivo", &ctx, &reg).is_none());
        assert!(parse_row("*Some other footnote", &ctx, &reg).is_none());
    }

    #[test]
    fn age_stratified_reference_lines_are_filtered() {
        let ctx = SectionContext::default();
        let reg = empty_reg();
        // Testosterona Livre age table: each row is a reference for an age
        // bracket, not a measurement.
        assert!(parse_row("< 1 mês 1.5 - 31.0 pg/mL", &ctx, &reg).is_none());
        assert!(parse_row("1 - 3 meses 3.3 - 18.0 pg/mL", &ctx, &reg).is_none());
        assert!(parse_row("3 - 5 meses 0.7 - 14.0 pg/mL", &ctx, &reg).is_none());
        assert!(parse_row("5 - 7 meses 0.4 - 4.8 pg/mL", &ctx, &reg).is_none());
        assert!(parse_row("6 - 9 anos 0.1 - 3.2 pg/mL", &ctx, &reg).is_none());
        assert!(parse_row("Adultos 20-50 anos 8.69 - 54.69 pg/mL", &ctx, &reg).is_none());
        assert!(parse_row("20-50 anos 8.69 - 54.69 pg/mL", &ctx, &reg).is_none());
    }

    #[test]
    fn electroforese_paired_value_picks_percent() {
        // Albumina 62.6 % 4.5 g/dl 3.5 - 5.0 — same shape as the leucograma
        // case, but the absolute column is "<num> g/dl" instead of
        // "<num> x 10^N/". Both forms must skip past the absolute column.
        let ctx = SectionContext::default();
        let reg = empty_reg();
        let row = parse_row("Albumina 62.6 % 4.5 g/dl 3.5 - 5.0", &ctx, &reg).unwrap();
        assert_eq!(
            row.value.numeric,
            Some(62.6),
            "value should be the percent (62.6), not the absolute (4.5)"
        );
        assert_eq!(row.unit.as_deref(), Some("%"));
        assert_eq!(row.ref_low, Some(3.5));
        assert_eq!(row.ref_high, Some(5.0));
    }

    #[test]
    fn standard_g_dl_row_unaffected_by_paired_skip() {
        // Hemoglobina has only g/dl as primary unit (no separate % column).
        // The new RE_PAIRED_ABSOLUTE_LEAD must not misfire here.
        let ctx = SectionContext::default();
        let reg = empty_reg();
        let row = parse_row("Hemoglobina 13.4 g/dl 12.0 - 15.0", &ctx, &reg).unwrap();
        assert_eq!(row.value.numeric, Some(13.4));
        assert_eq!(row.unit.as_deref(), Some("g/dL"));
        assert_eq!(row.ref_low, Some(12.0));
        assert_eq!(row.ref_high, Some(15.0));
    }
}
