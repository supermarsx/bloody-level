use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologySeed {
    #[serde(rename = "$schema_version", default)]
    pub schema_version: u32,
    pub lab_meta: serde_json::Value,
    pub sections: Vec<String>,
    pub subsections: Vec<String>,
    pub panels: serde_json::Value,
    pub analytes: Vec<AnalyteDef>,
    pub range_examples: serde_json::Value,
    pub unit_variants: serde_json::Value,
    pub parser_hazards: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyteDef {
    pub id: String,
    pub pt_name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub loinc: Option<String>,
    pub section: String,
    #[serde(default)]
    pub subsection: Option<String>,
    #[serde(default)]
    pub panel: Option<String>,
    #[serde(default)]
    pub expected_units: Vec<String>,
    #[serde(default)]
    pub range_grammar: Vec<String>,
    #[serde(default)]
    pub method_annotation: Option<String>,
    #[serde(default)]
    pub default_ref: Option<serde_json::Value>,
    #[serde(default)]
    pub sex_dependent: bool,
    #[serde(default)]
    pub age_dependent: bool,
    #[serde(default)]
    pub cycle_dependent: bool,
    #[serde(default)]
    pub is_qualitative: bool,
    #[serde(default)]
    pub is_derived: bool,
    #[serde(default)]
    pub is_panel_header: bool,
    #[serde(default)]
    pub paired_value: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub high_means: Option<String>,
    #[serde(default)]
    pub low_means: Option<String>,
    #[serde(default)]
    pub unit_notes: Option<String>,
    /// Tier table: e.g., Vit D `[{label:"Deficiência",max:10},{label:"Insuficiência",min:10,max:30},…]`.
    #[serde(default)]
    pub categorical_tiers: Option<serde_json::Value>,
    /// Cycle-phase reference: Estradiol `{follicular_-12d:[11,69], ovulation_-1d:[146,526], …}`.
    #[serde(default)]
    pub cycle_phases: Option<serde_json::Value>,
}

pub fn load(path: &Path) -> AppResult<OntologySeed> {
    let bytes = std::fs::read(path).map_err(AppError::from)?;
    let seed: OntologySeed = serde_json::from_slice(&bytes).map_err(AppError::from)?;
    Ok(seed)
}

pub fn install_into_db(seed: &OntologySeed, conn: &rusqlite::Connection) -> AppResult<usize> {
    // Defensively roll back any leaked transaction state before we start a new one.
    if !conn.is_autocommit() {
        let _ = conn.execute_batch("ROLLBACK");
    }

    // Use rusqlite's RAII Transaction so a failure rolls back automatically.
    let tx = conn.unchecked_transaction()?;
    // Aliases are referenced from analytes only — safe to wipe seed-source ones.
    // We do NOT delete analyte rows: `results.analyte_id REFERENCES analytes(id)`
    // would block the delete once any parsed rows exist. Instead we UPSERT so
    // existing IDs get refreshed in place and FK references stay valid.
    tx.execute("DELETE FROM analyte_aliases WHERE source = 'seed'", [])?;

    let mut inserted = 0usize;
    {
        // Seed-driven INSERT/UPSERT — explicitly tags the row with
        // `source='seed'` and the `WHERE analytes.source = 'seed'` clause
        // on the UPDATE arm makes user-created/edited rows immune to
        // refresh. New analytes added through the UI have `source='user'`,
        // so they remain untouched on every reload.
        let mut stmt = tx.prepare(
            "INSERT INTO analytes(
                id, pt_name, loinc, section, subsection, panel,
                is_qualitative, is_derived, paired_value, is_panel_header,
                expected_units_json, default_ref_json,
                sex_dependent, age_dependent, cycle_dependent, method_annotation,
                description, high_means, low_means, unit_notes,
                categorical_tiers_json, cycle_phases_json, source
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,'seed')
            ON CONFLICT(id) DO UPDATE SET
                pt_name              = excluded.pt_name,
                loinc                = excluded.loinc,
                section              = excluded.section,
                subsection           = excluded.subsection,
                panel                = excluded.panel,
                is_qualitative       = excluded.is_qualitative,
                is_derived           = excluded.is_derived,
                paired_value         = excluded.paired_value,
                is_panel_header      = excluded.is_panel_header,
                expected_units_json  = excluded.expected_units_json,
                default_ref_json     = excluded.default_ref_json,
                sex_dependent        = excluded.sex_dependent,
                age_dependent        = excluded.age_dependent,
                cycle_dependent      = excluded.cycle_dependent,
                method_annotation    = excluded.method_annotation,
                description          = excluded.description,
                high_means           = excluded.high_means,
                low_means            = excluded.low_means,
                unit_notes           = excluded.unit_notes,
                categorical_tiers_json = excluded.categorical_tiers_json,
                cycle_phases_json    = excluded.cycle_phases_json
                WHERE analytes.source = 'seed'",
        )?;
        let mut alias_stmt = tx.prepare(
            "INSERT OR IGNORE INTO analyte_aliases(alias, analyte_id, source) VALUES(?1, ?2, 'seed')",
        )?;

        for a in &seed.analytes {
            stmt.execute(rusqlite::params![
                a.id,
                a.pt_name,
                a.loinc,
                a.section,
                a.subsection,
                a.panel,
                a.is_qualitative as i32,
                a.is_derived as i32,
                a.paired_value as i32,
                a.is_panel_header as i32,
                serde_json::to_string(&a.expected_units)?,
                a.default_ref.as_ref().map(|v| v.to_string()),
                a.sex_dependent as i32,
                a.age_dependent as i32,
                a.cycle_dependent as i32,
                a.method_annotation,
                a.description,
                a.high_means,
                a.low_means,
                a.unit_notes,
                a.categorical_tiers.as_ref().map(|v| v.to_string()),
                a.cycle_phases.as_ref().map(|v| v.to_string()),
            ])?;
            inserted += 1;
            alias_stmt.execute(rusqlite::params![a.pt_name, a.id])?;
            for alias in &a.aliases {
                alias_stmt.execute(rusqlite::params![alias, a.id])?;
            }
        }
    }

    tx.commit()?;
    Ok(inserted)
}
