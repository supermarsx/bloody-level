use rusqlite::OptionalExtension;
use serde::Serialize;
use serde_json::json;
use tauri::State;

use crate::commands::audit;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Serialize)]
pub struct AnalyteInfo {
    pub id: String,
    pub pt_name: String,
    pub loinc: Option<String>,
    pub section: String,
    pub subsection: Option<String>,
    pub panel: Option<String>,
    pub method_annotation: Option<String>,
    pub expected_units: Vec<String>,
    pub default_ref_json: Option<String>,
    pub sex_dependent: bool,
    pub age_dependent: bool,
    pub cycle_dependent: bool,
    pub is_derived: bool,
    pub is_qualitative: bool,
    /// True for ontology entries that label a group of analytes on the PDF
    /// rather than being a measurable test themselves (e.g. "Ionograma sérico"
    /// heads the Na/K/Cl trio). The frontend uses this to render an
    /// info-only page instead of an empty-looking timeseries.
    pub is_panel_header: bool,
    pub description: Option<String>,
    pub high_means: Option<String>,
    pub low_means: Option<String>,
    pub unit_notes: Option<String>,
    pub categorical_tiers_json: Option<String>,
    pub cycle_phases_json: Option<String>,
    pub aliases: Vec<String>,
}

#[tauri::command]
pub async fn analyte_info(
    state: State<'_, AppState>,
    analyte_id: String,
) -> AppResult<AnalyteInfo> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;

    let info = db
        .conn
        .query_row(
            "SELECT id, pt_name, loinc, section, subsection, panel, method_annotation,
                    expected_units_json, default_ref_json,
                    sex_dependent, age_dependent, cycle_dependent, is_derived, is_qualitative,
                    description, high_means, low_means, unit_notes,
                    categorical_tiers_json, cycle_phases_json, is_panel_header
             FROM analytes WHERE id = ?1",
            [&analyte_id],
            |r| {
                let units_json: String = r.get(7)?;
                let units: Vec<String> = serde_json::from_str(&units_json).unwrap_or_default();
                Ok(AnalyteInfo {
                    id: r.get(0)?,
                    pt_name: r.get(1)?,
                    loinc: r.get(2)?,
                    section: r.get(3)?,
                    subsection: r.get(4)?,
                    panel: r.get(5)?,
                    method_annotation: r.get(6)?,
                    expected_units: units,
                    default_ref_json: r.get(8)?,
                    sex_dependent: r.get::<_, i64>(9)? != 0,
                    age_dependent: r.get::<_, i64>(10)? != 0,
                    cycle_dependent: r.get::<_, i64>(11)? != 0,
                    is_derived: r.get::<_, i64>(12)? != 0,
                    is_qualitative: r.get::<_, i64>(13)? != 0,
                    is_panel_header: r.get::<_, Option<i64>>(20)?.unwrap_or(0) != 0,
                    description: r.get(14)?,
                    high_means: r.get(15)?,
                    low_means: r.get(16)?,
                    unit_notes: r.get(17)?,
                    categorical_tiers_json: r.get(18)?,
                    cycle_phases_json: r.get(19)?,
                    aliases: vec![],
                })
            },
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("analyte {analyte_id}")))?;

    let mut info = info;
    let mut alias_stmt = db.conn.prepare(
        "SELECT alias FROM analyte_aliases WHERE analyte_id = ?1 ORDER BY source DESC, alias",
    )?;
    let aliases: Vec<String> = alias_stmt
        .query_map([&analyte_id], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    info.aliases = aliases;

    Ok(info)
}

#[derive(Serialize)]
pub struct AnalyteOntologyEntry {
    pub id: String,
    pub pt_name: String,
    pub section: String,
    pub subsection: Option<String>,
    pub panel: Option<String>,
    pub loinc: Option<String>,
    pub method_annotation: Option<String>,
    pub expected_units: Vec<String>,
    pub default_ref_json: Option<String>,
    pub categorical_tiers_json: Option<String>,
    pub cycle_phases_json: Option<String>,
    pub sex_dependent: bool,
    pub age_dependent: bool,
    pub cycle_dependent: bool,
    pub is_qualitative: bool,
    pub is_derived: bool,
    pub is_panel_header: bool,
    pub paired_value: bool,
    /// Whether the bundled descriptions live on the row. Surfaced so the
    /// management UI can call out incomplete entries at a glance.
    pub has_description: bool,
    pub has_high_means: bool,
    pub has_low_means: bool,
    pub has_unit_notes: bool,
    pub alias_count: i64,
    /// Number of `results` rows in the DB that have ever been linked to
    /// this analyte — proxy for "is this in active use here". Excludes
    /// inline-prior copies.
    pub result_count: i64,
    /// `seed` (bundled JSON) or `user` (created via the UI). Drives the
    /// editing affordances — user rows are fully editable / deletable,
    /// seed rows can be edited but `reload_ontology` will overwrite the
    /// changes.
    pub source: String,
}

