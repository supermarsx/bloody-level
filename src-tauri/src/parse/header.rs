use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportHeader {
    pub patient_name: Option<String>,
    pub salutation: Option<String>,
    pub inferred_sex: Option<&'static str>,
    pub collection_date_iso: Option<String>,
    pub emission_date_iso: Option<String>,
    pub age_years: Option<u32>,
    pub inscription_id: Option<String>,
    pub process_id: Option<String>,
    pub origin_id: Option<String>,
    pub clinic_entity: Option<String>,
    pub requesting_physician: Option<String>,
    /// Patient date of birth in ISO YYYY-MM-DD form. Two confidence levels:
    ///   - `Exact`: the PDF literally printed a "Data de Nascimento" / "DOB"
    ///     line we could parse.
    ///   - `Approximate`: derived from `age_years + collection_date`. Day +
    ///     month default to `01` because age has year resolution at best.
    pub dob_iso: Option<String>,
    pub dob_confidence: Option<&'static str>, // "exact" | "approximate"
}

static RE_DATE_DMY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(\d{2})/(\d{2})/(\d{4})\b").unwrap());
static RE_AGE: Lazy<Regex> = Lazy::new(|| Regex::new(r"Idade\s+(\d{1,3})\s+Anos").unwrap());

pub fn extract(text: &str) -> ReportHeader {
    let mut h = ReportHeader::default();

    extract_patient_name(text, &mut h);
    extract_collection_date(text, &mut h);
    extract_emission_date(text, &mut h);
    extract_patient_age(text, &mut h);
    extract_patient_dob(text, &mut h);
    h.inscription_id = capture_after(text, "Nº Inscrição", 60);
    h.process_id = capture_after(text, "Nº Processo", 40);
    h.origin_id = capture_after(text, "Nº Origem", 40);
    extract_clinic_entity(text, &mut h);
    extract_requesting_physician(text, &mut h);
    h.inferred_sex = match h.salutation.as_deref() {
        Some("Sr.") => Some("m"),
        Some("Sra.") => Some("f"),
        _ => None,
    };

    // Fall back to age-derived DOB only when no literal one was found.
    // Approximate values use Jan-1 because age has year resolution at best —
    // we don't pretend to know month/day. UPSERT logic in ingest never
    // overwrites an existing exact value with an approximate one.
    if h.dob_iso.is_none() {
        if let (Some(age), Some(col)) = (h.age_years, h.collection_date_iso.as_deref()) {
            if let Some(approx) = approximate_dob_from_age(age, col) {
                h.dob_iso = Some(approx);
                h.dob_confidence = Some("approximate");
            }
        }
    }

    h
}

fn extract_patient_name(text: &str, h: &mut ReportHeader) {
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    for i in 0..lines.len() {
        if lines[i].starts_with("Exmo Sr.") {
            h.salutation = Some("Sr.".into());
            if i + 1 < lines.len() && !lines[i + 1].is_empty() {
                h.patient_name = Some(lines[i + 1].to_string());
            }
        } else if lines[i].starts_with("Exma Sra.") {
            h.salutation = Some("Sra.".into());
            if i + 1 < lines.len() && !lines[i + 1].is_empty() {
                h.patient_name = Some(lines[i + 1].to_string());
            }
        }
    }
}

/// Snap `end` down to the nearest UTF-8 char boundary at or before `end`.
/// Raw byte slicing on `&str` panics when the index lands inside a multi-byte
/// codepoint (e.g. `idx + 80` falling inside `ã`/`ç`/`õ`). Always route slice
/// ends through this helper.
fn snap_boundary(text: &str, end: usize) -> usize {
    let mut e = end.min(text.len());
    while e > 0 && !text.is_char_boundary(e) {
        e -= 1;
    }
    e
}

fn extract_collection_date(text: &str, h: &mut ReportHeader) {
    if let Some(idx) = text.find("Data de colheita") {
        let end = snap_boundary(text, idx + 80);
        if let Some(c) = RE_DATE_DMY.captures(&text[idx..end]) {
            h.collection_date_iso = Some(format!("{}-{}-{}", &c[3], &c[2], &c[1]));
        }
    }
}

fn extract_emission_date(text: &str, h: &mut ReportHeader) {
    if let Some(idx) = text.find("Data de emissão") {
        let end = snap_boundary(text, idx + 80);
        if let Some(c) = RE_DATE_DMY.captures(&text[idx..end]) {
            h.emission_date_iso = Some(format!("{}-{}-{}", &c[3], &c[2], &c[1]));
        }
    }
}

