// Section detection — methods 11..20 (real implementations).

use once_cell::sync::Lazy;
use regex::Regex;

const SECTIONS: &[(&str, &str)] = &[
    ("HEMATOLOGIA", "HEMATOLOGIA"),
    ("PATOLOGIA QUÍMICA", "PATOLOGIA QUÍMICA"),
    ("IMUNOLOGIA", "IMUNOLOGIA"),
    ("MICROBIOLOGIA", "MICROBIOLOGIA"),
    ("SEROLOGIA", "SEROLOGIA"),
    ("ENDOCRINOLOGIA", "ENDOCRINOLOGIA"),
];

const SUBSECTIONS: &[&str] = &[
    "HEMOGRAMA",
    "Eritrograma",
    "Leucograma",
    "Trombocitograma",
    "FIBRINÓLISE",
    "METABOLISMO DO FERRO",
    "MATURAÇÃO MEGALOBLÁSTICA",
    "PROTEÍNAS",
    "FUNÇÃO RENAL",
    "FUNÇÃO HEPATO-BILIAR",
    "ENZIMOLOGIA",
    "BIOMARCADORES CARDÍACOS",
    "METABOLISMO DOS HIDRATOS DE CARBONO",
    "METABOLISMO LÍPIDICO",
    "METABOLISMO PURÍNICO",
    "METABOLISMO FOSFO-CÁLCICO",
    "EQUILÍBRIO HIDROELECTROLÍTICO",
    "EIXO HIPÓFISO-TIROIDEU",
    "EIXO HIPÓFISO-GONADAL",
    "EIXO HIPÓFISO-CORTO-SUPRA-RENAL",
    "RITMO HORMONAL",
    "DOENÇAS ATÓPICAS",
    "DOENÇAS ATÓPICAS I",
];

static RE_DATE_DMY: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\d{2})/(\d{2})/(\d{4})\b").unwrap());

pub fn detect_section(line: &str) -> Option<&'static str> {
    let t = line.trim();
    for (needle, canon) in SECTIONS {
        if t == *needle || t.starts_with(needle) {
            return Some(*canon);
        }
    }
    None
}

pub fn detect_subsection(line: &str) -> Option<&'static str> {
    let t = line.trim();
    for sub in SUBSECTIONS {
        if t == *sub {
            return Some(*sub);
        }
    }
    None
}

#[derive(Debug, Clone, Default)]
pub struct SectionContext {
    pub section: Option<&'static str>,
    pub subsection: Option<&'static str>,
    pub prior_dates: Vec<String>, // ISO format yyyy-mm-dd
    expecting_prior_dates: bool,
}

impl SectionContext {
    pub fn update(&mut self, line: &str) {
        if let Some(s) = detect_section(line) {
            self.section = Some(s);
            self.subsection = None;
            self.prior_dates.clear();
            self.expecting_prior_dates = false;
        } else if let Some(sub) = detect_subsection(line) {
            self.subsection = Some(sub);
        }

        if line.contains("Resultados anteriores") {
            self.prior_dates.clear();
            let dates = scan_dates_iso(line);
            if dates.is_empty() {
                self.expecting_prior_dates = true;
            } else {
                self.prior_dates = dates;
                self.expecting_prior_dates = false;
            }
        } else if self.expecting_prior_dates {
            let dates = scan_dates_iso(line);
            if !dates.is_empty() {
                self.prior_dates = dates;
                self.expecting_prior_dates = false;
            }
        }
    }
}