/// Single round-trip for the Ontology management tab. Returns one row per
/// analyte registered in the database, with the descriptive flags + a
/// usage counter. The actual long-form fields (description, high_means,
/// etc.) are fetched per-row via `analyte_info` — keeping this query
/// light enough that the page can render a 138-row table instantly.
#[tauri::command]
pub async fn list_ontology_entries(
    state: State<'_, AppState>,
) -> AppResult<Vec<AnalyteOntologyEntry>> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;

    let mut stmt = db.conn.prepare(
        "SELECT a.id, a.pt_name, a.section, a.subsection, a.panel, a.loinc,
                a.method_annotation, a.expected_units_json,
                a.default_ref_json, a.categorical_tiers_json, a.cycle_phases_json,
                a.sex_dependent, a.age_dependent, a.cycle_dependent,
                a.is_qualitative, a.is_derived, a.is_panel_header, a.paired_value,
                a.description, a.high_means, a.low_means, a.unit_notes,
                (SELECT COUNT(*) FROM analyte_aliases al WHERE al.analyte_id = a.id) AS alias_count,
                (SELECT COUNT(*) FROM results res
                    WHERE res.analyte_id = a.id AND res.inline_prior_pdf = 0) AS result_count,
                a.source
         FROM analytes a
         ORDER BY a.section, a.subsection, a.pt_name COLLATE NOCASE",
    )?;
    let rows: Vec<AnalyteOntologyEntry> = stmt
        .query_map([], |r| {
            let units_json: String = r.get(7)?;
            let units: Vec<String> = serde_json::from_str(&units_json).unwrap_or_default();
            let desc: Option<String> = r.get(18)?;
            let hi: Option<String> = r.get(19)?;
            let lo: Option<String> = r.get(20)?;
            let un: Option<String> = r.get(21)?;
            Ok(AnalyteOntologyEntry {
                id: r.get(0)?,
                pt_name: r.get(1)?,
                section: r.get(2)?,
                subsection: r.get(3)?,
                panel: r.get(4)?,
                loinc: r.get(5)?,
                method_annotation: r.get(6)?,
                expected_units: units,
                default_ref_json: r.get(8)?,
                categorical_tiers_json: r.get(9)?,
                cycle_phases_json: r.get(10)?,
                sex_dependent: r.get::<_, i64>(11)? != 0,
                age_dependent: r.get::<_, i64>(12)? != 0,
                cycle_dependent: r.get::<_, i64>(13)? != 0,
                is_qualitative: r.get::<_, i64>(14)? != 0,
                is_derived: r.get::<_, i64>(15)? != 0,
                is_panel_header: r.get::<_, i64>(16)? != 0,
                paired_value: r.get::<_, i64>(17)? != 0,
                has_description: desc.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false),
                has_high_means: hi.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false),
                has_low_means: lo.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false),
                has_unit_notes: un.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false),
                alias_count: r.get(22)?,
                result_count: r.get(23)?,
                source: r.get(24)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

// ────────────────────────────────────────────────────────────────────────────
// Editing — create / update / delete + alias management
//
// Source semantics:
//   - `seed` rows are owned by the bundled JSON. They CAN be edited via
//     `update_analyte` (so the user can tweak descriptions / refs without
//     forking the seed), but `reload_ontology` will overwrite the changes.
//     The UI flags this with a "your edits will be lost on next reload"
//     banner so the user knows what they're getting into.
//   - `user` rows are fully theirs — created here, edited freely, deleted
//     freely. The seed install loop never touches them.
//
// Aliases: any new alias added through this API is tagged source='user' so
// it survives `reload_ontology` (which only wipes seed-source aliases).
// ────────────────────────────────────────────────────────────────────────────

use serde::Deserialize;

