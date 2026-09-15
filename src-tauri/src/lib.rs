mod balance;
mod commands;
mod db;
mod error;
mod export;
mod import;
mod kinds;
mod models;
mod money;
mod reports;
mod time_util;

use tauri::Manager;
use tauri_plugin_window_state::{Builder as WindowStateBuilder, StateFlags};

pub fn run() {
    tauri::Builder::default()
        .plugin(
            WindowStateBuilder::new()
                .with_state_flags(StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let conn = db::open_ledger().map_err(|e| e.to_string())?;
            app.manage(commands::DbState {
                conn: std::sync::Mutex::new(conn),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_kinds,
            commands::get_bootstrap,
            commands::get_settings,
            commands::update_settings,
            commands::list_accounts,
            commands::create_account,
            commands::update_account,
            commands::delete_account,
            commands::reorder_accounts,
            commands::account_usage,
            commands::list_categories,
            commands::create_main_category,
            commands::create_sub_category,
            commands::rename_category,
            commands::update_category_color,
            commands::delete_category,
            commands::reorder_categories,
            commands::category_usage,
            commands::list_tags,
            commands::delete_tag,
            commands::create_entry,
            commands::update_entry,
            commands::delete_entry,
            commands::get_entry,
            commands::list_entries,
            commands::get_report,
            commands::export_entries_csv,
            commands::export_entries_txt,
            commands::export_backup_json,
            commands::list_pending_entries,
            commands::update_pending_entry,
            commands::delete_pending_entry,
            commands::post_pending_entry,
            commands::import_pending_csv,
            commands::write_pending_csv_template,
            commands::list_trash,
            commands::restore_trash,
            commands::purge_trash,
            commands::empty_trash
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
