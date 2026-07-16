use once_cell::sync::Lazy;
use regex::Regex;

use super::values::parse_numeric_decimal;
use super::RangeGrammar;

#[derive(Debug, Clone)]
pub struct ParsedRange {
    pub low: Option<f64>,
    pub high: Option<f64>,
    pub grammar: RangeGrammar,
    pub raw: String,
}

impl ParsedRange {
    pub fn unparsed(raw: impl Into<String>) -> Self {
        Self {
            low: None,
            high: None,
            grammar: RangeGrammar::Unparsed,
            raw: raw.into(),
        }
    }
    pub fn none(raw: impl Into<String>) -> Self {
        Self {
            low: None,
            high: None,
            grammar: RangeGrammar::None,
            raw: raw.into(),
        }
    }
}

static RE_AB: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(-?\d+(?:[.,]\d+)?)\s*[-–—]\s*(-?\d+(?:[.,]\d+)?)\s*$").unwrap());
static RE_LTE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*<=\s*(-?\d+(?:[.,]\d+)?)\s*$").unwrap());
static RE_GTE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*>=\s*(-?\d+(?:[.,]\d+)?)\s*$").unwrap());
static RE_LT: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*<\s*(-?\d+(?:[.,]\d+)?)\s*$").unwrap());
static RE_GT: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*>\s*(-?\d+(?:[.,]\d+)?)\s*$").unwrap());
static RE_TITER: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*1\s*:\s*\d+\s*$").unwrap());

// Diacritic-folded, lowercased keyword stems — the matcher folds the input
// the same way before checking. Keeps us robust to "Deficiencia" vs
// "Deficiência", colon present or absent, casing variations.
const CATEGORICAL_KEYWORDS_FOLDED: &[&str] = &[
    "deficiencia",
    "insuficiencia",
    "suficiencia",
    "toxicidade",
    "baixo ou moderado",
    "elevado:",
    "muito elevado",
    "ferropenia",
];

const CYCLE_PHASE_KEYWORDS_FOLDED: &[&str] = &[
    "fase folicular",
    "ovulacao",
    "fase luteinica",
    "luteinica",
    "pos-menopausa",
    "posmenopausa",
];

fn fold_for_match(s: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    s.nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .collect::<String>()
        .to_lowercase()
}

pub fn parse_range_a_b(s: &str) -> Option<ParsedRange> {
    let cap = RE_AB.captures(s)?;
    let lo = parse_numeric_decimal(&cap[1])?;
    let hi = parse_numeric_decimal(&cap[2])?;
    Some(ParsedRange {
        low: Some(lo),
        high: Some(hi),
        grammar: RangeGrammar::AB,
        raw: s.into(),
    })
}

pub fn parse_range_lt(s: &str) -> Option<ParsedRange> {
    let cap = RE_LT.captures(s)?;
    let v = parse_numeric_decimal(&cap[1])?;
    Some(ParsedRange {
        low: None,
        high: Some(v),
        grammar: RangeGrammar::Lt,
        raw: s.into(),
    })
}

pub fn parse_range_lte(s: &str) -> Option<ParsedRange> {
    let cap = RE_LTE.captures(s)?;
    let v = parse_numeric_decimal(&cap[1])?;
    Some(ParsedRange {
        low: None,
        high: Some(v),
        grammar: RangeGrammar::Lte,
        raw: s.into(),
    })
}

pub fn parse_range_gt(s: &str) -> Option<ParsedRange> {
    let cap = RE_GT.captures(s)?;
    let v = parse_numeric_decimal(&cap[1])?;
    Some(ParsedRange {
        low: Some(v),
        high: None,
        grammar: RangeGrammar::Gt,
        raw: s.into(),
    })
}

pub fn parse_range_gte(s: &str) -> Option<ParsedRange> {
    let cap = RE_GTE.captures(s)?;
    let v = parse_numeric_decimal(&cap[1])?;
    Some(ParsedRange {
        low: Some(v),
        high: None,
        grammar: RangeGrammar::Gte,
        raw: s.into(),
    })
}

fn parse_range_categorical(s: &str) -> Option<ParsedRange> {
    let folded = fold_for_match(s);
    if CATEGORICAL_KEYWORDS_FOLDED
        .iter()
        .any(|k| folded.contains(k))
    {
        return Some(ParsedRange {
            low: None,
            high: None,
            grammar: RangeGrammar::Categorical,
            raw: s.into(),
        });
    }
    None
}

