use once_cell::sync::Lazy;
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

use crate::error::AppResult;

const MIG_001: &str = include_str!("../../migrations/001_initial_schema.sql");
const MIG_002: &str = include_str!("../../migrations/002_analyte_descriptions.sql");
const MIG_003: &str = include_str!("../../migrations/003_categorical_tiers.sql");
const MIG_004: &str = include_str!("../../migrations/004_cycle_phase.sql");
const MIG_005: &str = include_str!("../../migrations/005_report_nickname.sql");
const MIG_006: &str = include_str!("../../migrations/006_patient_nickname_notes.sql");
const MIG_007: &str = include_str!("../../migrations/007_annotations_and_hrt.sql");
const MIG_008: &str = include_str!("../../migrations/008_analyte_source.sql");
const MIG_009: &str = include_str!("../../migrations/009_audit_log.sql");

static MIGRATIONS: Lazy<Migrations<'static>> = Lazy::new(|| {
    Migrations::new(vec![
        M::up(MIG_001),
        M::up(MIG_002),
        M::up(MIG_003),
        M::up(MIG_004),
        M::up(MIG_005),
        M::up(MIG_006),
        M::up(MIG_007),
        M::up(MIG_008),
        M::up(MIG_009),
    ])
});

pub fn run(conn: &mut Connection) -> AppResult<()> {
    MIGRATIONS.to_latest(conn)?;
    Ok(())
}