#[derive(Deserialize)]
pub struct AnalyteWriteArgs {
    pub id: String,
    pub pt_name: String,
    pub section: String,
    pub subsection: Option<String>,
    pub panel: Option<String>,
    pub loinc: Option<String>,
    pub method_annotation: Option<String>,
    /// Pass an array of unit strings (e.g. `["g/dl"]`).
    #[serde(default)]
    pub expected_units: Vec<String>,
    /// Stringified JSON like `{"m":[13,17],"f":[12,15]}` or `{"all":[null,500]}`.
    /// `None` clears the column.
    pub default_ref_json: Option<String>,
    pub categorical_tiers_json: Option<String>,
    pub cycle_phases_json: Option<String>,
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
    pub description: Option<String>,
    pub high_means: Option<String>,
    pub low_means: Option<String>,
    pub unit_notes: Option<String>,
    /// Aliases the new/updated analyte should resolve under. The command
    /// merges these into `analyte_aliases` with source='user'; pre-existing
    /// matching aliases on the same target are kept untouched. Doesn't
    /// remove aliases the user didn't list.
    #[serde(default)]
    pub aliases: Vec<String>,
}

fn validate_id(id: &str) -> AppResult<()> {
    if id.is_empty() {
        return Err(AppError::BadRequest("analyte id cannot be empty".into()));
    }
    if id.len() > 80 {
        return Err(AppError::BadRequest(
            "analyte id is too long (max 80 chars)".into(),
        ));
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(AppError::BadRequest(
            "analyte id may only contain a-z, 0-9, and underscores".into(),
        ));
    }
    Ok(())
}

fn write_aliases(tx: &rusqlite::Connection, analyte_id: &str, aliases: &[String]) -> AppResult<()> {
    let mut stmt = tx.prepare(
        "INSERT OR IGNORE INTO analyte_aliases(alias, analyte_id, source) VALUES(?1, ?2, 'user')",
    )?;
    // pt_name itself is registered as an alias by ingest convention; we
    // don't auto-add it here because the caller's `pt_name` is on the
    // analyte row anyway and the resolver checks both maps.
    for alias in aliases {
        let trimmed = alias.trim();
        if trimmed.is_empty() {
            continue;
        }
        stmt.execute(rusqlite::params![trimmed, analyte_id])?;
    }
    Ok(())
}

#[tauri::command]
pub async fn create_analyte(state: State<'_, AppState>, args: AnalyteWriteArgs) -> AppResult<()> {
    validate_id(&args.id)?;
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let tx = db.conn.unchecked_transaction()?;
    let exists: bool = tx
        .query_row::<i64, _, _>("SELECT 1 FROM analytes WHERE id = ?1", [&args.id], |r| {
            r.get(0)
        })
        .optional()?
        .is_some();
    if exists {
        return Err(AppError::BadRequest(format!(
            "analyte id `{}` already exists",
            args.id
        )));
    }
    let units_json = serde_json::to_string(&args.expected_units)?;
    tx.execute(
        "INSERT INTO analytes(
            id, pt_name, loinc, section, subsection, panel,
            is_qualitative, is_derived, paired_value, is_panel_header,
            expected_units_json, default_ref_json,
            sex_dependent, age_dependent, cycle_dependent, method_annotation,
            description, high_means, low_means, unit_notes,
            categorical_tiers_json, cycle_phases_json, source
         ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,'user')",
        rusqlite::params![
            args.id, args.pt_name, args.loinc, args.section, args.subsection, args.panel,
            args.is_qualitative as i32, args.is_derived as i32,
            args.paired_value as i32, args.is_panel_header as i32,
            units_json,
            args.default_ref_json,
            args.sex_dependent as i32, args.age_dependent as i32, args.cycle_dependent as i32,
            args.method_annotation,
            args.description, args.high_means, args.low_means, args.unit_notes,
            args.categorical_tiers_json, args.cycle_phases_json,
        ],
    )?;
    // Always register pt_name as an alias under user source so the parser
    // can resolve it on next ingest.
    tx.execute(
        "INSERT OR IGNORE INTO analyte_aliases(alias, analyte_id, source) VALUES(?1, ?2, 'user')",
        rusqlite::params![args.pt_name, args.id],
    )?;
    write_aliases(&tx, &args.id, &args.aliases)?;
    tx.commit()?;
    audit::log(
        &db.conn,
        "create",
        "analyte",
        Some(&args.id),
        &format!("Created analyte {} ({})", args.pt_name, args.id),
        Some(&json!({
            "pt_name": args.pt_name,
            "section": args.section,
            "subsection": args.subsection,
            "alias_count": args.aliases.len(),
        })),
    );
    Ok(())
}