fn parse_range_age_stratified(s: &str) -> Option<ParsedRange> {
    let needles = ["Adultos", "anos", "meses", "semanas", "dias"];
    let hits = needles.iter().filter(|n| s.contains(*n)).count();
    if hits >= 2 {
        Some(ParsedRange {
            low: None,
            high: None,
            grammar: RangeGrammar::AgeStrat,
            raw: s.into(),
        })
    } else {
        None
    }
}

fn parse_range_sex_stratified(s: &str) -> Option<ParsedRange> {
    let needles = ["Homens", "Mulheres", "Sexo masculino", "Sexo feminino"];
    if needles.iter().any(|n| s.contains(n)) {
        Some(ParsedRange {
            low: None,
            high: None,
            grammar: RangeGrammar::SexStrat,
            raw: s.into(),
        })
    } else {
        None
    }
}

fn parse_range_cycle_phase(s: &str) -> Option<ParsedRange> {
    let folded = fold_for_match(s);
    if CYCLE_PHASE_KEYWORDS_FOLDED
        .iter()
        .any(|k| folded.contains(k))
    {
        Some(ParsedRange {
            low: None,
            high: None,
            grammar: RangeGrammar::CyclePhase,
            raw: s.into(),
        })
    } else {
        None
    }
}

fn parse_range_gestational(s: &str) -> Option<ParsedRange> {
    if s.contains("Trimestre") {
        Some(ParsedRange {
            low: None,
            high: None,
            grammar: RangeGrammar::Gestational,
            raw: s.into(),
        })
    } else {
        None
    }
}

fn parse_range_qualitative(s: &str) -> Option<ParsedRange> {
    if super::values::parse_qualitative_value(s.trim()).is_some() {
        Some(ParsedRange {
            low: None,
            high: None,
            grammar: RangeGrammar::Qualitative,
            raw: s.into(),
        })
    } else {
        None
    }
}

fn parse_range_titer(s: &str) -> Option<ParsedRange> {
    if RE_TITER.is_match(s) {
        Some(ParsedRange {
            low: None,
            high: None,
            grammar: RangeGrammar::Titer,
            raw: s.into(),
        })
    } else {
        None
    }
}

