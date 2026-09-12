use std::collections::HashMap;
use std::sync::Mutex;

use tauri::State;

use crate::db;
use crate::error::Result;
use crate::export;
use crate::kinds::registry;
use crate::models::*;
use crate::reports;

pub struct DbState {
    pub conn: Mutex<rusqlite::Connection>,
}

fn lock(state: &DbState) -> Result<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    Ok(state.conn.lock()?)
}

fn kinds_dto() -> Vec<KindDto> {
    registry::implemented()
        .iter()
        .map(KindDto::from_desc)
        .collect()
}

#[tauri::command]
pub fn list_kinds() -> Vec<KindDto> {
    kinds_dto()
}

#[tauri::command]
pub fn get_bootstrap(state: State<DbState>) -> Result<BootstrapDto> {
    let conn = lock(&state)?;
    let settings = db::get_settings(&conn)?;
    let resolved_language = db::resolved_language(&settings);
    Ok(BootstrapDto {
        settings,
        accounts: db::list_accounts(&conn)?,
        categories: db::list_categories(&conn)?,
        tags: db::list_tags(&conn)?,
        kinds: kinds_dto(),
        db_path: db::default_db_path().display().to_string(),
        resolved_language,
        system_language: db::system_language(),
        category_palette: db::palette::all(),
    })
}

#[tauri::command]
pub fn get_settings(state: State<DbState>) -> Result<SettingsDto> {
    let conn = lock(&state)?;
    db::get_settings(&conn)
}

#[tauri::command]
pub fn update_settings(state: State<DbState>, settings: SettingsDto) -> Result<SettingsDto> {
    let conn = lock(&state)?;
    db::update_settings(&conn, settings)
}

#[tauri::command]
pub fn list_accounts(state: State<DbState>) -> Result<Vec<AccountDto>> {
    let conn = lock(&state)?;
    db::list_accounts(&conn)
}

#[tauri::command]
pub fn create_account(state: State<DbState>, write: AccountWrite) -> Result<AccountDto> {
    let conn = lock(&state)?;
    db::create_account(&conn, write)
}

#[tauri::command]
pub fn update_account(state: State<DbState>, id: String, write: AccountWrite) -> Result<AccountDto> {
    let conn = lock(&state)?;
    db::update_account(&conn, &id, write)
}

#[tauri::command]
pub fn delete_account(state: State<DbState>, id: String) -> Result<()> {
    let conn = lock(&state)?;
    db::delete_account(&conn, &id)
}

#[tauri::command]
pub fn reorder_accounts(state: State<DbState>, ordered_ids: Vec<String>) -> Result<()> {
    let conn = lock(&state)?;
    db::reorder_accounts(&conn, &ordered_ids)
}

#[tauri::command]
pub fn account_usage(state: State<DbState>, id: String) -> Result<i64> {
    let conn = lock(&state)?;
    db::account_usage(&conn, &id)
}

#[tauri::command]
pub fn list_categories(state: State<DbState>) -> Result<Vec<CategoryDto>> {
    let conn = lock(&state)?;
    db::list_categories(&conn)
}

#[tauri::command]
pub fn create_main_category(
    state: State<DbState>,
    name: String,
    other_label: String,
) -> Result<Vec<CategoryDto>> {
    let conn = lock(&state)?;
    db::create_main_category(&conn, &name, &other_label)
}

#[tauri::command]
pub fn create_sub_category(
    state: State<DbState>,
    parent_id: String,
    name: String,
) -> Result<Vec<CategoryDto>> {
    let conn = lock(&state)?;
    db::create_sub_category(&conn, &parent_id, &name)
}

#[tauri::command]
pub fn rename_category(state: State<DbState>, id: String, name: String) -> Result<Vec<CategoryDto>> {
    let conn = lock(&state)?;
    db::rename_category(&conn, &id, &name)
}

#[tauri::command]
pub fn update_category_color(
    state: State<DbState>,
    id: String,
    color_hex: String,
) -> Result<Vec<CategoryDto>> {
    let conn = lock(&state)?;
    db::update_category_color(&conn, &id, &color_hex)
}

#[tauri::command]
pub fn delete_category(state: State<DbState>, id: String) -> Result<()> {
    let conn = lock(&state)?;
    db::delete_category(&conn, &id)
}

#[tauri::command]
pub fn reorder_categories(
    state: State<DbState>,
    parent_id: Option<String>,
    ordered_ids: Vec<String>,
) -> Result<()> {
    let conn = lock(&state)?;
    db::reorder_categories(&conn, parent_id.as_deref(), &ordered_ids)
}

#[tauri::command]
pub fn category_usage(state: State<DbState>, id: String) -> Result<i64> {
    let conn = lock(&state)?;
    db::category_usage(&conn, &id)
}

#[tauri::command]
pub fn list_tags(state: State<DbState>) -> Result<Vec<TagDto>> {
    let conn = lock(&state)?;
    db::list_tags(&conn)
}

#[tauri::command]
pub fn delete_tag(state: State<DbState>, id: String) -> Result<()> {
    let conn = lock(&state)?;
    db::delete_tag(&conn, &id)
}

#[tauri::command]
pub fn create_entry(state: State<DbState>, write: EntryWrite) -> Result<EntryDto> {
    let mut conn = lock(&state)?;
    db::create_entry(&mut conn, write)
}

#[tauri::command]
pub fn update_entry(state: State<DbState>, id: String, write: EntryWrite) -> Result<EntryDto> {
    let mut conn = lock(&state)?;
    db::update_entry(&mut conn, &id, write)
}

#[tauri::command]
pub fn delete_entry(state: State<DbState>, id: String) -> Result<()> {
    let conn = lock(&state)?;
    db::delete_entry(&conn, &id)
}

#[tauri::command]
pub fn get_entry(state: State<DbState>, id: String) -> Result<EntryDto> {
    let conn = lock(&state)?;
    db::get_entry(&conn, &id)
}

#[tauri::command]
pub fn list_entries(state: State<DbState>, filter: LedgerFilter) -> Result<Vec<EntryDto>> {
    let conn = lock(&state)?;
    db::list_entries(&conn, &filter)
}

#[tauri::command]
pub fn get_report(state: State<DbState>, query: ReportQuery) -> Result<ReportDto> {
    let conn = lock(&state)?;
    let entries = db::list_all_entry_rows(&conn)?;
    let categories = db::list_categories(&conn)?;
    reports::build_report(&entries, &categories, &query)
}

#[tauri::command]
pub fn export_entries_csv(
    state: State<DbState>,
    path: String,
    from_date: Option<String>,
    to_date: Option<String>,
    labels: HashMap<String, String>,
) -> Result<()> {
    let conn = lock(&state)?;
    export::write_entries_csv(&conn, &path, from_date, to_date, &labels)
}

#[tauri::command]
pub fn export_entries_txt(
    state: State<DbState>,
    path: String,
    from_date: Option<String>,
    to_date: Option<String>,
    labels: HashMap<String, String>,
) -> Result<()> {
    let conn = lock(&state)?;
    export::write_entries_txt(&conn, &path, from_date, to_date, &labels)
}

#[tauri::command]
pub fn export_backup_json(state: State<DbState>, path: String) -> Result<()> {
    let conn = lock(&state)?;
    export::write_backup_json(&conn, &path)
}