fn extract_patient_age(text: &str, h: &mut ReportHeader) {
    if let Some(c) = RE_AGE.captures(text) {
        h.age_years = c[1].parse().ok();
    }
}

/// Look for an explicit DOB anchor printed on the report. Portuguese labs
/// don't always include this (CUF / Germano de Sousa typically print only
/// the age), but EU electronic health summaries and some private labs do.
/// Anchors covered (case-insensitive):
///   - "Data de Nascimento" / "Data Nasc" / "Data de Nasc."
///   - "Nascimento"
///   - "DOB" / "Date of Birth" / "Birth Date"
fn extract_patient_dob(text: &str, h: &mut ReportHeader) {
    const ANCHORS: &[&str] = &[
        "Data de Nascimento",
        "Data Nascimento",
        "Data Nasc.",
        "Data Nasc",
        "Nascimento",
        "Date of Birth",
        "Birth Date",
        "DOB",
    ];
    let lower = text.to_lowercase();
    for a in ANCHORS {
        let needle = a.to_lowercase();
        if let Some(idx) = lower.find(&needle) {
            // `idx` from a lowercased copy is byte-aligned with the original
            // because `to_lowercase` only re-encodes per-codepoint and our
            // anchors are ASCII; still, snap_boundary defensively.
            let start = snap_boundary(text, idx);
            let end = snap_boundary(text, start + 80);
            if start > end {
                continue;
            }
            if let Some(c) = RE_DATE_DMY.captures(&text[start..end]) {
                h.dob_iso = Some(format!("{}-{}-{}", &c[3], &c[2], &c[1]));
                h.dob_confidence = Some("exact");
                return;
            }
        }
    }
}

/// Subtract `age_years` whole years from a YYYY-MM-DD collection date and
/// return YYYY-01-01 of the resulting year. Year-resolution only — we
/// deliberately drop month/day because the report's age field doesn't carry
/// them. Returns None if the input collection date is malformed.
pub(crate) fn approximate_dob_from_age(age_years: u32, collection_date_iso: &str) -> Option<String> {
    let year_str = collection_date_iso.get(0..4)?;
    let year: i32 = year_str.parse().ok()?;
    let dob_year = year.checked_sub(age_years as i32)?;
    if dob_year < 1900 || dob_year > 2200 {
        return None;
    }
    Some(format!("{dob_year:04}-01-01"))
}

fn extract_clinic_entity(text: &str, h: &mut ReportHeader) {
    if let Some(idx) = text.find("Entidade") {
        let end = snap_boundary(text, idx + 80);
        let after_start = idx + "Entidade".len();
        if after_start <= end {
            let after = text[after_start..end].trim();
            let line = after.lines().next().unwrap_or(after).trim();
            if !line.is_empty() {
                h.clinic_entity = Some(line.to_string());
            }
        }
    }
}

fn extract_requesting_physician(text: &str, h: &mut ReportHeader) {
    if let Some(idx) = text.find("Requisitado por") {
        let end = snap_boundary(text, idx + 80);
        let after_start = idx + "Requisitado por".len();
        if after_start <= end {
            let after = text[after_start..end].trim();
            let line = after.lines().next().unwrap_or(after).trim();
            if !line.is_empty() {
                h.requesting_physician = Some(line.to_string());
            }
        }
    }
}