pub fn try_parse_any(s: &str) -> ParsedRange {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return ParsedRange::none("");
    }
    if let Some(r) = parse_range_lte(trimmed) {
        return r;
    }
    if let Some(r) = parse_range_gte(trimmed) {
        return r;
    }
    if let Some(r) = parse_range_a_b(trimmed) {
        return r;
    }
    if let Some(r) = parse_range_lt(trimmed) {
        return r;
    }
    if let Some(r) = parse_range_gt(trimmed) {
        return r;
    }
    if let Some(r) = parse_range_titer(trimmed) {
        return r;
    }
    if let Some(r) = parse_range_qualitative(trimmed) {
        return r;
    }
    if let Some(r) = parse_range_categorical(trimmed) {
        return r;
    }
    if let Some(r) = parse_range_cycle_phase(trimmed) {
        return r;
    }
    if let Some(r) = parse_range_gestational(trimmed) {
        return r;
    }
    if let Some(r) = parse_range_age_stratified(trimmed) {
        return r;
    }
    if let Some(r) = parse_range_sex_stratified(trimmed) {
        return r;
    }
    ParsedRange::unparsed(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ab() {
        let r = parse_range_a_b("13.0 - 17.0").unwrap();
        assert_eq!(r.low, Some(13.0));
        assert_eq!(r.high, Some(17.0));
    }
    #[test]
    fn ab_pt() {
        let r = parse_range_a_b("11,54 - 54,49").unwrap();
        assert_eq!(r.high, Some(54.49));
    }
    #[test]
    fn lt() {
        assert_eq!(parse_range_lt("< 190").unwrap().high, Some(190.0));
    }
    #[test]
    fn gte() {
        assert_eq!(parse_range_gte(">= 60").unwrap().low, Some(60.0));
    }
    #[test]
    fn cat() {
        assert_eq!(
            parse_range_categorical("Deficiência: <10").unwrap().grammar,
            RangeGrammar::Categorical
        );
    }
    #[test]
    fn cycle() {
        assert_eq!(
            parse_range_cycle_phase("Fase folicular (-12 d)")
                .unwrap()
                .grammar,
            RangeGrammar::CyclePhase
        );
    }
    #[test]
    fn ges() {
        assert_eq!(
            parse_range_gestational("1º Trimestre").unwrap().grammar,
            RangeGrammar::Gestational
        );
    }
    #[test]
    fn ord() {
        assert_eq!(try_parse_any("13.0 - 17.0").grammar, RangeGrammar::AB);
    }

    // ───── A-B range edge cases ─────────────────────────────────────────

    #[test]
    fn ab_with_em_dash() {
        // PDFs sometimes use Unicode em-dash (—) instead of ASCII hyphen.
        let r = parse_range_a_b("13.0 — 17.0").unwrap();
        assert_eq!(r.low, Some(13.0));
        assert_eq!(r.high, Some(17.0));
    }

    #[test]
    fn ab_with_en_dash() {
        let r = parse_range_a_b("4.5 – 11.0").unwrap();
        assert_eq!(r.low, Some(4.5));
        assert_eq!(r.high, Some(11.0));
    }

    #[test]
    fn ab_no_spaces_around_dash() {
        let r = parse_range_a_b("4.5-11.0").unwrap();
        assert_eq!(r.low, Some(4.5));
        assert_eq!(r.high, Some(11.0));
    }

    #[test]
    fn ab_with_negative_low() {
        let r = parse_range_a_b("-2 - 2").unwrap();
        assert_eq!(r.low, Some(-2.0));
        assert_eq!(r.high, Some(2.0));
    }

    #[test]
    fn ab_rejects_text_around_range() {
        assert!(parse_range_a_b("Ref: 13 - 17 mg/dL").is_none());
    }

    #[test]
    fn ab_rejects_single_value() {
        assert!(parse_range_a_b("13.0").is_none());
    }

    #[test]
    fn ab_rejects_three_values() {
        assert!(parse_range_a_b("1 - 2 - 3").is_none());
    }

    #[test]
    fn ab_inverted_range_still_parses() {
        // We don't validate ordering here; just that both numbers parse.
        let r = parse_range_a_b("17.0 - 13.0").unwrap();
        assert_eq!(r.low, Some(17.0));
        assert_eq!(r.high, Some(13.0));
    }

    // ───── Inequality range edge cases ──────────────────────────────────

    #[test]
    fn lt_pt_decimal() {
        assert_eq!(parse_range_lt("< 0,5").unwrap().high, Some(0.5));
    }

    #[test]
    fn lt_no_space() {
        assert_eq!(parse_range_lt("<5").unwrap().high, Some(5.0));
    }

    #[test]
    fn lte_picks_lte_not_lt() {
        // Order matters in try_parse_any: <= must match before <.
        let r = try_parse_any("<= 10");
        assert_eq!(r.grammar, RangeGrammar::Lte);
        assert_eq!(r.high, Some(10.0));
    }

    #[test]
    fn gte_picks_gte_not_gt() {
        let r = try_parse_any(">= 60");
        assert_eq!(r.grammar, RangeGrammar::Gte);
        assert_eq!(r.low, Some(60.0));
    }

    #[test]
    fn lt_rejects_trailing_text() {
        assert!(parse_range_lt("< 5 mg/dL").is_none());
    }

    // ───── Titer ─────────────────────────────────────────────────────────

    #[test]
    fn titer_basic() {
        let r = parse_range_titer("1:160").unwrap();
        assert_eq!(r.grammar, RangeGrammar::Titer);
    }

    #[test]
    fn titer_with_spaces() {
        assert_eq!(
            parse_range_titer("1 : 80").unwrap().grammar,
            RangeGrammar::Titer
        );
    }

    #[test]
    fn titer_rejects_non_titer() {
        assert!(parse_range_titer("2:160").is_none());
        assert!(parse_range_titer("1:abc").is_none());
    }

    // ───── Categorical / cycle phase / age / sex / gestational ──────────

    #[test]
    fn categorical_no_diacritics() {
        // Folded match means "Deficiencia" without accent works too.
        assert_eq!(
            parse_range_categorical("Deficiencia: <10").unwrap().grammar,
            RangeGrammar::Categorical
        );
    }

    #[test]
    fn categorical_uppercase() {
        assert_eq!(
            parse_range_categorical("FERROPENIA").unwrap().grammar,
            RangeGrammar::Categorical
        );
    }

    #[test]
    fn categorical_multi_keyword() {
        // Only one keyword needs to match.
        let r = parse_range_categorical("Suficiência (≥30) Toxicidade (>100)").unwrap();
        assert_eq!(r.grammar, RangeGrammar::Categorical);
    }

    #[test]
    fn categorical_no_match_returns_none() {
        assert!(parse_range_categorical("Normal").is_none());
        assert!(parse_range_categorical("").is_none());
    }

    #[test]
    fn cycle_phase_ovulation_no_diacritics() {
        assert_eq!(
            parse_range_cycle_phase("Ovulação (-1 d)").unwrap().grammar,
            RangeGrammar::CyclePhase
        );
    }

    #[test]
    fn cycle_phase_lutein() {
        assert_eq!(
            parse_range_cycle_phase("Fase luteínica (+8 d)")
                .unwrap()
                .grammar,
            RangeGrammar::CyclePhase
        );
    }

    #[test]
    fn cycle_phase_pos_menopausa() {
        assert_eq!(
            parse_range_cycle_phase("Pós-menopausa").unwrap().grammar,
            RangeGrammar::CyclePhase
        );
    }

    #[test]
    fn age_stratified_needs_two_needles() {
        // "Adultos" + "anos" in same string -> AgeStrat.
        let r = parse_range_age_stratified("Adultos: 1-99 anos").unwrap();
        assert_eq!(r.grammar, RangeGrammar::AgeStrat);
    }

    #[test]
    fn age_stratified_one_needle_not_enough() {
        assert!(parse_range_age_stratified("12 anos").is_none());
        assert!(parse_range_age_stratified("Adultos saudáveis").is_none());
    }

    #[test]
    fn sex_stratified_homens() {
        let r = parse_range_sex_stratified("Homens: 13-17 g/dL").unwrap();
        assert_eq!(r.grammar, RangeGrammar::SexStrat);
    }

    #[test]
    fn sex_stratified_mulheres() {
        let r = parse_range_sex_stratified("Mulheres: 12-15 g/dL").unwrap();
        assert_eq!(r.grammar, RangeGrammar::SexStrat);
    }

    #[test]
    fn sex_stratified_formal_phrasing() {
        let r = parse_range_sex_stratified("Sexo masculino: x").unwrap();
        assert_eq!(r.grammar, RangeGrammar::SexStrat);
    }

    #[test]
    fn gestational_trimestre() {
        let r = parse_range_gestational("2º Trimestre").unwrap();
        assert_eq!(r.grammar, RangeGrammar::Gestational);
    }

    #[test]
    fn gestational_no_trimestre_means_none() {
        assert!(parse_range_gestational("Gravidez").is_none());
    }

    // ───── try_parse_any dispatch ───────────────────────────────────────

    #[test]
    fn try_parse_any_empty_returns_none() {
        let r = try_parse_any("");
        assert_eq!(r.grammar, RangeGrammar::None);
    }

    #[test]
    fn try_parse_any_whitespace_returns_none() {
        let r = try_parse_any("   \t  ");
        assert_eq!(r.grammar, RangeGrammar::None);
    }

    #[test]
    fn try_parse_any_unparsable_returns_unparsed() {
        let r = try_parse_any("complete gibberish ¿?");
        assert_eq!(r.grammar, RangeGrammar::Unparsed);
    }

    #[test]
    fn try_parse_any_qualitative() {
        let r = try_parse_any("Negativo");
        assert_eq!(r.grammar, RangeGrammar::Qualitative);
    }

    #[test]
    fn try_parse_any_titer() {
        let r = try_parse_any("1:160");
        assert_eq!(r.grammar, RangeGrammar::Titer);
    }

    #[test]
    fn try_parse_any_lte_takes_priority_over_lt() {
        // Important: "<= 10" must NOT match the < regex first.
        let r = try_parse_any("<= 10");
        assert_eq!(r.grammar, RangeGrammar::Lte);
    }

    #[test]
    fn try_parse_any_gte_takes_priority_over_gt() {
        let r = try_parse_any(">= 5");
        assert_eq!(r.grammar, RangeGrammar::Gte);
    }

    #[test]
    fn try_parse_any_strips_outer_whitespace() {
        let r = try_parse_any("   13.0 - 17.0  \n");
        assert_eq!(r.grammar, RangeGrammar::AB);
    }
}
