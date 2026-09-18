mod commands;
mod crypto;
mod db;
mod error;
mod llm;
mod ocr;
mod ocr_vision;
mod ontology;
mod parse;
mod pdf;
mod state;

use tauri::Manager;

const LEGACY_APP_IDENTIFIER: &str = "com.blevel.tracker";

/// Relaunch the native process so the webview and all native services start
/// from a clean state. `AppHandle::restart` performs the platform-specific
/// executable handoff (including installed macOS app bundles).
#[tauri::command]
fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

/// Keep existing vaults usable after the product identifier changed from the
/// original internal name to bloody-level. Only migrate when the new location
/// does not exist, so two populated vaults are never merged or overwritten.
fn migrate_legacy_data_dir(data_dir: &std::path::Path) -> std::io::Result<()> {
    if data_dir.exists() {
        return Ok(());
    }
    let Some(parent) = data_dir.parent() else {
        return Ok(());
    };
    let legacy_dir = parent.join(LEGACY_APP_IDENTIFIER);
    if legacy_dir.is_dir() {
        std::fs::rename(legacy_dir, data_dir)?;
    }
    Ok(())
}

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("resolve app data dir");
            migrate_legacy_data_dir(&data_dir)?;
            std::fs::create_dir_all(&data_dir)?;
            std::fs::create_dir_all(data_dir.join("pdfs"))?;
            std::fs::create_dir_all(data_dir.join("models"))?;

            let resource_dir = app.path().resource_dir().ok();
            // Tell the pdf module where bundled resources live so it can find
            // pdfium.dll in installed builds (build.rs places it at
            // src-tauri/binaries/, and tauri.conf.json bundles binaries/* as
            // resources, landing them at <resource_dir>/binaries/).
            if let Some(rd) = resource_dir.clone() {
                pdf::set_resource_dir(rd);
            }

            app.manage(state::AppState::new(data_dir, resource_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::auth_status,
            commands::auth::auth_setup_password,
            commands::auth::auth_unlock_password,
            commands::auth::auth_change_password,
            commands::auth::auth_lock,
            commands::auth::auth_reset_instance,
            commands::auth::auth_register_passkey,
            commands::auth::auth_unlock_passkey,
            restart_app,
            commands::settings::settings_get,
            commands::settings::settings_set,
            commands::settings::settings_get_all,
            commands::settings::settings_reset_all,
            commands::ingest::ingest_pdf,
            commands::ingest::ingest_pdfs,
            commands::reports::list_patients,
            commands::reports::list_reports,
            commands::reports::analyte_timeseries,
            commands::reports::patient_analyte_summaries,
            commands::reports::list_flagged_analytes,
            commands::tiers::tier_status_tesseract,
            commands::tiers::tier_status_llm,
            commands::tiers::tier_load_llm,
            commands::tiers::tier_unload_llm,
            commands::tiers::tier_status_olmocr,
            commands::tiers::tier_load_olmocr,
            commands::tiers::tier_unload_olmocr,
            commands::tiers::tier_status_pdfium,
            commands::app_info::app_info,
            commands::app_info::export_vault,
            commands::app_info::import_vault,
            commands::report_detail::report_detail,
            commands::samples::sample_pdf_paths,
            commands::records_admin::delete_report,
            commands::records_admin::bulk_delete_reports,
            commands::records_admin::delete_patient,
            commands::records_admin::delete_result,
            commands::records_admin::update_patient,
            commands::records_admin::update_report,
            commands::records_admin::update_result,
            commands::records_admin::link_unmatched_analyte,
            commands::records_admin::list_analytes,
            commands::records_admin::merge_patients,
            commands::records_admin::reload_ontology,
            commands::records_admin::set_report_cycle_phase,
            commands::records_admin::set_report_nickname,
            commands::records_admin::backfill_patient_sex,
            commands::records_admin::backfill_patient_dob,
            commands::records_admin::create_patient,
            commands::records_admin::patient_overviews,
            commands::records_admin::set_patient_nickname,
            commands::records_admin::set_patient_notes,
            commands::records_admin::set_patient_hrt_start,
            commands::records_admin::set_report_annotations,
            commands::reparse::reparse_report,
            commands::reparse::reparse_all_reports,
            commands::analyte_info::analyte_info,
            commands::analyte_info::list_ontology_entries,
            commands::analyte_info::create_analyte,
            commands::analyte_info::update_analyte,
            commands::analyte_info::delete_analyte,
            commands::analyte_info::add_analyte_alias,
            commands::analyte_info::remove_analyte_alias,
            commands::shell::open_file_external,
            commands::shell::open_url,
            commands::search::global_search,
            commands::export::export_analyte_timeseries_csv,
            commands::export::export_report_rows_csv,
            commands::audit::list_audit_entries,
            commands::audit::clear_audit_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