fn capture_after(text: &str, anchor: &str, span: usize) -> Option<String> {
    let idx = text.find(anchor)?;
    let start = idx + anchor.len();
    if start > text.len() { return None; }
    let end = snap_boundary(text, start + span);
    if start > end { return None; }
    let chunk = text[start..end].trim_start();
    let token: String = chunk
        .chars()
        .take_while(|c| !c.is_control() && *c != '\n' && *c != '\r')
        .collect();
    let token = token.trim();
    if token.is_empty() { None } else { Some(token.to_string()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patient_male() {
        let txt = "Exmo Sr.\nEDUARDO LUIS COELHO MOTA\nRUA DO BOCAJE Nº11\n";
        let h = extract(txt);
        assert_eq!(h.salutation.as_deref(), Some("Sr."));
        assert_eq!(h.patient_name.as_deref(), Some("EDUARDO LUIS COELHO MOTA"));
        assert_eq!(h.inferred_sex, Some("m"));
    }

    #[test]
    fn date_format_conversion() {
        let txt = "Data de colheita 09/02/2022\nData de emissão 17/02/2022";
        let h = extract(txt);
        assert_eq!(h.collection_date_iso.as_deref(), Some("2022-02-09"));
        assert_eq!(h.emission_date_iso.as_deref(), Some("2022-02-17"));
    }

    #[test]
    fn age() {
        let txt = "Idade 26 Anos";
        let h = extract(txt);
        assert_eq!(h.age_years, Some(26));
    }

    #[test]
    fn does_not_panic_when_anchor_near_eof_with_multibyte() {
        // Regression: `idx + 80` could land inside a multi-byte codepoint
        // (e.g., `ã` in `emissão`), causing `&text[idx..end]` to panic.
        let txt = "Data de emissão 17/02/2022 ã ç õ á é í ó ú";
        let h = extract(txt);
        assert_eq!(h.emission_date_iso.as_deref(), Some("2022-02-17"));
    }

    #[test]
    fn anchor_at_eof_does_not_panic() {
        // `idx + 80` would exceed text length and land inside a multi-byte char.
        let txt = "Data de emissão";
        let h = extract(txt);
        assert!(h.emission_date_iso.is_none());
    }

    // ───── Salutation / patient name ─────────────────────────────────────

    #[test]
    fn patient_female() {
        let txt = "Exma Sra.\nMARIA JOSÉ PINTO\nRua das Flores 12\n";
        let h = extract(txt);
        assert_eq!(h.salutation.as_deref(), Some("Sra."));
        assert_eq!(h.patient_name.as_deref(), Some("MARIA JOSÉ PINTO"));
        assert_eq!(h.inferred_sex, Some("f"));
    }

    #[test]
    fn no_salutation_means_no_inferred_sex() {
        let txt = "ALGUMA PESSOA\nRua qualquer\n";
        let h = extract(txt);
        assert!(h.salutation.is_none());
        assert!(h.patient_name.is_none());
        assert!(h.inferred_sex.is_none());
    }

    #[test]
    fn salutation_with_empty_following_line_skips_name() {
        let txt = "Exmo Sr.\n\nRUA DO BOCAJE\n";
        let h = extract(txt);
        assert_eq!(h.salutation.as_deref(), Some("Sr."));
        assert!(h.patient_name.is_none());
    }

    #[test]
    fn salutation_at_last_line_doesnt_panic() {
        let txt = "Some preamble\nExmo Sr.";
        let h = extract(txt);
        assert_eq!(h.salutation.as_deref(), Some("Sr."));
        assert!(h.patient_name.is_none());
    }

    #[test]
    fn both_salutations_present_takes_last_one() {
        // The loop assigns each match in order, so the later one wins.
        let txt = "Exmo Sr.\nFOO\nExma Sra.\nBAR\n";
        let h = extract(txt);
        assert_eq!(h.salutation.as_deref(), Some("Sra."));
        assert_eq!(h.patient_name.as_deref(), Some("BAR"));
        assert_eq!(h.inferred_sex, Some("f"));
    }

    #[test]
    fn patient_name_with_portuguese_diacritics() {
        let txt = "Exma Sra.\nANTÓNIA GONÇALVES SOUSA AÇORES\nMorada\n";
        let h = extract(txt);
        assert_eq!(
            h.patient_name.as_deref(),
            Some("ANTÓNIA GONÇALVES SOUSA AÇORES")
        );
    }

    // ───── Date parsing ─────────────────────────────────────────────────

    #[test]
    fn collection_date_only() {
        let txt = "Data de colheita 01/01/2025\nOther stuff";
        let h = extract(txt);
        assert_eq!(h.collection_date_iso.as_deref(), Some("2025-01-01"));
        assert!(h.emission_date_iso.is_none());
    }

    #[test]
    fn collection_date_anchor_present_but_no_date() {
        let txt = "Data de colheita\nNo date follows";
        let h = extract(txt);
        assert!(h.collection_date_iso.is_none());
    }

    #[test]
    fn emission_date_anchor_with_garbage() {
        let txt = "Data de emissão XX/YY/ZZZZ";
        let h = extract(txt);
        assert!(h.emission_date_iso.is_none());
    }

    #[test]
    fn date_with_extra_whitespace() {
        let txt = "Data de colheita   05/03/2024";
        let h = extract(txt);
        assert_eq!(h.collection_date_iso.as_deref(), Some("2024-03-05"));
    }

    #[test]
    fn picks_first_date_within_window() {
        let txt = "Data de colheita 09/02/2022 reanalisada em 11/02/2022";
        let h = extract(txt);
        assert_eq!(h.collection_date_iso.as_deref(), Some("2022-02-09"));
    }

    #[test]
    fn date_just_outside_80_byte_window_is_missed() {
        // If the date is past byte 80 from the anchor it's outside the search
        // window. Document the behavior so a future window-size change is
        // an explicit decision.
        let mut txt = String::from("Data de colheita ");
        txt.push_str(&" ".repeat(80));
        txt.push_str("09/02/2022");
        let h = extract(&txt);
        assert!(h.collection_date_iso.is_none());
    }

    #[test]
    fn dates_with_dense_multibyte_padding() {
        // Stress: multiple multi-byte chars inside the 80-byte window.
        let txt = "Data de emissão  ã ç õ á é í ó ú ñ ÁÉÍÓÚÃÕÇ 31/12/1999";
        let h = extract(txt);
        // Date is at byte position past 80 due to multi-byte padding; we
        // assert no panic. The boundary fix is what's being tested here.
        let _ = h.emission_date_iso;
    }

    // ───── Age ───────────────────────────────────────────────────────────

    #[test]
    fn age_no_anchor() {
        let h = extract("nothing about age here");
        assert!(h.age_years.is_none());
    }

    #[test]
    fn age_three_digits() {
        let h = extract("Idade 105 Anos");
        assert_eq!(h.age_years, Some(105));
    }

    #[test]
    fn age_zero() {
        let h = extract("Idade 0 Anos");
        assert_eq!(h.age_years, Some(0));
    }

    #[test]
    fn age_with_extra_spaces() {
        let h = extract("Idade   42   Anos");
        assert_eq!(h.age_years, Some(42));
    }

    #[test]
    fn age_too_many_digits_fails() {
        // Regex accepts 1-3 digits only.
        let h = extract("Idade 9999 Anos");
        assert!(h.age_years.is_none());
    }

    // ───── Clinic / physician (capture_after-shaped fields) ─────────────

    #[test]
    fn clinic_entity_basic() {
        let txt = "Entidade ACSS - SNS\nNext line";
        let h = extract(txt);
        assert_eq!(h.clinic_entity.as_deref(), Some("ACSS - SNS"));
    }

    #[test]
    fn clinic_entity_with_diacritics() {
        let txt = "Entidade Hospital São João\n";
        let h = extract(txt);
        assert_eq!(h.clinic_entity.as_deref(), Some("Hospital São João"));
    }

    #[test]
    fn clinic_entity_anchor_at_eof() {
        let h = extract("Entidade");
        assert!(h.clinic_entity.is_none());
    }

    #[test]
    fn clinic_entity_empty_after_anchor() {
        let h = extract("Entidade   \n");
        assert!(h.clinic_entity.is_none());
    }

    #[test]
    fn requesting_physician_basic() {
        let txt = "Requisitado por Dr. António Silva\nMore text";
        let h = extract(txt);
        assert_eq!(h.requesting_physician.as_deref(), Some("Dr. António Silva"));
    }

    #[test]
    fn requesting_physician_anchor_only() {
        let h = extract("Requisitado por");
        assert!(h.requesting_physician.is_none());
    }

    // ───── ID fields (capture_after) ─────────────────────────────────────

    #[test]
    fn inscription_id_extracted() {
        let txt = "Nº Inscrição 1234567\nfollowing";
        let h = extract(txt);
        assert_eq!(h.inscription_id.as_deref(), Some("1234567"));
    }

    #[test]
    fn process_id_extracted() {
        let txt = "Nº Processo PR-001\n";
        let h = extract(txt);
        assert_eq!(h.process_id.as_deref(), Some("PR-001"));
    }

    #[test]
    fn origin_id_extracted() {
        let txt = "Nº Origem ORIG-99\n";
        let h = extract(txt);
        assert_eq!(h.origin_id.as_deref(), Some("ORIG-99"));
    }

    #[test]
    fn id_anchor_at_eof_returns_none() {
        let h = extract("Nº Inscrição");
        assert!(h.inscription_id.is_none());
    }

    #[test]
    fn id_with_only_whitespace_through_eof_returns_none() {
        // No non-whitespace token within the search window -> None.
        let h = extract("Nº Processo    \n   ");
        assert!(h.process_id.is_none());
    }

    #[test]
    fn id_skips_leading_whitespace_including_newlines() {
        // trim_start() strips newlines too, so the next non-whitespace token
        // is what gets captured. Document the behavior.
        let h = extract("Nº Processo    \nnext\n");
        assert_eq!(h.process_id.as_deref(), Some("next"));
    }

    #[test]
    fn id_stops_at_newline_not_at_first_space() {
        // capture_after takes everything up to a control char (incl. newline),
        // then trims. So multi-token tails on a single line are kept.
        let txt = "Nº Inscrição AB-12 / X\n";
        let h = extract(txt);
        assert_eq!(h.inscription_id.as_deref(), Some("AB-12 / X"));
    }

    // ───── Empty / pathological inputs ──────────────────────────────────

    #[test]
    fn empty_input_yields_default_header() {
        let h = extract("");
        assert!(h.patient_name.is_none());
        assert!(h.salutation.is_none());
        assert!(h.collection_date_iso.is_none());
        assert!(h.emission_date_iso.is_none());
        assert!(h.age_years.is_none());
        assert!(h.inscription_id.is_none());
        assert!(h.process_id.is_none());
        assert!(h.origin_id.is_none());
        assert!(h.clinic_entity.is_none());
        assert!(h.requesting_physician.is_none());
        assert!(h.inferred_sex.is_none());
    }

    #[test]
    fn whitespace_only_input() {
        let h = extract("   \n\n\t  \r\n");
        assert!(h.patient_name.is_none());
        assert!(h.collection_date_iso.is_none());
    }

    #[test]
    fn input_with_only_multibyte_chars() {
        // Every byte index that lands inside one of these chars must be snapped.
        let txt = "ãçõáéíóúÁÉÍÓÚÃÕÇ";
        let h = extract(txt);
        assert!(h.collection_date_iso.is_none());
    }

    #[test]
    fn anchor_immediately_followed_by_multibyte() {
        // The byte just after "Data de colheita" is the start of `ã`. The
        // window read must not split that codepoint.
        let txt = "Data de colheitaãçõ no date here";
        let h = extract(txt);
        assert!(h.collection_date_iso.is_none());
    }

    #[test]
    fn very_long_input_does_not_panic() {
        let mut txt = String::from("Data de colheita 01/01/2025\n");
        // Append lots of multi-byte content past the date.
        for _ in 0..1000 {
            txt.push_str("ãçõáéíóú ");
        }
        let h = extract(&txt);
        assert_eq!(h.collection_date_iso.as_deref(), Some("2025-01-01"));
    }

    // ───── snap_boundary helper ──────────────────────────────────────────

    #[test]
    fn snap_boundary_clamps_to_text_len() {
        let s = "abc";
        assert_eq!(snap_boundary(s, 100), 3);
    }

    #[test]
    fn snap_boundary_steps_back_off_multibyte() {
        // "ã" is 2 bytes (0xC3 0xA3). Index 1 lands inside it.
        let s = "ã";
        assert_eq!(s.len(), 2);
        assert_eq!(snap_boundary(s, 1), 0);
        assert_eq!(snap_boundary(s, 2), 2);
    }

    #[test]
    fn snap_boundary_on_ascii_is_identity() {
        let s = "hello";
        assert_eq!(snap_boundary(s, 0), 0);
        assert_eq!(snap_boundary(s, 3), 3);
        assert_eq!(snap_boundary(s, 5), 5);
    }

    #[test]
    fn snap_boundary_zero_is_always_valid() {
        assert_eq!(snap_boundary("ãçõ", 0), 0);
        assert_eq!(snap_boundary("", 0), 0);
        assert_eq!(snap_boundary("", 100), 0);
    }

    #[test]
    fn snap_boundary_handles_4byte_codepoint() {
        // U+1F600 (😀) is 4 bytes in UTF-8: F0 9F 98 80
        let s = "a😀b";
        assert_eq!(s.len(), 6);
        // bytes 2, 3, 4 are mid-codepoint -> snap to 1
        assert_eq!(snap_boundary(s, 2), 1);
        assert_eq!(snap_boundary(s, 3), 1);
        assert_eq!(snap_boundary(s, 4), 1);
        // byte 5 is the start of 'b' -> keep it
        assert_eq!(snap_boundary(s, 5), 5);
    }

    // ───── Combined / realistic ──────────────────────────────────────────

    #[test]
    fn full_realistic_header_extracts_all_fields() {
        let txt = "\
Exmo Sr.
JOÃO PEDRO ALMEIDA
Rua das Flores 42
Idade 47 Anos
Data de colheita 03/04/2024
Data de emissão 05/04/2024
Nº Inscrição 7654321
Nº Processo PR-2024-09
Nº Origem ORIG-A1
Entidade ACSS - SNS
Requisitado por Dra. Maria Açoreana
";
        let h = extract(txt);
        assert_eq!(h.salutation.as_deref(), Some("Sr."));
        assert_eq!(h.patient_name.as_deref(), Some("JOÃO PEDRO ALMEIDA"));
        assert_eq!(h.inferred_sex, Some("m"));
        assert_eq!(h.age_years, Some(47));
        assert_eq!(h.collection_date_iso.as_deref(), Some("2024-04-03"));
        assert_eq!(h.emission_date_iso.as_deref(), Some("2024-04-05"));
        assert_eq!(h.inscription_id.as_deref(), Some("7654321"));
        assert_eq!(h.process_id.as_deref(), Some("PR-2024-09"));
        assert_eq!(h.origin_id.as_deref(), Some("ORIG-A1"));
        assert_eq!(h.clinic_entity.as_deref(), Some("ACSS - SNS"));
        assert_eq!(
            h.requesting_physician.as_deref(),
            Some("Dra. Maria Açoreana")
        );
    }

    // ───── DOB extraction & approximation ───────────────────────────────

    #[test]
    fn dob_explicit_data_de_nascimento() {
        let txt = "Data de Nascimento 12/05/1985\nIdade 39 Anos\nData de colheita 03/06/2024";
        let h = extract(txt);
        assert_eq!(h.dob_iso.as_deref(), Some("1985-05-12"));
        assert_eq!(h.dob_confidence, Some("exact"));
    }

    #[test]
    fn dob_falls_back_to_age_derivation() {
        let txt = "Idade 39 Anos\nData de colheita 03/06/2024";
        let h = extract(txt);
        // 2024 - 39 = 1985, day defaults to Jan 1.
        assert_eq!(h.dob_iso.as_deref(), Some("1985-01-01"));
        assert_eq!(h.dob_confidence, Some("approximate"));
    }

    #[test]
    fn dob_explicit_overrides_age_fallback() {
        // Age says 1985, but the explicit DOB line is canonical.
        let txt = "Data de Nascimento 02/02/1980\nIdade 39 Anos\nData de colheita 03/06/2024";
        let h = extract(txt);
        assert_eq!(h.dob_iso.as_deref(), Some("1980-02-02"));
        assert_eq!(h.dob_confidence, Some("exact"));
    }

    #[test]
    fn dob_not_set_without_age_or_explicit() {
        let txt = "Data de colheita 03/06/2024";
        let h = extract(txt);
        assert!(h.dob_iso.is_none());
        assert!(h.dob_confidence.is_none());
    }

    #[test]
    fn dob_dob_anchor_english() {
        let txt = "DOB 31/12/1970";
        let h = extract(txt);
        assert_eq!(h.dob_iso.as_deref(), Some("1970-12-31"));
    }

    #[test]
    fn approximate_dob_handles_year_only() {
        assert_eq!(
            approximate_dob_from_age(40, "2025-08-19"),
            Some("1985-01-01".to_string())
        );
    }

    #[test]
    fn approximate_dob_rejects_garbage_collection_date() {
        assert!(approximate_dob_from_age(40, "garbage").is_none());
    }

    #[test]
    fn approximate_dob_rejects_unrealistic_year() {
        // age 200 + 2024 = 1824 → before our 1900 floor → None.
        assert!(approximate_dob_from_age(200, "2024-01-01").is_none());
    }

    #[test]
    fn missing_anchors_yield_none_for_those_fields_only() {
        let txt = "Exmo Sr.\nFOO BAR\nIdade 30 Anos\n";
        let h = extract(txt);
        assert_eq!(h.patient_name.as_deref(), Some("FOO BAR"));
        assert_eq!(h.age_years, Some(30));
        assert!(h.collection_date_iso.is_none());
        assert!(h.inscription_id.is_none());
        assert!(h.clinic_entity.is_none());
    }
}