#[tauri::command]
pub async fn update_analyte(state: State<'_, AppState>, args: AnalyteWriteArgs) -> AppResult<()> {
    validate_id(&args.id)?;
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    let tx = db.conn.unchecked_transaction()?;
    let units_json = serde_json::to_string(&args.expected_units)?;
    let affected = tx.execute(
        "UPDATE analytes SET
            pt_name = ?2, loinc = ?3, section = ?4, subsection = ?5, panel = ?6,
            is_qualitative = ?7, is_derived = ?8, paired_value = ?9, is_panel_header = ?10,
            expected_units_json = ?11, default_ref_json = ?12,
            sex_dependent = ?13, age_dependent = ?14, cycle_dependent = ?15,
            method_annotation = ?16,
            description = ?17, high_means = ?18, low_means = ?19, unit_notes = ?20,
            categorical_tiers_json = ?21, cycle_phases_json = ?22
         WHERE id = ?1",
        rusqlite::params![
            args.id,
            args.pt_name,
            args.loinc,
            args.section,
            args.subsection,
            args.panel,
            args.is_qualitative as i32,
            args.is_derived as i32,
            args.paired_value as i32,
            args.is_panel_header as i32,
            units_json,
            args.default_ref_json,
            args.sex_dependent as i32,
            args.age_dependent as i32,
            args.cycle_dependent as i32,
            args.method_annotation,
            args.description,
            args.high_means,
            args.low_means,
            args.unit_notes,
            args.categorical_tiers_json,
            args.cycle_phases_json,
        ],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("analyte {}", args.id)));
    }
    write_aliases(&tx, &args.id, &args.aliases)?;
    tx.commit()?;
    audit::log(
        &db.conn,
        "update",
        "analyte",
        Some(&args.id),
        &format!("Updated analyte {} ({})", args.pt_name, args.id),
        Some(&json!({
            "pt_name": args.pt_name,
            "section": args.section,
            "subsection": args.subsection,
        })),
    );
    Ok(())
}

#[tauri::command]
pub async fn delete_analyte(state: State<'_, AppState>, analyte_id: String) -> AppResult<()> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    // Guard: only delete user-source rows. Seed rows would just come back
    // on next reload anyway, and deleting them while results reference
    // them would orphan FK pointers.
    let source: Option<String> = db
        .conn
        .query_row::<String, _, _>(
            "SELECT source FROM analytes WHERE id = ?1",
            [&analyte_id],
            |r| r.get(0),
        )
        .optional()?;
    let Some(source) = source else {
        return Err(AppError::NotFound(format!("analyte {analyte_id}")));
    };
    if source != "user" {
        return Err(AppError::BadRequest(
            "Only user-created analytes can be deleted. Seed rows are owned by the bundled JSON."
                .into(),
        ));
    }
    let result_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM results WHERE analyte_id = ?1",
        [&analyte_id],
        |r| r.get(0),
    )?;
    if result_count > 0 {
        return Err(AppError::BadRequest(format!(
            "Can't delete `{analyte_id}` — {result_count} result row(s) still reference it. Re-link them first."
        )));
    }
    db.conn
        .execute("DELETE FROM analytes WHERE id = ?1", [&analyte_id])?;
    audit::log(
        &db.conn,
        "delete",
        "analyte",
        Some(&analyte_id),
        &format!("Deleted analyte {analyte_id}"),
        None,
    );
    Ok(())
}

#[derive(Deserialize)]
pub struct AliasArgs {
    pub analyte_id: String,
    pub alias: String,
}

#[tauri::command]
pub async fn add_analyte_alias(state: State<'_, AppState>, args: AliasArgs) -> AppResult<()> {
    let alias = args.alias.trim().to_string();
    if alias.is_empty() {
        return Err(AppError::BadRequest("alias cannot be empty".into()));
    }
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    db.conn.execute(
        "INSERT OR IGNORE INTO analyte_aliases(alias, analyte_id, source) VALUES(?1, ?2, 'user')",
        rusqlite::params![alias, args.analyte_id],
    )?;
    audit::log(
        &db.conn,
        "create",
        "alias",
        Some(&args.analyte_id),
        &format!("Added alias \"{alias}\" → {}", args.analyte_id),
        Some(&json!({ "alias": alias, "analyte_id": args.analyte_id })),
    );
    Ok(())
}

#[tauri::command]
pub async fn remove_analyte_alias(state: State<'_, AppState>, alias: String) -> AppResult<()> {
    let guard = state.db.lock().await;
    let db = guard.as_ref().ok_or(AppError::Locked)?;
    // Only user-source aliases get removed. Seed-source aliases are
    // re-installed by reload_ontology anyway, so removing them is futile.
    let removed = db.conn.execute(
        "DELETE FROM analyte_aliases WHERE alias = ?1 AND source = 'user'",
        [&alias],
    )?;
    if removed > 0 {
        audit::log(
            &db.conn,
            "delete",
            "alias",
            None,
            &format!("Removed alias \"{alias}\""),
            Some(&json!({ "alias": alias })),
        );
    }
    Ok(())
}
