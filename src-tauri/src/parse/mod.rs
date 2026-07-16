pub mod canonical;
pub mod header;
pub mod preprocess;
pub mod ranges;
pub mod rows;
pub mod sections;
pub mod units;
pub mod values;

#[cfg(test)]
mod acceptance;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RangeGrammar {
    AB,
    Lt,
    Lte,
    Gt,
    Gte,
    Categorical,
    AgeStrat,
    SexStrat,
    CyclePhase,
    Gestational,
    Qualitative,
    Titer,
    None,
    Unparsed,
}

impl RangeGrammar {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AB => "a_b",
            Self::Lt => "lt",
            Self::Lte => "lte",
            Self::Gt => "gt",
            Self::Gte => "gte",
            Self::Categorical => "categorical",
            Self::AgeStrat => "age_stratified",
            Self::SexStrat => "sex_stratified",
            Self::CyclePhase => "cycle_phase",
            Self::Gestational => "gestational",
            Self::Qualitative => "qualitative",
            Self::Titer => "titer",
            Self::None => "none",
            Self::Unparsed => "unparsed",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParsedValue {
    pub numeric: Option<f64>,
    pub qualitative: Option<String>,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRow {
    pub raw_analyte_text: String,
    pub analyte_id: Option<String>,
    pub value: ParsedValue,
    pub unit: Option<String>,
    pub unit_raw: String,
    pub ref_low: Option<f64>,
    pub ref_high: Option<f64>,
    pub ref_grammar: RangeGrammar,
    pub ref_raw_text: Option<String>,
    pub flag: Option<&'static str>,
    pub method_annotation: Option<String>,
    pub parse_method: &'static str,
    pub confidence: f32,
    pub inline_priors: Vec<(String, f64)>,
}
