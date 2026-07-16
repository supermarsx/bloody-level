use std::collections::HashMap;

use rusqlite::Connection;
use strsim::normalized_levenshtein;

use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct AnalyteRegistry {
    canonical_pt: HashMap<String, String>,
    aliases: HashMap<String, String>,
}

impl AnalyteRegistry {
    pub fn load_from_db(conn: &Connection) -> AppResult<Self> {
        let mut r = Self {
            canonical_pt: HashMap::new(),
            aliases: HashMap::new(),
        };
        let mut s = conn.prepare("SELECT id, pt_name FROM analytes")?;
        let rows = s.query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            Ok((id, name))
        })?;
        for row in rows {
            let (id, name) = row?;
            r.canonical_pt.insert(fold(&name), id);
        }
        let mut a = conn.prepare("SELECT alias, analyte_id FROM analyte_aliases")?;
        let arows = a.query_map([], |row| {
            let alias: String = row.get(0)?;
            let id: String = row.get(1)?;
            Ok((alias, id))
        })?;
        for row in arows {
            let (alias, id) = row?;
            r.aliases.insert(fold(&alias), id);
        }
        Ok(r)
    }

    pub fn resolve(&self, raw: &str) -> Option<&str> {
        let key = fold(raw);
        if let Some(id) = self.canonical_pt.get(&key) {
            return Some(id.as_str());
        }
        if let Some(id) = self.aliases.get(&key) {
            return Some(id.as_str());
        }

        // Methodology-suffix fallback: many lab printouts append a standard
        // / equation name in square brackets (e.g., "TFGe [CKD-EPI 2009]",
        // "Hemoglobina Glicada (A1c) [NGSP]"). The bracket content is the
        // *method*, not the parameter — strip and try the parameter alone.
        let stripped = strip_method_brackets(raw);
        if stripped != raw {
            let key2 = fold(&stripped);
            if let Some(id) = self.canonical_pt.get(&key2) {
                return Some(id.as_str());
            }
            if let Some(id) = self.aliases.get(&key2) {
                return Some(id.as_str());
            }
        }

        // Fuzzy fallback (>= 0.92 normalized Levenshtein) against canonical names.
        let mut best: Option<(&str, f64)> = None;
        for (k, id) in &self.canonical_pt {
            let s = normalized_levenshtein(&key, k);
            if s >= 0.92 && best.map_or(true, |(_, b)| s > b) {
                best = Some((id.as_str(), s));
            }
        }
        for (k, id) in &self.aliases {
            let s = normalized_levenshtein(&key, k);
            if s >= 0.92 && best.map_or(true, |(_, b)| s > b) {
                best = Some((id.as_str(), s));
            }
        }
        best.map(|(id, _)| id)
    }
}

fn strip_method_brackets(s: &str) -> String {
    use once_cell::sync::Lazy;
    use regex::Regex;
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*\[[^\]]+\]").unwrap());
    RE.replace_all(s, "").trim().to_string()
}

/// Fold a name for ontology lookup:
///   - strip diacritics
///   - lowercase
///   - drop interior dots so "V.G.M." matches "VGM" / "vgm"
///   - collapse whitespace
pub fn fold(s: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    s.nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .collect::<String>()
        .to_lowercase()
        .replace('.', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_is_normalized() {
        assert_eq!(fold("Hemoglobina"), "hemoglobina");
        assert_eq!(fold("Tireoestimulina  (TSH)"), "tireoestimulina (tsh)");
        assert_eq!(fold("Ácido Fólico"), "acido folico");
    }

    #[test]
    fn fold_collapses_acronym_dots() {
        assert_eq!(fold("V.G.M."), "vgm");
        assert_eq!(fold("VGM"), "vgm");
        assert_eq!(fold("v.g.m"), "vgm");
        assert_eq!(fold("C.M.H.G."), "cmhg");
        assert_eq!(fold("R.D.W."), "rdw");
    }

    #[test]
    fn strips_methodology_brackets() {
        assert_eq!(strip_method_brackets("TFGe [CKD-EPI 2009]"), "TFGe");
        assert_eq!(
            strip_method_brackets("Hemoglobina Glicada (A1c) [NGSP]"),
            "Hemoglobina Glicada (A1c)"
        );
        assert_eq!(strip_method_brackets("Plain Name"), "Plain Name");
        assert_eq!(strip_method_brackets("Foo [m1] [m2]"), "Foo");
    }
}
