use once_cell::sync::Lazy;
use std::collections::HashMap;

static CANON: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    let pairs: &[(&[&str], &str)] = &[
        (&["g/dl", "g/dL"], "g/dL"),
        (&["mg/dl", "mg/dL"], "mg/dL"),
        (&["mmol/l", "mmol/L"], "mmol/L"),
        (&["mmol/mol"], "mmol/mol"),
        (&["mUI/l", "mUI/L", "mU/L", "uIU/mL"], "mUI/L"),
        (&["mUI/ml", "mUI/mL", "mIU/mL"], "mUI/mL"),
        (&["U/l", "U/L", "UI/l", "UI/L"], "U/L"),
        (&["UI/ml", "UI/mL", "IU/mL"], "UI/mL"),
        (&["ng/dl", "ng/dL"], "ng/dL"),
        (&["ng/ml", "ng/mL"], "ng/mL"),
        (&["ng/l", "ng/L"], "ng/L"),
        (&["pg/ml", "pg/mL"], "pg/mL"),
        (&["µg/dl", "ug/dL", "µg/dL"], "µg/dL"),
        (&["nmol/l", "nmol/L"], "nmol/L"),
        (&["fl", "fL"], "fL"),
        (&["pg"], "pg"),
        (&["%"], "%"),
        (&["µl", "uL", "ul", "µL"], "µL"),
        // Caret variants from typesetting AND no-caret variants from pdfium
        // (which extracts the superscript "³" inline as "3").
        (
            &["x 10^3/µl", "x 103/µl", "× 10³/µL", "x 10^3/uL", "x10^3/uL"],
            "x 10^3/µL",
        ),
        (
            &["x 10^6/µl", "x 106/µl", "× 10⁶/µL", "x 10^6/uL", "x10^6/uL"],
            "x 10^6/µL",
        ),
        // Truncated form used in leucograma sub-rows where µl is implicit
        // from the parent Leucócitos row.
        (&["x 10^3/", "x 103/"], "x 10^3/µL"),
        (
            &["ml/min/1,73 m2", "mL/min/1.73 m²", "ml/min/1.73m2"],
            "mL/min/1.73 m²",
        ),
    ];
    for (vars, canon) in pairs {
        for v in *vars {
            m.insert(*v, *canon);
        }
    }
    m
});

pub fn normalize_unit(raw: &str) -> Option<&'static str> {
    let t = raw.trim();
    if t.is_empty() {
        return None;
    }
    CANON.get(t).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_g_dl() {
        assert_eq!(normalize_unit("g/dl"), Some("g/dL"));
    }
    #[test]
    fn canonicalizes_mg_dl() {
        assert_eq!(normalize_unit("mg/dL"), Some("mg/dL"));
    }
    #[test]
    fn canonicalizes_truncated_leuco() {
        assert_eq!(normalize_unit("x 10^3/"), Some("x 10^3/µL"));
    }
}
