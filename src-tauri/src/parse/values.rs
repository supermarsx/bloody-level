use once_cell::sync::Lazy;
use regex::Regex;

static RE_NUMBER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^-?\d{1,3}(?:[.,\s]\d{3})*(?:[.,]\d+)?$").unwrap()
});
static RE_INEQ: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([<>]=?)\s*(-?\d+(?:[.,]\d+)?)$").unwrap()
});

pub const QUALITATIVE_TERMS: &[&str] = &[
    "Positivo",
    "Negativo",
    "Não detectado",
    "Reactivo",
    "Não Reactivo",
    "Indeterminado",
    "Reativo",
    "Não Reativo",
];

pub fn parse_numeric_decimal(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() { return None; }

    let last_comma = t.rfind(',');
    let last_dot = t.rfind('.');

    let normalized: String = match (last_comma, last_dot) {
        (Some(c), Some(d)) if c > d => {
            t.chars().filter(|&ch| ch != '.').map(|ch| if ch == ',' { '.' } else { ch }).collect()
        }
        (Some(_), Some(_)) => t.chars().filter(|&ch| ch != ',').collect(),
        (Some(_), None) => t.replace(',', "."),
        (None, _) => t.to_string(),
    };

    if !RE_NUMBER.is_match(t) && normalized.parse::<f64>().is_err() {
        return None;
    }
    normalized.parse::<f64>().ok()
}