pub fn scan_dates_iso(line: &str) -> Vec<String> {
    RE_DATE_DMY
        .captures_iter(line)
        .map(|c| format!("{}-{}-{}", &c[3], &c[2], &c[1]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects() {
        let mut ctx = SectionContext::default();
        ctx.update("HEMATOLOGIA");
        assert_eq!(ctx.section, Some("HEMATOLOGIA"));
        ctx.update("Eritrograma");
        assert_eq!(ctx.subsection, Some("Eritrograma"));
        ctx.update("PATOLOGIA QUÍMICA");
        assert_eq!(ctx.section, Some("PATOLOGIA QUÍMICA"));
        assert_eq!(ctx.subsection, None);
    }

    #[test]
    fn captures_prior_dates_inline() {
        let mut ctx = SectionContext::default();
        ctx.update("HEMATOLOGIA Resultado Valores de Referência Resultados anteriores 01/07/2025 05/04/2025 13/01/2025");
        assert_eq!(
            ctx.prior_dates,
            vec!["2025-07-01", "2025-04-05", "2025-01-13"]
        );
    }

    #[test]
    fn captures_prior_dates_next_line() {
        let mut ctx = SectionContext::default();
        ctx.update("HEMATOLOGIA Resultado Valores de Referência Resultados anteriores");
        ctx.update("01/07/2025 05/04/2025");
        assert_eq!(ctx.prior_dates, vec!["2025-07-01", "2025-04-05"]);
    }

    // ───── Section detection edge cases ──────────────────────────────────

    #[test]
    fn no_section_detected_for_random_text() {
        assert!(detect_section("Hemoglobina 13.4 g/dl").is_none());
        assert!(detect_section("").is_none());
        assert!(detect_section("   ").is_none());
    }

    #[test]
    fn section_with_trailing_text_after_keyword() {
        // Sections may be followed by column headers on the same logical line.
        assert_eq!(
            detect_section("HEMATOLOGIA Resultado Valores de Referência"),
            Some("HEMATOLOGIA")
        );
    }

    #[test]
    fn section_with_leading_whitespace() {
        assert_eq!(detect_section("   HEMATOLOGIA"), Some("HEMATOLOGIA"));
    }

    #[test]
    fn section_lowercase_does_not_match() {
        assert!(detect_section("hematologia").is_none());
    }

    #[test]
    fn section_substring_match_inside_word_does_not_match() {
        // detect_section uses starts_with, not contains, so a section keyword
        // appearing mid-sentence is NOT a section header.
        assert!(detect_section("Algo HEMATOLOGIA aqui").is_none());
    }

    #[test]
    fn detects_imunologia_microbiologia_serologia_endocrinologia() {
        assert_eq!(detect_section("IMUNOLOGIA"), Some("IMUNOLOGIA"));
        assert_eq!(detect_section("MICROBIOLOGIA"), Some("MICROBIOLOGIA"));
        assert_eq!(detect_section("SEROLOGIA"), Some("SEROLOGIA"));
        assert_eq!(detect_section("ENDOCRINOLOGIA"), Some("ENDOCRINOLOGIA"));
    }

    #[test]
    fn detects_patologia_quimica_with_diacritic() {
        assert_eq!(
            detect_section("PATOLOGIA QUÍMICA"),
            Some("PATOLOGIA QUÍMICA")
        );
    }

    // ───── Subsection edge cases ─────────────────────────────────────────

    #[test]
    fn subsection_must_match_exactly() {
        // detect_subsection uses strict equality, no starts_with.
        assert_eq!(detect_subsection("Eritrograma"), Some("Eritrograma"));
        assert!(detect_subsection("Eritrograma extras").is_none());
    }

    #[test]
    fn subsection_unrelated_text_returns_none() {
        assert!(detect_subsection("nothing").is_none());
        assert!(detect_subsection("").is_none());
    }

    #[test]
    fn detects_complex_subsections() {
        assert_eq!(
            detect_subsection("FUNÇÃO HEPATO-BILIAR"),
            Some("FUNÇÃO HEPATO-BILIAR")
        );
        assert_eq!(
            detect_subsection("METABOLISMO FOSFO-CÁLCICO"),
            Some("METABOLISMO FOSFO-CÁLCICO")
        );
        assert_eq!(detect_subsection("RITMO HORMONAL"), Some("RITMO HORMONAL"));
    }

    // ───── SectionContext lifecycle ──────────────────────────────────────

    #[test]
    fn section_change_clears_subsection() {
        let mut ctx = SectionContext::default();
        ctx.update("HEMATOLOGIA");
        ctx.update("Eritrograma");
        // Same-section update with a non-section header keeps subsection.
        ctx.update("more text");
        assert_eq!(ctx.subsection, Some("Eritrograma"));
        // Switching sections clears the subsection.
        ctx.update("PATOLOGIA QUÍMICA");
        assert_eq!(ctx.section, Some("PATOLOGIA QUÍMICA"));
        assert!(ctx.subsection.is_none());
    }

    #[test]
    fn section_with_inline_prior_dates_keeps_dates() {
        // The single line "HEMATOLOGIA Resultados anteriores DATES" both
        // resets section state and captures the prior dates.
        let mut ctx = SectionContext::default();
        ctx.update("HEMATOLOGIA Resultados anteriores 01/01/2025");
        assert_eq!(ctx.section, Some("HEMATOLOGIA"));
        assert_eq!(ctx.prior_dates, vec!["2025-01-01"]);
    }

    #[test]
    fn subsection_does_not_reset_prior_dates() {
        let mut ctx = SectionContext::default();
        ctx.update("HEMATOLOGIA Resultados anteriores 01/01/2025");
        ctx.update("Eritrograma");
        // Subsection update should not wipe prior dates.
        assert_eq!(ctx.prior_dates, vec!["2025-01-01"]);
        assert_eq!(ctx.subsection, Some("Eritrograma"));
    }

    #[test]
    fn expecting_prior_dates_only_consumes_first_date_line() {
        let mut ctx = SectionContext::default();
        ctx.update("Resultados anteriores");
        // expecting_prior_dates is true; first line with dates wins.
        ctx.update("01/07/2025");
        ctx.update("05/04/2025"); // would be a different field, ignored
        assert_eq!(ctx.prior_dates, vec!["2025-07-01"]);
    }

    #[test]
    fn expecting_prior_dates_skips_non_date_intermediate_line() {
        let mut ctx = SectionContext::default();
        ctx.update("Resultados anteriores");
        ctx.update("Some intermediate text");
        ctx.update("01/07/2025 05/04/2025");
        assert_eq!(ctx.prior_dates, vec!["2025-07-01", "2025-04-05"]);
    }

    #[test]
    fn second_resultados_anteriores_replaces_first() {
        let mut ctx = SectionContext::default();
        ctx.update("Resultados anteriores 01/01/2024");
        ctx.update("Resultados anteriores 05/05/2025");
        assert_eq!(ctx.prior_dates, vec!["2025-05-05"]);
    }

    #[test]
    fn default_context_has_no_state() {
        let ctx = SectionContext::default();
        assert!(ctx.section.is_none());
        assert!(ctx.subsection.is_none());
        assert!(ctx.prior_dates.is_empty());
    }

    // ───── scan_dates_iso ────────────────────────────────────────────────

    #[test]
    fn scan_dates_iso_no_dates() {
        assert!(scan_dates_iso("nothing here").is_empty());
        assert!(scan_dates_iso("").is_empty());
    }

    #[test]
    fn scan_dates_iso_single() {
        assert_eq!(scan_dates_iso("date 09/02/2022 here"), vec!["2022-02-09"]);
    }

    #[test]
    fn scan_dates_iso_multiple_with_text_between() {
        let dates = scan_dates_iso("Foo 01/02/2024 bar 03/04/2025 baz");
        assert_eq!(dates, vec!["2024-02-01", "2025-04-03"]);
    }

    #[test]
    fn scan_dates_iso_rejects_malformed() {
        // RE_DATE_DMY needs \b boundaries and exactly 2/2/4 digits.
        assert!(scan_dates_iso("9/2/2022").is_empty()); // single digits
        assert!(scan_dates_iso("01/02/202").is_empty()); // 3-digit year
        assert!(scan_dates_iso("01/02/20245").is_empty()); // 5-digit year
    }

    #[test]
    fn scan_dates_iso_does_not_validate_calendar() {
        // We don't validate that 99/99/9999 is a real date.
        assert_eq!(scan_dates_iso("99/99/9999"), vec!["9999-99-99"]);
    }
}