pub fn parse_qualitative_value(s: &str) -> Option<&'static str> {
    let t = s.trim();
    for q in QUALITATIVE_TERMS {
        if t.eq_ignore_ascii_case(q) {
            return Some(q);
        }
        if normalize_diacritics(t).eq_ignore_ascii_case(&normalize_diacritics(q)) {
            return Some(q);
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InequalityOp {
    Lt,
    Lte,
    Gt,
    Gte,
}

pub fn parse_inequality_value(s: &str) -> Option<(InequalityOp, f64)> {
    let cap = RE_INEQ.captures(s.trim())?;
    let op = match &cap[1] {
        "<" => InequalityOp::Lt,
        "<=" => InequalityOp::Lte,
        ">" => InequalityOp::Gt,
        ">=" => InequalityOp::Gte,
        _ => return None,
    };
    let v = parse_numeric_decimal(&cap[2])?;
    Some((op, v))
}

fn normalize_diacritics(s: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    s.nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn parses_pt_decimal()   { assert_eq!(parse_numeric_decimal("13,8"), Some(13.8)); }
    #[test] fn parses_en_decimal()   { assert_eq!(parse_numeric_decimal("13.8"), Some(13.8)); }
    #[test] fn parses_thousands_pt() { assert_eq!(parse_numeric_decimal("1.234,5"), Some(1234.5)); }
    #[test] fn parses_thousands_en() { assert_eq!(parse_numeric_decimal("1,234.5"), Some(1234.5)); }
    #[test] fn parses_int()          { assert_eq!(parse_numeric_decimal("353"), Some(353.0)); }
    #[test] fn parses_lt()           { assert_eq!(parse_inequality_value("< 38").map(|(_,v)| v), Some(38.0)); }
    #[test] fn parses_gte()          { assert_eq!(parse_inequality_value(">= 60").map(|(_,v)| v), Some(60.0)); }
    #[test] fn parses_qual()         { assert_eq!(parse_qualitative_value("Positivo"), Some("Positivo")); }
    #[test] fn parses_qual_nodia()   { assert_eq!(parse_qualitative_value("Nao detectado"), Some("Não detectado")); }

    // ───── Numeric edge cases ────────────────────────────────────────────

    #[test] fn rejects_empty_value()        { assert!(parse_numeric_decimal("").is_none()); }
    #[test] fn rejects_whitespace_only()    { assert!(parse_numeric_decimal("   ").is_none()); }
    #[test] fn rejects_pure_alphabetic()    { assert!(parse_numeric_decimal("abc").is_none()); }
    #[test] fn rejects_mixed_alphanumeric() { assert!(parse_numeric_decimal("12abc").is_none()); }
    #[test] fn rejects_double_decimal()     { assert!(parse_numeric_decimal("1..2").is_none()); }
    #[test] fn rejects_double_comma()       { assert!(parse_numeric_decimal("1,,2").is_none()); }
    #[test] fn rejects_just_a_dot()         { assert!(parse_numeric_decimal(".").is_none()); }
    #[test] fn rejects_just_a_comma()       { assert!(parse_numeric_decimal(",").is_none()); }
    #[test] fn rejects_just_a_minus()       { assert!(parse_numeric_decimal("-").is_none()); }
    #[test] fn rejects_internal_space()     { assert!(parse_numeric_decimal("12 .5").is_none()); }

    #[test]
    fn parses_negative_decimal_pt() {
        assert_eq!(parse_numeric_decimal("-3,5"), Some(-3.5));
    }

    #[test]
    fn parses_negative_int() {
        assert_eq!(parse_numeric_decimal("-100"), Some(-100.0));
    }

    #[test]
    fn parses_zero_in_both_locales() {
        assert_eq!(parse_numeric_decimal("0"), Some(0.0));
        assert_eq!(parse_numeric_decimal("0.0"), Some(0.0));
        assert_eq!(parse_numeric_decimal("0,0"), Some(0.0));
    }

    #[test]
    fn parses_leading_plus_accepted_via_f64_fallback() {
        // Documented quirk: the regex grammar only allows `-`, but the f64
        // fallback parser accepts `+5`. This is harmless for lab data
        // (which never carries leading `+`) but worth pinning.
        assert_eq!(parse_numeric_decimal("+5"), Some(5.0));
    }

    #[test]
    fn parses_trims_outer_whitespace() {
        assert_eq!(parse_numeric_decimal("   42   "), Some(42.0));
        assert_eq!(parse_numeric_decimal("\t13.8\n"), Some(13.8));
    }

    #[test]
    fn parses_long_decimal() {
        let v = parse_numeric_decimal("3.14159265358979").unwrap();
        assert!((v - std::f64::consts::PI).abs() < 1e-12);
    }

    #[test]
    fn parses_pt_thousands_no_decimal() {
        assert_eq!(parse_numeric_decimal("1.234"), Some(1.234));
    }

    #[test]
    fn parses_huge_number() {
        assert_eq!(parse_numeric_decimal("999999"), Some(999_999.0));
    }

    // ───── Inequality edge cases ─────────────────────────────────────────

    #[test]
    fn parses_lt_no_space() {
        assert_eq!(parse_inequality_value("<38").map(|(_, v)| v), Some(38.0));
    }

    #[test]
    fn parses_lte() {
        let (op, v) = parse_inequality_value("<= 0.05").unwrap();
        assert_eq!(op, InequalityOp::Lte);
        assert_eq!(v, 0.05);
    }

    #[test]
    fn parses_gt() {
        let (op, v) = parse_inequality_value("> 100").unwrap();
        assert_eq!(op, InequalityOp::Gt);
        assert_eq!(v, 100.0);
    }

    #[test]
    fn parses_inequality_with_pt_decimal() {
        let (op, v) = parse_inequality_value("< 0,5").unwrap();
        assert_eq!(op, InequalityOp::Lt);
        assert_eq!(v, 0.5);
    }

    #[test]
    fn parses_inequality_with_negative() {
        let (_, v) = parse_inequality_value("< -2").unwrap();
        assert_eq!(v, -2.0);
    }

    #[test]
    fn rejects_inequality_without_value() {
        assert!(parse_inequality_value("<").is_none());
        assert!(parse_inequality_value(">=  ").is_none());
    }

    #[test]
    fn rejects_invalid_inequality_op() {
        assert!(parse_inequality_value("=< 5").is_none());
        assert!(parse_inequality_value("=> 5").is_none());
        assert!(parse_inequality_value("!= 5").is_none());
    }

    #[test]
    fn rejects_inequality_with_extra_tokens() {
        assert!(parse_inequality_value("< 5 mg/dL").is_none());
    }

    // ───── Qualitative edge cases ────────────────────────────────────────

    #[test]
    fn qualitative_case_insensitive() {
        assert_eq!(parse_qualitative_value("POSITIVO"), Some("Positivo"));
        assert_eq!(parse_qualitative_value("positivo"), Some("Positivo"));
        assert_eq!(parse_qualitative_value("NeGaTiVo"), Some("Negativo"));
    }

    #[test]
    fn qualitative_with_diacritics_match() {
        assert_eq!(parse_qualitative_value("Não detectado"), Some("Não detectado"));
    }

    #[test]
    fn qualitative_reactivo_variants() {
        assert_eq!(parse_qualitative_value("Reactivo"), Some("Reactivo"));
        assert_eq!(parse_qualitative_value("Reativo"), Some("Reativo"));
        assert_eq!(parse_qualitative_value("Não Reactivo"), Some("Não Reactivo"));
        assert_eq!(parse_qualitative_value("Nao Reativo"), Some("Não Reativo"));
    }

    #[test]
    fn qualitative_unknown_term_rejected() {
        assert!(parse_qualitative_value("Pendente").is_none());
        assert!(parse_qualitative_value("Anormal").is_none());
        assert!(parse_qualitative_value("").is_none());
    }

    #[test]
    fn qualitative_trims_whitespace() {
        assert_eq!(parse_qualitative_value("  Positivo  "), Some("Positivo"));
    }
}
