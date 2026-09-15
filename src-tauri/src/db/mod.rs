mod migrate;
mod pending;
pub mod palette;
mod schema;
mod seed;
mod trash;

use std::path::{Path, PathBuf};

use rusqlite::{params, params_from_iter, Connection, OptionalExtension};
use uuid::Uuid;

use crate::balance;
use crate::error::{AppError, Result};
use crate::kinds::{self, registry};
use crate::models::*;
use crate::time_util;

pub use pending::{
    delete_pending_entry, insert_pending_row, list_pending_entries, list_pending_including_deleted,
    pending_count, post_pending_entry, update_pending_entry,
};
pub use trash::{empty_trash, list_trash, purge_trash, restore_trash, trash_count};

pub fn default_db_path() -> PathBuf {
    let local = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    local.join("YuliLedger").join("ledger.sqlite")
}

pub fn open_ledger() -> Result<Connection> {
    let path = default_db_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    open_at(&path)
}

pub fn open_at(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", true)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.execute_batch(schema::SCHEMA)?;
    seed::seed_if_empty(&conn)?;
    migrate::apply(&conn)?;
    Ok(conn)
}

pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

fn account_kind_ok(kind: &str) -> bool {
    matches!(kind, "cash" | "bank" | "ewallet" | "credit" | "other")
}

pub fn list_accounts(conn: &Connection) -> Result<Vec<AccountDto>> {
    let entries = list_all_entry_rows(conn)?;
    let mut stmt = conn.prepare(
        "SELECT id, name, account_kind, opening_balance_minor, opening_debt_minor, opening_at, note, sort_order, preset_key
         FROM account WHERE deleted_at IS NULL ORDER BY sort_order ASC, id ASC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, i64>(3)?,
            r.get::<_, i64>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, Option<String>>(6)?,
            r.get::<_, i32>(7)?,
            r.get::<_, Option<String>>(8)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (id, name, account_kind, opening, opening_debt, opening_at, note, sort_order, preset_key) = row?;
        let balance_minor = balance::balance_for_account(opening, &opening_at, &id, &entries);
        let debt_minor = balance::debt_for_account(opening_debt, &opening_at, &id, &entries);
        out.push(AccountDto {
            id,
            name,
            account_kind,
            opening_balance_minor: opening,
            opening_debt_minor: opening_debt,
            opening_at,
            note,
            sort_order,
            preset_key,
            balance_minor,
            debt_minor,
            deleted_at: None,
        });
    }
    Ok(out)
}

fn account_write_ok(write: &AccountWrite) -> Result<()> {
    if write.name.trim().is_empty() {
        return Err(AppError::new("error.accountNameRequired"));
    }
    if !account_kind_ok(&write.account_kind) {
        return Err(AppError::new("error.accountKindInvalid"));
    }
    if write.opening_debt_minor < 0 {
        return Err(AppError::new("error.amountInvalid"));
    }
    let _ = time_util::parse_utc_minute(&write.opening_at)?;
    Ok(())
}

pub fn create_account(conn: &Connection, write: AccountWrite) -> Result<AccountDto> {
    account_write_ok(&write)?;
    let max: i32 = conn
        .query_row("SELECT COALESCE(MAX(sort_order), -1) FROM account", [], |r| r.get(0))?;
    let id = new_id();
    conn.execute(
        "INSERT INTO account (id, name, account_kind, opening_balance_minor, opening_debt_minor, opening_at, note, sort_order, preset_key)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL)",
        params![
            id,
            write.name.trim(),
            write.account_kind,
            write.opening_balance_minor,
            write.opening_debt_minor,
            time_util::format_utc_minute(time_util::parse_utc_minute(&write.opening_at)?),
            write.note,
            max + 1
        ],
    )?;
    list_accounts(conn)?
        .into_iter()
        .find(|a| a.id == id)
        .ok_or_else(|| AppError::new("error.db"))
}

pub fn update_account(conn: &Connection, id: &str, write: AccountWrite) -> Result<AccountDto> {
    account_write_ok(&write)?;
    let opening_at = time_util::format_utc_minute(time_util::parse_utc_minute(&write.opening_at)?);
    let n = conn.execute(
        "UPDATE account SET name = ?1, account_kind = ?2, opening_balance_minor = ?3, opening_debt_minor = ?4, opening_at = ?5, note = ?6
         WHERE id = ?7 AND deleted_at IS NULL",
        params![
            write.name.trim(),
            write.account_kind,
            write.opening_balance_minor,
            write.opening_debt_minor,
            opening_at,
            write.note,
            id
        ],
    )?;
    if n == 0 {
        return Err(AppError::new("error.accountNotFound"));
    }
    list_accounts(conn)?
        .into_iter()
        .find(|a| a.id == id)
        .ok_or_else(|| AppError::new("error.db"))
}

pub fn account_usage(conn: &Connection, id: &str) -> Result<i64> {
    let posted: i64 = conn.query_row(
        "SELECT COUNT(*) FROM entry WHERE account_id = ?1 OR counter_account_id = ?1",
        [id],
        |r| r.get(0),
    )?;
    let pending: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pending_entry WHERE account_id = ?1 OR counter_account_id = ?1",
        [id],
        |r| r.get(0),
    )?;
    Ok(posted + pending)
}

pub fn delete_account(conn: &Connection, id: &str) -> Result<()> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM account WHERE deleted_at IS NULL",
        [],
        |r| r.get(0),
    )?;
    if count <= 1 {
        return Err(AppError::new("error.lastAccount"));
    }
    let default_id: String =
        conn.query_row("SELECT default_account_id FROM ledger_settings WHERE id = 1", [], |r| {
            r.get(0)
        })?;
    if default_id == id {
        return Err(AppError::new("error.defaultAccount"));
    }
    let used = account_usage(conn, id)?;
    if used > 0 {
        return Err(AppError::with_count("error.accountInUse", used));
    }
    trash::mark_deleted(conn, "account", id, "error.accountNotFound")
}

pub fn reorder_accounts(conn: &Connection, ordered_ids: &[String]) -> Result<()> {
    for (i, id) in ordered_ids.iter().enumerate() {
        conn.execute(
            "UPDATE account SET sort_order = ?1 WHERE id = ?2",
            params![i as i32, id],
        )?;
    }
    Ok(())
}

pub fn list_categories(conn: &Connection) -> Result<Vec<CategoryDto>> {
    let mut stmt = conn.prepare(
        "SELECT id, parent_id, name, preset_key, sort_order, color_hex FROM category
         WHERE deleted_at IS NULL
         ORDER BY parent_id IS NOT NULL, sort_order ASC, id ASC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(CategoryDto {
            id: r.get(0)?,
            parent_id: r.get(1)?,
            name: r.get(2)?,
            preset_key: r.get(3)?,
            sort_order: r.get(4)?,
            color_hex: r.get(5)?,
            deleted_at: None,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn next_main_color(conn: &Connection) -> Result<String> {
    let mut stmt = conn.prepare(
        "SELECT color_hex FROM category WHERE parent_id IS NULL AND deleted_at IS NULL",
    )?;
    let used: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(palette::next_unused(&used))
}

pub fn create_main_category(conn: &Connection, name: &str, other_label: &str) -> Result<Vec<CategoryDto>> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::new("error.categoryNameRequired"));
    }
    let other = other_label.trim();
    if other.is_empty() {
        return Err(AppError::new("error.categoryNameRequired"));
    }
    let color = next_main_color(conn)?;
    let max: i32 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) FROM category WHERE parent_id IS NULL",
        [],
        |r| r.get(0),
    )?;
    let main_id = new_id();
    let sub_id = new_id();
    conn.execute(
        "INSERT INTO category (id, parent_id, name, preset_key, sort_order, color_hex)
         VALUES (?1, NULL, ?2, NULL, ?3, ?4)",
        params![main_id, name, max + 1, color],
    )?;
    conn.execute(
        "INSERT INTO category (id, parent_id, name, preset_key, sort_order, color_hex)
         VALUES (?1, ?2, ?3, NULL, 0, NULL)",
        params![sub_id, main_id, other],
    )?;
    list_categories(conn)
}

pub fn create_sub_category(conn: &Connection, parent_id: &str, name: &str) -> Result<Vec<CategoryDto>> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::new("error.categoryNameRequired"));
    }
    let parent_is_root: Option<Option<String>> = conn
        .query_row(
            "SELECT parent_id FROM category WHERE id = ?1 AND deleted_at IS NULL",
            [parent_id],
            |r| r.get(0),
        )
        .optional()?;
    match parent_is_root {
        None => return Err(AppError::new("error.categoryNotFound")),
        Some(Some(_)) => return Err(AppError::new("error.categoryDepth")),
        Some(None) => {}
    }
    let max: i32 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) FROM category WHERE parent_id = ?1",
        [parent_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO category (id, parent_id, name, preset_key, sort_order, color_hex)
         VALUES (?1, ?2, ?3, NULL, ?4, NULL)",
        params![new_id(), parent_id, name, max + 1],
    )?;
    list_categories(conn)
}

pub fn update_category_color(conn: &Connection, id: &str, color_hex: &str) -> Result<Vec<CategoryDto>> {
    let color = color_hex.trim();
    if !palette::is_palette_color(color) {
        return Err(AppError::new("error.colorInvalid"));
    }
    let parent: Option<Option<String>> = conn
        .query_row(
            "SELECT parent_id FROM category WHERE id = ?1 AND deleted_at IS NULL",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    match parent {
        None => return Err(AppError::new("error.categoryNotFound")),
        Some(Some(_)) => return Err(AppError::new("error.notMainCategory")),
        Some(None) => {}
    }
    conn.execute(
        "UPDATE category SET color_hex = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![color, id],
    )?;
    list_categories(conn)
}

pub fn rename_category(conn: &Connection, id: &str, name: &str) -> Result<Vec<CategoryDto>> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::new("error.categoryNameRequired"));
    }
    let n = conn.execute(
        "UPDATE category SET name = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![name, id],
    )?;
    if n == 0 {
        return Err(AppError::new("error.categoryNotFound"));
    }
    list_categories(conn)
}

pub fn category_usage(conn: &Connection, id: &str) -> Result<i64> {
    let posted: i64 = conn.query_row(
        "SELECT COUNT(*) FROM entry WHERE category_id = ?1 OR fee_category_id = ?1",
        [id],
        |r| r.get(0),
    )?;
    let pending: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pending_entry WHERE category_id = ?1 OR fee_category_id = ?1",
        [id],
        |r| r.get(0),
    )?;
    Ok(posted + pending)
}

fn is_default_fee(conn: &Connection, id: &str) -> Result<bool> {
    let fee: Option<String> = conn.query_row(
        "SELECT default_fee_category_id FROM ledger_settings WHERE id = 1",
        [],
        |r| r.get(0),
    )?;
    Ok(fee.as_deref() == Some(id))
}

pub fn delete_category(conn: &Connection, id: &str) -> Result<()> {
    let parent_id: Option<String> = conn
        .query_row(
            "SELECT parent_id FROM category WHERE id = ?1 AND deleted_at IS NULL",
            [id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::new("error.categoryNotFound"))?;

    let now = time_util::now_utc_minute();
    if parent_id.is_none() {
        let mut child_ids = Vec::new();
        let mut stmt = conn.prepare(
            "SELECT id FROM category WHERE parent_id = ?1 AND deleted_at IS NULL",
        )?;
        let rows = stmt.query_map([id], |r| r.get::<_, String>(0))?;
        for row in rows {
            child_ids.push(row?);
        }
        for child in &child_ids {
            if is_default_fee(conn, child)? {
                return Err(AppError::new("error.defaultFeeCategory"));
            }
            let used = category_usage(conn, child)?;
            if used > 0 {
                return Err(AppError::with_count("error.categoryInUse", used));
            }
        }
        for child in &child_ids {
            conn.execute(
                "UPDATE category SET deleted_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
                params![now, child],
            )?;
        }
        conn.execute(
            "UPDATE category SET deleted_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
            params![now, id],
        )?;
        Ok(())
    } else {
        if is_default_fee(conn, id)? {
            return Err(AppError::new("error.defaultFeeCategory"));
        }
        let used = category_usage(conn, id)?;
        if used > 0 {
            return Err(AppError::with_count("error.categoryInUse", used));
        }
        trash::mark_deleted(conn, "category", id, "error.categoryNotFound")
    }
}

pub fn reorder_categories(conn: &Connection, parent_id: Option<&str>, ordered_ids: &[String]) -> Result<()> {
    for (i, id) in ordered_ids.iter().enumerate() {
        match parent_id {
            None => {
                conn.execute(
                    "UPDATE category SET sort_order = ?1 WHERE id = ?2 AND parent_id IS NULL",
                    params![i as i32, id],
                )?;
            }
            Some(pid) => {
                conn.execute(
                    "UPDATE category SET sort_order = ?1 WHERE id = ?2 AND parent_id = ?3",
                    params![i as i32, id, pid],
                )?;
            }
        }
    }
    Ok(())
}

pub fn list_tags(conn: &Connection) -> Result<Vec<TagDto>> {
    let mut stmt = conn.prepare("SELECT id, name FROM tag ORDER BY lower(name) ASC")?;
    let rows = stmt.query_map([], |r| {
        Ok(TagDto {
            id: r.get(0)?,
            name: r.get(1)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn delete_tag(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM entry_tag WHERE tag_id = ?1", [id])?;
    conn.execute("DELETE FROM pending_entry_tag WHERE tag_id = ?1", [id])?;
    let n = conn.execute("DELETE FROM tag WHERE id = ?1", [id])?;
    if n == 0 {
        return Err(AppError::new("error.tagNotFound"));
    }
    Ok(())
}

pub(super) fn upsert_tags(conn: &Connection, entry_id: &str, names: &[String]) -> Result<Vec<String>> {
    conn.execute("DELETE FROM entry_tag WHERE entry_id = ?1", [entry_id])?;
    let mut ids = Vec::new();
    for raw in names {
        let name = raw.trim();
        if name.is_empty() {
            return Err(AppError::new("error.tagEmpty"));
        }
        let existing: Option<String> = conn
            .query_row(
                "SELECT id FROM tag WHERE lower(name) = lower(?1)",
                [name],
                |r| r.get(0),
            )
            .optional()?;
        let tag_id = if let Some(id) = existing {
            id
        } else {
            let id = new_id();
            conn.execute("INSERT INTO tag (id, name) VALUES (?1, ?2)", params![id, name])?;
            id
        };
        conn.execute(
            "INSERT OR IGNORE INTO entry_tag (entry_id, tag_id) VALUES (?1, ?2)",
            params![entry_id, tag_id],
        )?;
        if !ids.contains(&tag_id) {
            ids.push(tag_id);
        }
    }
    Ok(ids)
}

pub(super) fn require_subcategory(conn: &Connection, id: &str) -> Result<()> {
    let parent: Option<String> = conn
        .query_row(
            "SELECT parent_id FROM category WHERE id = ?1 AND deleted_at IS NULL",
            [id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::new("error.categoryNotFound"))?;
    if parent.is_none() {
        return Err(AppError::new("error.notSubcategory"));
    }
    Ok(())
}

pub(super) fn require_account(conn: &Connection, id: &str) -> Result<()> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM account WHERE id = ?1 AND deleted_at IS NULL",
        [id],
        |r| r.get(0),
    )?;
    if n == 0 {
        return Err(AppError::new("error.accountNotFound"));
    }
    Ok(())
}

fn tags_for(conn: &Connection, entry_id: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT tag_id FROM entry_tag WHERE entry_id = ?1")?;
    let rows = stmt.query_map([entry_id], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn map_entry_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<EntryRow> {
    Ok(EntryRow {
        id: r.get(0)?,
        kind_id: r.get(1)?,
        amount_minor: r.get(2)?,
        occurred_at: r.get(3)?,
        account_id: r.get(4)?,
        counter_account_id: r.get(5)?,
        counter_amount_minor: r.get(6)?,
        category_id: r.get(7)?,
        fee_category_id: r.get(8)?,
        note: r.get(9)?,
        kind_payload: r.get(10)?,
        created_at: r.get(11)?,
        updated_at: r.get(12)?,
    })
}

pub fn list_all_entry_rows(conn: &Connection) -> Result<Vec<EntryRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, kind_id, amount_minor, occurred_at, account_id, counter_account_id,
                counter_amount_minor, category_id, fee_category_id, note, kind_payload,
                created_at, updated_at FROM entry WHERE deleted_at IS NULL",
    )?;
    let rows = stmt.query_map([], map_entry_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn to_dto(conn: &Connection, row: EntryRow) -> Result<EntryDto> {
    let tag_ids = tags_for(conn, &row.id)?;
    Ok(EntryDto {
        id: row.id,
        kind_id: row.kind_id,
        amount_minor: row.amount_minor,
        occurred_at: row.occurred_at,
        account_id: row.account_id,
        counter_account_id: row.counter_account_id,
        counter_amount_minor: row.counter_amount_minor,
        category_id: row.category_id,
        fee_category_id: row.fee_category_id,
        note: row.note,
        kind_payload: row.kind_payload,
        created_at: row.created_at,
        updated_at: row.updated_at,
        tag_ids,
        deleted_at: None,
    })
}

pub fn get_entry(conn: &Connection, id: &str) -> Result<EntryDto> {
    let mut stmt = conn.prepare(
        "SELECT id, kind_id, amount_minor, occurred_at, account_id, counter_account_id,
                counter_amount_minor, category_id, fee_category_id, note, kind_payload,
                created_at, updated_at FROM entry WHERE id = ?1 AND deleted_at IS NULL",
    )?;
    let row = stmt
        .query_row([id], map_entry_row)
        .optional()?
        .ok_or_else(|| AppError::new("error.entryNotFound"))?;
    to_dto(conn, row)
}

fn validate_write(conn: &Connection, input: &EntryWrite) -> Result<()> {
    let desc = kinds::require_implemented(&input.kind_id)?;
    kinds::validate_kind_fields(desc, input)?;
    require_account(conn, &input.account_id)?;
    if let Some(c) = &input.counter_account_id {
        require_account(conn, c)?;
    }
    require_subcategory(conn, &input.category_id)?;
    if let Some(fee) = &input.fee_category_id {
        require_subcategory(conn, fee)?;
    }
    time_util::parse_utc_minute(&input.occurred_at)?;
    Ok(())
}

pub fn create_entry(conn: &mut Connection, mut input: EntryWrite) -> Result<EntryDto> {
    kinds::normalize_write(&mut input);
    validate_write(conn, &input)?;
    let id = new_id();
    let now = time_util::now_utc_minute();
    let occurred = time_util::format_utc_minute(time_util::parse_utc_minute(&input.occurred_at)?);
    let payload = input.kind_payload.clone().unwrap_or_else(kinds::default_payload);
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO entry (
            id, kind_id, amount_minor, occurred_at, account_id, counter_account_id,
            counter_amount_minor, category_id, fee_category_id, note, kind_payload,
            created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            id,
            input.kind_id,
            input.amount_minor,
            occurred,
            input.account_id,
            input.counter_account_id,
            input.counter_amount_minor,
            input.category_id,
            input.fee_category_id,
            input.note,
            payload,
            now,
            now
        ],
    )?;
    upsert_tags(&tx, &id, &input.tag_names)?;
    tx.commit()?;
    get_entry(conn, &id)
}

pub fn update_entry(conn: &mut Connection, id: &str, mut input: EntryWrite) -> Result<EntryDto> {
    let existing = get_entry(conn, id)?;
    if registry::get(&existing.kind_id).is_none() && input.kind_id == existing.kind_id {
        let tx = conn.transaction()?;
        tx.execute(
            "UPDATE entry SET note = ?1, updated_at = ?2 WHERE id = ?3 AND deleted_at IS NULL",
            params![input.note, time_util::now_utc_minute(), id],
        )?;
        upsert_tags(&tx, id, &input.tag_names)?;
        tx.commit()?;
        return get_entry(conn, id);
    }
    kinds::normalize_write(&mut input);
    validate_write(conn, &input)?;
    let occurred = time_util::format_utc_minute(time_util::parse_utc_minute(&input.occurred_at)?);
    let payload = input.kind_payload.clone().unwrap_or_else(kinds::default_payload);
    let tx = conn.transaction()?;
    let n = tx.execute(
        "UPDATE entry SET
            kind_id = ?1, amount_minor = ?2, occurred_at = ?3, account_id = ?4,
            counter_account_id = ?5, counter_amount_minor = ?6, category_id = ?7,
            fee_category_id = ?8, note = ?9, kind_payload = ?10, updated_at = ?11
         WHERE id = ?12 AND deleted_at IS NULL",
        params![
            input.kind_id,
            input.amount_minor,
            occurred,
            input.account_id,
            input.counter_account_id,
            input.counter_amount_minor,
            input.category_id,
            input.fee_category_id,
            input.note,
            payload,
            time_util::now_utc_minute(),
            id
        ],
    )?;
    if n == 0 {
        return Err(AppError::new("error.entryNotFound"));
    }
    upsert_tags(&tx, id, &input.tag_names)?;
    tx.commit()?;
    get_entry(conn, id)
}

pub fn delete_entry(conn: &Connection, id: &str) -> Result<()> {
    trash::mark_deleted(conn, "entry", id, "error.entryNotFound")
}

pub fn list_entries(conn: &Connection, filter: &LedgerFilter) -> Result<Vec<EntryDto>> {
    let from = time_util::parse_optional_local_date(&filter.from_date)?;
    let to = time_util::parse_optional_local_date(&filter.to_date)?;
    if let (Some(from), Some(to)) = (from, to) {
        if from > to {
            return Err(AppError::new("error.invalidRange"));
        }
    }

    let mut sql = String::from(
        "SELECT id, kind_id, amount_minor, occurred_at, account_id, counter_account_id,
                counter_amount_minor, category_id, fee_category_id, note, kind_payload,
                created_at, updated_at FROM entry WHERE deleted_at IS NULL",
    );
    let mut binds: Vec<String> = Vec::new();
    if let Some(from) = from {
        let start = time_util::format_utc_minute(time_util::local_date_start_utc(from));
        binds.push(start);
        sql.push_str(&format!(" AND occurred_at >= ?{}", binds.len()));
    }
    if let Some(to) = to {
        let end = time_util::local_date_end_utc(to)
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        binds.push(end);
        sql.push_str(&format!(" AND occurred_at <= ?{}", binds.len()));
    }
    sql.push_str(" ORDER BY occurred_at DESC, created_at DESC, id DESC");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(binds.iter()), map_entry_row)?;
    let mut collected = Vec::new();
    for row in rows {
        collected.push(row?);
    }

    let sub_ids = if let Some(cid) = &filter.category_id {
        category_match_ids(conn, cid)?
    } else {
        Vec::new()
    };

    let mut out = Vec::new();
    for row in collected {
        if !time_util::in_optional_local_range(&row.occurred_at, from, to)? {
            continue;
        }
        if !filter.kind_ids.is_empty() && !filter.kind_ids.iter().any(|k| k == &row.kind_id) {
            continue;
        }
        if !filter.account_ids.is_empty() {
            let hit = filter.account_ids.iter().any(|a| {
                a == &row.account_id || row.counter_account_id.as_ref() == Some(a)
            });
            if !hit {
                continue;
            }
        }
        if let Some(cid) = &filter.category_id {
            let hit = sub_ids.iter().any(|s| {
                s == &row.category_id || row.fee_category_id.as_ref() == Some(s)
            }) || row.category_id == *cid
                || row.fee_category_id.as_ref() == Some(cid);
            if !hit {
                continue;
            }
        }
        if let Some(tag_id) = &filter.tag_id {
            let n: i64 = conn.query_row(
                "SELECT COUNT(*) FROM entry_tag WHERE entry_id = ?1 AND tag_id = ?2",
                params![row.id, tag_id],
                |r| r.get(0),
            )?;
            if n == 0 {
                continue;
            }
        }
        if let Some(q) = &filter.note_contains {
            let q = q.trim();
            if !q.is_empty() {
                let note = row.note.clone().unwrap_or_default();
                if !note.to_lowercase().contains(&q.to_lowercase()) {
                    continue;
                }
            }
        }
        out.push(to_dto(conn, row)?);
    }
    Ok(out)
}

pub fn list_accounts_including_deleted(conn: &Connection) -> Result<Vec<AccountDto>> {
    let entries = list_all_entry_rows(conn)?;
    let mut stmt = conn.prepare(
        "SELECT id, name, account_kind, opening_balance_minor, opening_debt_minor, opening_at, note, sort_order, preset_key, deleted_at
         FROM account ORDER BY sort_order ASC, id ASC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, i64>(3)?,
            r.get::<_, i64>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, Option<String>>(6)?,
            r.get::<_, i32>(7)?,
            r.get::<_, Option<String>>(8)?,
            r.get::<_, Option<String>>(9)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (id, name, account_kind, opening, opening_debt, opening_at, note, sort_order, preset_key, deleted_at) =
            row?;
        let balance_minor = balance::balance_for_account(opening, &opening_at, &id, &entries);
        let debt_minor = balance::debt_for_account(opening_debt, &opening_at, &id, &entries);
        out.push(AccountDto {
            id,
            name,
            account_kind,
            opening_balance_minor: opening,
            opening_debt_minor: opening_debt,
            opening_at,
            note,
            sort_order,
            preset_key,
            balance_minor,
            debt_minor,
            deleted_at,
        });
    }
    Ok(out)
}

pub fn list_categories_including_deleted(conn: &Connection) -> Result<Vec<CategoryDto>> {
    let mut stmt = conn.prepare(
        "SELECT id, parent_id, name, preset_key, sort_order, color_hex, deleted_at FROM category
         ORDER BY parent_id IS NOT NULL, sort_order ASC, id ASC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(CategoryDto {
            id: r.get(0)?,
            parent_id: r.get(1)?,
            name: r.get(2)?,
            preset_key: r.get(3)?,
            sort_order: r.get(4)?,
            color_hex: r.get(5)?,
            deleted_at: r.get(6)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn list_entries_including_deleted(conn: &Connection) -> Result<Vec<EntryDto>> {
    let mut stmt = conn.prepare(
        "SELECT id, kind_id, amount_minor, occurred_at, account_id, counter_account_id,
                counter_amount_minor, category_id, fee_category_id, note, kind_payload,
                created_at, updated_at, deleted_at FROM entry
         ORDER BY occurred_at DESC, created_at DESC, id DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((map_entry_row(r)?, r.get::<_, Option<String>>(13)?))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (entry, deleted_at) = row?;
        let mut dto = to_dto(conn, entry)?;
        dto.deleted_at = deleted_at;
        out.push(dto);
    }
    Ok(out)
}

fn category_match_ids(conn: &Connection, id: &str) -> Result<Vec<String>> {
    let parent: Option<String> = conn
        .query_row(
            "SELECT parent_id FROM category WHERE id = ?1 AND deleted_at IS NULL",
            [id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::new("error.categoryNotFound"))?;
    if parent.is_some() {
        return Ok(vec![id.to_string()]);
    }
    let mut stmt = conn.prepare(
        "SELECT id FROM category WHERE parent_id = ?1 AND deleted_at IS NULL",
    )?;
    let rows = stmt.query_map([id], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_settings(conn: &Connection) -> Result<SettingsDto> {
    conn.query_row(
        "SELECT currency_code, default_account_id, schema_version, ui_language, color_scheme,
                ui_theme, default_fee_category_id, report_mode, report_side, report_custom_from,
                report_custom_to, last_kind_id, last_account_id, last_counter_account_id,
                last_category_id, last_fee_category_id, last_occurred_at
         FROM ledger_settings WHERE id = 1",
        [],
        |r| {
            Ok(SettingsDto {
                currency_code: r.get(0)?,
                default_account_id: r.get(1)?,
                schema_version: r.get(2)?,
                ui_language: r.get(3)?,
                color_scheme: r.get(4)?,
                ui_theme: r.get(5)?,
                default_fee_category_id: r.get(6)?,
                report_mode: r.get(7)?,
                report_side: r.get(8)?,
                report_custom_from: r.get(9)?,
                report_custom_to: r.get(10)?,
                last_kind_id: r.get(11)?,
                last_account_id: r.get(12)?,
                last_counter_account_id: r.get(13)?,
                last_category_id: r.get(14)?,
                last_fee_category_id: r.get(15)?,
                last_occurred_at: r.get(16)?,
            })
        },
    )
    .map_err(Into::into)
}

pub fn update_settings(conn: &Connection, patch: SettingsDto) -> Result<SettingsDto> {
    if !patch.currency_code.is_empty() && patch.currency_code != "CNY" {
        return Err(AppError::new("error.currencyLocked"));
    }
    require_account(conn, &patch.default_account_id)?;
    if let Some(fee) = &patch.default_fee_category_id {
        require_subcategory(conn, fee)?;
    }
    if let Some(lang) = &patch.ui_language {
        if lang != "en" && lang != "zh-Hans" {
            return Err(AppError::new("error.languageInvalid"));
        }
    }
    if let Some(scheme) = &patch.color_scheme {
        if scheme != "light" && scheme != "dark" && scheme != "system" {
            return Err(AppError::new("error.colorSchemeInvalid"));
        }
    }
    if let Some(theme) = &patch.ui_theme {
        if !matches!(
            theme.as_str(),
            "metal" | "claude" | "vscode" | "github" | "tiktok"
        ) {
            return Err(AppError::new("error.uiThemeInvalid"));
        }
    }
    conn.execute(
        "UPDATE ledger_settings SET
            default_account_id = ?1, ui_language = ?2, color_scheme = ?3, ui_theme = ?4,
            default_fee_category_id = ?5, report_mode = ?6, report_side = ?7,
            report_custom_from = ?8, report_custom_to = ?9,
            last_kind_id = ?10, last_account_id = ?11, last_counter_account_id = ?12,
            last_category_id = ?13, last_fee_category_id = ?14, last_occurred_at = ?15
         WHERE id = 1",
        params![
            patch.default_account_id,
            patch.ui_language,
            patch.color_scheme,
            empty_to_none(patch.ui_theme),
            patch.default_fee_category_id,
            patch.report_mode,
            patch.report_side,
            patch.report_custom_from,
            patch.report_custom_to,
            empty_to_none(patch.last_kind_id),
            empty_to_none(patch.last_account_id),
            empty_to_none(patch.last_counter_account_id),
            empty_to_none(patch.last_category_id),
            empty_to_none(patch.last_fee_category_id),
            empty_to_none(patch.last_occurred_at),
        ],
    )?;
    get_settings(conn)
}

pub(super) fn empty_to_none(value: Option<String>) -> Option<String> {
    value.and_then(|s| {
        let t = s.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    })
}

pub fn system_language() -> Option<String> {
    let loc = sys_locale::get_locale()?;
    let lower = loc.to_lowercase();
    if lower.starts_with("zh") {
        Some("zh-Hans".into())
    } else if lower.starts_with("en") {
        Some("en".into())
    } else {
        None
    }
}

pub fn resolved_language(settings: &SettingsDto) -> String {
    settings
        .ui_language
        .clone()
        .or_else(system_language)
        .unwrap_or_else(|| "en".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinds::registry;

    struct TestDb {
        _dir: tempfile::TempDir,
        conn: Connection,
    }

    fn temp_conn() -> TestDb {
        let dir = tempfile::tempdir().unwrap();
        let conn = open_at(&dir.path().join("ledger.sqlite")).unwrap();
        TestDb { _dir: dir, conn }
    }

    fn category_id_by_preset(conn: &Connection, key: &str) -> Result<String> {
        conn.query_row(
            "SELECT id FROM category WHERE preset_key = ?1",
            [key],
            |r| r.get(0),
        )
        .map_err(|_| AppError::new("error.categoryNotFound"))
    }

    #[test]
    fn migrates_existing_file_missing_deleted_at() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ledger.sqlite");
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                r#"
                PRAGMA foreign_keys = ON;
                CREATE TABLE account (
                  id TEXT PRIMARY KEY,
                  name TEXT,
                  account_kind TEXT NOT NULL,
                  opening_balance_minor INTEGER NOT NULL,
                  opening_debt_minor INTEGER NOT NULL DEFAULT 0,
                  opening_at TEXT NOT NULL,
                  note TEXT,
                  sort_order INTEGER NOT NULL,
                  preset_key TEXT
                );
                CREATE TABLE category (
                  id TEXT PRIMARY KEY,
                  parent_id TEXT REFERENCES category(id),
                  name TEXT,
                  preset_key TEXT,
                  sort_order INTEGER NOT NULL,
                  color_hex TEXT
                );
                CREATE TABLE tag (id TEXT PRIMARY KEY, name TEXT NOT NULL);
                CREATE TABLE ledger_settings (
                  id INTEGER PRIMARY KEY CHECK (id = 1),
                  currency_code TEXT NOT NULL,
                  default_account_id TEXT NOT NULL REFERENCES account(id),
                  schema_version INTEGER NOT NULL
                );
                CREATE TABLE entry (
                  id TEXT PRIMARY KEY,
                  kind_id TEXT NOT NULL,
                  amount_minor INTEGER NOT NULL,
                  occurred_at TEXT NOT NULL,
                  account_id TEXT NOT NULL REFERENCES account(id),
                  counter_account_id TEXT REFERENCES account(id),
                  counter_amount_minor INTEGER,
                  category_id TEXT NOT NULL REFERENCES category(id),
                  fee_category_id TEXT REFERENCES category(id),
                  note TEXT,
                  kind_payload TEXT NOT NULL,
                  created_at TEXT NOT NULL,
                  updated_at TEXT NOT NULL
                );
                CREATE TABLE pending_entry (
                  id TEXT PRIMARY KEY,
                  amount_minor INTEGER NOT NULL,
                  occurred_at TEXT NOT NULL,
                  kind_id TEXT,
                  account_id TEXT REFERENCES account(id),
                  counter_account_id TEXT REFERENCES account(id),
                  counter_amount_minor INTEGER,
                  category_id TEXT REFERENCES category(id),
                  fee_category_id TEXT REFERENCES category(id),
                  note TEXT,
                  created_at TEXT NOT NULL,
                  updated_at TEXT NOT NULL
                );
                INSERT INTO account (id, name, account_kind, opening_balance_minor, opening_debt_minor, opening_at, note, sort_order, preset_key)
                VALUES ('a1', 'Cash', 'cash', 0, 0, '1970-01-01T00:00:00Z', NULL, 0, NULL);
                INSERT INTO ledger_settings (id, currency_code, default_account_id, schema_version)
                VALUES (1, 'CNY', 'a1', 4);
                "#,
            )
            .unwrap();
        }
        let conn = open_at(&path).unwrap();
        let version: i32 = conn
            .query_row("SELECT schema_version FROM ledger_settings WHERE id = 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 5);
        for (table, col) in [
            ("account", "deleted_at"),
            ("category", "deleted_at"),
            ("entry", "deleted_at"),
            ("pending_entry", "deleted_at"),
        ] {
            let n: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM pragma_table_info(?1) WHERE name = ?2",
                    rusqlite::params![table, col],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(n, 1, "{table}.{col}");
        }
        assert_eq!(list_accounts(&conn).unwrap().len(), 1);
        assert_eq!(trash_count(&conn).unwrap(), 0);
    }

    #[test]
    fn seeds_default_and_tree() {
        let db = temp_conn();
        let conn = &db.conn;
        let accounts = list_accounts(&conn).unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].preset_key.as_deref(), Some("preset.account.default"));
        let cats = list_categories(&conn).unwrap();
        assert!(cats.iter().any(|c| c.preset_key.as_deref() == Some("preset.category.transfer.fee")));
        let settings = get_settings(&conn).unwrap();
        assert_eq!(settings.currency_code, "CNY");
        assert_eq!(settings.schema_version, 5);
        assert_eq!(settings.ui_theme, None);
        assert!(cats
            .iter()
            .any(|c| c.preset_key.as_deref() == Some("preset.category.finance.loan")));
    }

    #[test]
    fn persists_and_rejects_ui_theme() {
        let db = temp_conn();
        let mut settings = get_settings(&db.conn).unwrap();
        settings.ui_theme = Some("tiktok".into());
        let saved = update_settings(&db.conn, settings.clone()).unwrap();
        assert_eq!(saved.ui_theme.as_deref(), Some("tiktok"));
        settings.ui_theme = Some("solarized".into());
        assert_eq!(
            update_settings(&db.conn, settings).unwrap_err().code,
            "error.uiThemeInvalid"
        );
    }

    #[test]
    fn expense_and_transfer_roundtrip() {
        let mut db = temp_conn();
        let account = list_accounts(&db.conn).unwrap().pop().unwrap();
        let food = category_id_by_preset(&db.conn, "preset.category.food.dining").unwrap();
        let withdraw = category_id_by_preset(&db.conn, "preset.category.transfer.wechat").unwrap();
        let fee_cat = category_id_by_preset(&db.conn, "preset.category.transfer.fee").unwrap();
        let bank = create_account(
            &db.conn,
            AccountWrite {
                name: "Bank".into(),
                account_kind: "bank".into(),
                opening_balance_minor: 0,
                opening_debt_minor: 0,
                opening_at: time_util::OPENING_EPOCH.into(),
                note: None,
            },
        )
        .unwrap();

        create_entry(
            &mut db.conn,
            EntryWrite {
                kind_id: "expense".into(),
                amount_minor: 1250,
                occurred_at: "2026-09-09T12:00:00Z".into(),
                account_id: account.id.clone(),
                counter_account_id: None,
                counter_amount_minor: None,
                category_id: food,
                fee_category_id: None,
                note: Some("lunch".into()),
                tag_names: vec!["Work".into()],
                kind_payload: None,
            },
        )
        .unwrap();

        create_entry(
            &mut db.conn,
            EntryWrite {
                kind_id: "transfer".into(),
                amount_minor: 50000,
                occurred_at: "2026-09-09T13:00:00Z".into(),
                account_id: account.id.clone(),
                counter_account_id: Some(bank.id.clone()),
                counter_amount_minor: Some(49900),
                category_id: withdraw,
                fee_category_id: Some(fee_cat),
                note: None,
                tag_names: vec![],
                kind_payload: None,
            },
        )
        .unwrap();

        let accounts = list_accounts(&db.conn).unwrap();
        let src = accounts.iter().find(|a| a.id == account.id).unwrap();
        let dst = accounts.iter().find(|a| a.id == bank.id).unwrap();
        assert_eq!(src.balance_minor, -1250 - 50000);
        assert_eq!(dst.balance_minor, 49900);

        let (inc, exp) = registry::pnl_amounts("transfer", 50000, Some(49900));
        assert_eq!(inc, 0);
        assert_eq!(exp, 100);

        let used = account_usage(&db.conn, &account.id).unwrap();
        assert!(used >= 2);
        assert!(delete_account(&db.conn, &account.id).is_err());
    }

    #[test]
    fn transfer_rejects_same_account_and_overflow_dest() {
        let mut db = temp_conn();
        let account = list_accounts(&db.conn).unwrap().pop().unwrap();
        let cat = category_id_by_preset(&db.conn, "preset.category.transfer.internal").unwrap();
        let err = create_entry(
            &mut db.conn,
            EntryWrite {
                kind_id: "transfer".into(),
                amount_minor: 100,
                occurred_at: "2026-09-09T12:00:00Z".into(),
                account_id: account.id.clone(),
                counter_account_id: Some(account.id.clone()),
                counter_amount_minor: Some(100),
                category_id: cat.clone(),
                fee_category_id: None,
                note: None,
                tag_names: vec![],
                kind_payload: None,
            },
        )
        .unwrap_err();
        assert_eq!(err.code, "error.transferSameAccount");

        let bank = create_account(
            &db.conn,
            AccountWrite {
                name: "Bank".into(),
                account_kind: "bank".into(),
                opening_balance_minor: 0,
                opening_debt_minor: 0,
                opening_at: time_util::OPENING_EPOCH.into(),
                note: None,
            },
        )
        .unwrap();
        let err = create_entry(
            &mut db.conn,
            EntryWrite {
                kind_id: "transfer".into(),
                amount_minor: 100,
                occurred_at: "2026-09-09T12:00:00Z".into(),
                account_id: account.id,
                counter_account_id: Some(bank.id),
                counter_amount_minor: Some(200),
                category_id: cat,
                fee_category_id: None,
                note: None,
                tag_names: vec![],
                kind_payload: None,
            },
        )
        .unwrap_err();
        assert_eq!(err.code, "error.transferDestExceedsSource");
    }

    #[test]
    fn prepayment_raises_debt_not_balance_and_repayment_needs_counter() {
        let mut db = temp_conn();
        let cash = list_accounts(&db.conn).unwrap().pop().unwrap();
        let housing = category_id_by_preset(&db.conn, "preset.category.housing.rent").unwrap();
        let repay_cat = category_id_by_preset(&db.conn, "preset.category.finance.repayment").unwrap();
        let debt_acct = create_account(
            &db.conn,
            AccountWrite {
                name: "Loan".into(),
                account_kind: "other".into(),
                opening_balance_minor: 0,
                opening_debt_minor: 5000,
                opening_at: time_util::OPENING_EPOCH.into(),
                note: None,
            },
        )
        .unwrap();

        create_entry(
            &mut db.conn,
            EntryWrite {
                kind_id: "prepayment".into(),
                amount_minor: 2000,
                occurred_at: "2026-09-09T12:00:00Z".into(),
                account_id: debt_acct.id.clone(),
                counter_account_id: None,
                counter_amount_minor: None,
                category_id: housing,
                fee_category_id: None,
                note: None,
                tag_names: vec![],
                kind_payload: None,
            },
        )
        .unwrap();

        let err = create_entry(
            &mut db.conn,
            EntryWrite {
                kind_id: "repayment".into(),
                amount_minor: 800,
                occurred_at: "2026-09-09T13:00:00Z".into(),
                account_id: cash.id.clone(),
                counter_account_id: None,
                counter_amount_minor: None,
                category_id: repay_cat.clone(),
                fee_category_id: None,
                note: None,
                tag_names: vec![],
                kind_payload: None,
            },
        )
        .unwrap_err();
        assert_eq!(err.code, "error.counterAccountRequired");

        create_entry(
            &mut db.conn,
            EntryWrite {
                kind_id: "repayment".into(),
                amount_minor: 800,
                occurred_at: "2026-09-09T13:00:00Z".into(),
                account_id: cash.id.clone(),
                counter_account_id: Some(debt_acct.id.clone()),
                counter_amount_minor: None,
                category_id: repay_cat,
                fee_category_id: None,
                note: None,
                tag_names: vec![],
                kind_payload: None,
            },
        )
        .unwrap();

        let accounts = list_accounts(&db.conn).unwrap();
        let cash_row = accounts.iter().find(|a| a.id == cash.id).unwrap();
        let debt_row = accounts.iter().find(|a| a.id == debt_acct.id).unwrap();
        assert_eq!(cash_row.balance_minor, -800);
        assert_eq!(cash_row.debt_minor, 0);
        assert_eq!(debt_row.balance_minor, 0);
        assert_eq!(debt_row.debt_minor, 5000 + 2000 - 800);
        let (inc, exp) = registry::pnl_amounts("prepayment", 2000, None);
        assert_eq!((inc, exp), (0, 2000));
    }

    #[test]
    fn loan_raises_balance_and_counter_debt() {
        let mut db = temp_conn();
        let cash = list_accounts(&db.conn).unwrap().pop().unwrap();
        let loan_cat = category_id_by_preset(&db.conn, "preset.category.finance.loan").unwrap();
        let debt_acct = create_account(
            &db.conn,
            AccountWrite {
                name: "Credit".into(),
                account_kind: "credit".into(),
                opening_balance_minor: 0,
                opening_debt_minor: 0,
                opening_at: time_util::OPENING_EPOCH.into(),
                note: None,
            },
        )
        .unwrap();

        let err = create_entry(
            &mut db.conn,
            EntryWrite {
                kind_id: "loan".into(),
                amount_minor: 8000,
                occurred_at: "2026-09-09T12:00:00Z".into(),
                account_id: cash.id.clone(),
                counter_account_id: None,
                counter_amount_minor: None,
                category_id: loan_cat.clone(),
                fee_category_id: None,
                note: None,
                tag_names: vec![],
                kind_payload: None,
            },
        )
        .unwrap_err();
        assert_eq!(err.code, "error.counterAccountRequired");

        create_entry(
            &mut db.conn,
            EntryWrite {
                kind_id: "loan".into(),
                amount_minor: 8000,
                occurred_at: "2026-09-09T12:00:00Z".into(),
                account_id: cash.id.clone(),
                counter_account_id: Some(debt_acct.id.clone()),
                counter_amount_minor: None,
                category_id: loan_cat,
                fee_category_id: None,
                note: None,
                tag_names: vec![],
                kind_payload: None,
            },
        )
        .unwrap();

        let accounts = list_accounts(&db.conn).unwrap();
        let cash_row = accounts.iter().find(|a| a.id == cash.id).unwrap();
        let debt_row = accounts.iter().find(|a| a.id == debt_acct.id).unwrap();
        assert_eq!(cash_row.balance_minor, 8000);
        assert_eq!(cash_row.debt_minor, 0);
        assert_eq!(debt_row.balance_minor, 0);
        assert_eq!(debt_row.debt_minor, 8000);
        let (inc, exp) = registry::pnl_amounts("loan", 8000, None);
        assert_eq!((inc, exp), (0, 0));
    }

    #[test]
    fn loan_allows_same_account_for_balance_and_debt() {
        let mut db = temp_conn();
        let cash = list_accounts(&db.conn).unwrap().pop().unwrap();
        let loan_cat = category_id_by_preset(&db.conn, "preset.category.finance.loan").unwrap();
        create_entry(
            &mut db.conn,
            EntryWrite {
                kind_id: "loan".into(),
                amount_minor: 3000,
                occurred_at: "2026-09-09T12:00:00Z".into(),
                account_id: cash.id.clone(),
                counter_account_id: Some(cash.id.clone()),
                counter_amount_minor: None,
                category_id: loan_cat,
                fee_category_id: None,
                note: None,
                tag_names: vec![],
                kind_payload: None,
            },
        )
        .unwrap();
        let row = list_accounts(&db.conn)
            .unwrap()
            .into_iter()
            .find(|a| a.id == cash.id)
            .unwrap();
        assert_eq!(row.balance_minor, 3000);
        assert_eq!(row.debt_minor, 3000);
    }

    #[test]
    fn repayment_rejects_same_account() {
        let mut db = temp_conn();
        let cash = list_accounts(&db.conn).unwrap().pop().unwrap();
        let repay_cat = category_id_by_preset(&db.conn, "preset.category.finance.repayment").unwrap();
        let err = create_entry(
            &mut db.conn,
            EntryWrite {
                kind_id: "repayment".into(),
                amount_minor: 800,
                occurred_at: "2026-09-09T13:00:00Z".into(),
                account_id: cash.id.clone(),
                counter_account_id: Some(cash.id.clone()),
                counter_amount_minor: None,
                category_id: repay_cat,
                fee_category_id: None,
                note: None,
                tag_names: vec![],
                kind_payload: None,
            },
        )
        .unwrap_err();
        assert_eq!(err.code, "error.accountsMustDiffer");
    }

    #[test]
    fn pending_import_does_not_post_until_confirmed() {
        let mut db = temp_conn();
        let id = insert_pending_row(&db.conn, 1250, "2026-09-14T04:30:00Z").unwrap();
        assert_eq!(pending_count(&db.conn).unwrap(), 1);
        assert!(list_all_entry_rows(&db.conn).unwrap().is_empty());
        let err = post_pending_entry(&mut db.conn, &id).unwrap_err();
        assert_eq!(err.code, "error.kindRequired");
        let cash = list_accounts(&db.conn).unwrap().pop().unwrap();
        let food = category_id_by_preset(&db.conn, "preset.category.food.dining").unwrap();
        update_pending_entry(
            &mut db.conn,
            &id,
            PendingEntryWrite {
                amount_minor: 1250,
                occurred_at: "2026-09-14T04:30:00Z".into(),
                kind_id: Some("expense".into()),
                account_id: Some(cash.id.clone()),
                counter_account_id: None,
                counter_amount_minor: None,
                category_id: Some(food),
                fee_category_id: None,
                note: Some("imported".into()),
                tag_names: vec!["csv".into()],
            },
        )
        .unwrap();
        let posted = post_pending_entry(&mut db.conn, &id).unwrap();
        assert_eq!(posted.kind_id, "expense");
        assert_eq!(posted.amount_minor, 1250);
        assert_eq!(pending_count(&db.conn).unwrap(), 0);
        assert_eq!(list_all_entry_rows(&db.conn).unwrap().len(), 1);
    }

    #[test]
    fn update_main_color_and_reject_sub_or_unknown() {
        let db = temp_conn();
        let mains = list_categories(&db.conn)
            .unwrap()
            .into_iter()
            .filter(|c| c.parent_id.is_none())
            .collect::<Vec<_>>();
        let main = &mains[0];
        let next = palette::CATEGORY_PALETTE[20];
        let updated = update_category_color(&db.conn, &main.id, next).unwrap();
        let got = updated.iter().find(|c| c.id == main.id).unwrap();
        assert_eq!(got.color_hex.as_deref(), Some(next));

        let sub = list_categories(&db.conn)
            .unwrap()
            .into_iter()
            .find(|c| c.parent_id.as_deref() == Some(main.id.as_str()))
            .unwrap();
        assert_eq!(
            update_category_color(&db.conn, &sub.id, next).unwrap_err().code,
            "error.notMainCategory"
        );
        assert_eq!(
            update_category_color(&db.conn, &main.id, "#ffffff")
                .unwrap_err()
                .code,
            "error.colorInvalid"
        );
        assert_eq!(
            update_category_color(&db.conn, "missing", next)
                .unwrap_err()
                .code,
            "error.categoryNotFound"
        );
    }

    fn table_count(conn: &Connection, table: &str) -> i64 {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn delete_entry_goes_to_trash_and_leaves_balances() {
        let mut db = temp_conn();
        let account = list_accounts(&db.conn).unwrap().pop().unwrap();
        let food = category_id_by_preset(&db.conn, "preset.category.food.dining").unwrap();
        let entry = create_entry(
            &mut db.conn,
            EntryWrite {
                kind_id: "expense".into(),
                amount_minor: 1250,
                occurred_at: "2026-09-09T12:00:00Z".into(),
                account_id: account.id.clone(),
                counter_account_id: None,
                counter_amount_minor: None,
                category_id: food,
                fee_category_id: None,
                note: None,
                tag_names: vec![],
                kind_payload: None,
            },
        )
        .unwrap();
        assert_eq!(
            list_accounts(&db.conn)
                .unwrap()
                .into_iter()
                .find(|a| a.id == account.id)
                .unwrap()
                .balance_minor,
            -1250
        );
        delete_entry(&db.conn, &entry.id).unwrap();
        assert!(list_all_entry_rows(&db.conn).unwrap().is_empty());
        assert_eq!(table_count(&db.conn, "entry"), 1);
        assert_eq!(
            list_accounts(&db.conn)
                .unwrap()
                .into_iter()
                .find(|a| a.id == account.id)
                .unwrap()
                .balance_minor,
            0
        );
        let trash = list_trash(&db.conn).unwrap();
        assert_eq!(trash.len(), 1);
        assert_eq!(trash[0].item_kind, "entry");
        restore_trash(&db.conn, "entry", &entry.id).unwrap();
        assert_eq!(list_all_entry_rows(&db.conn).unwrap().len(), 1);
        assert_eq!(list_trash(&db.conn).unwrap().len(), 0);
        delete_entry(&db.conn, &entry.id).unwrap();
        purge_trash(&db.conn, "entry", &entry.id).unwrap();
        assert_eq!(table_count(&db.conn, "entry"), 0);
    }

    #[test]
    fn discard_pending_trashes_but_post_destroys() {
        let mut db = temp_conn();
        let discarded = insert_pending_row(&db.conn, 100, "2026-09-14T04:30:00Z").unwrap();
        delete_pending_entry(&db.conn, &discarded).unwrap();
        assert_eq!(pending_count(&db.conn).unwrap(), 0);
        assert_eq!(table_count(&db.conn, "pending_entry"), 1);
        assert_eq!(list_trash(&db.conn).unwrap()[0].item_kind, "pending");

        let posted = insert_pending_row(&db.conn, 1250, "2026-09-14T04:30:00Z").unwrap();
        let cash = list_accounts(&db.conn).unwrap().pop().unwrap();
        let food = category_id_by_preset(&db.conn, "preset.category.food.dining").unwrap();
        update_pending_entry(
            &mut db.conn,
            &posted,
            PendingEntryWrite {
                amount_minor: 1250,
                occurred_at: "2026-09-14T04:30:00Z".into(),
                kind_id: Some("expense".into()),
                account_id: Some(cash.id),
                counter_account_id: None,
                counter_amount_minor: None,
                category_id: Some(food),
                fee_category_id: None,
                note: None,
                tag_names: vec![],
            },
        )
        .unwrap();
        post_pending_entry(&mut db.conn, &posted).unwrap();
        assert_eq!(pending_count(&db.conn).unwrap(), 0);
        assert_eq!(table_count(&db.conn, "pending_entry"), 1);
        assert_eq!(list_all_entry_rows(&db.conn).unwrap().len(), 1);
        assert_eq!(
            list_trash(&db.conn)
                .unwrap()
                .iter()
                .filter(|i| i.item_kind == "pending")
                .count(),
            1
        );
    }

    #[test]
    fn unused_account_trashes_but_occupied_and_last_live_do_not() {
        let mut db = temp_conn();
        let live = list_accounts(&db.conn).unwrap().pop().unwrap();
        let extra = create_account(
            &db.conn,
            AccountWrite {
                name: "Spare".into(),
                account_kind: "cash".into(),
                opening_balance_minor: 0,
                opening_debt_minor: 0,
                opening_at: time_util::OPENING_EPOCH.into(),
                note: None,
            },
        )
        .unwrap();
        delete_account(&db.conn, &extra.id).unwrap();
        assert_eq!(list_accounts(&db.conn).unwrap().len(), 1);
        assert_eq!(table_count(&db.conn, "account"), 2);
        assert_eq!(
            delete_account(&db.conn, &live.id).unwrap_err().code,
            "error.lastAccount"
        );
        restore_trash(&db.conn, "account", &extra.id).unwrap();
        assert_eq!(list_accounts(&db.conn).unwrap().len(), 2);

        let food = category_id_by_preset(&db.conn, "preset.category.food.dining").unwrap();
        let entry = create_entry(
            &mut db.conn,
            EntryWrite {
                kind_id: "expense".into(),
                amount_minor: 100,
                occurred_at: "2026-09-09T12:00:00Z".into(),
                account_id: extra.id.clone(),
                counter_account_id: None,
                counter_amount_minor: None,
                category_id: food,
                fee_category_id: None,
                note: None,
                tag_names: vec![],
                kind_payload: None,
            },
        )
        .unwrap();
        delete_entry(&db.conn, &entry.id).unwrap();
        assert_eq!(
            delete_account(&db.conn, &extra.id).unwrap_err().code,
            "error.accountInUse"
        );
        empty_trash(&mut db.conn).unwrap();
        assert_eq!(list_trash(&db.conn).unwrap().len(), 0);
        assert_eq!(table_count(&db.conn, "entry"), 0);
        delete_account(&db.conn, &extra.id).unwrap();
        empty_trash(&mut db.conn).unwrap();
        assert_eq!(table_count(&db.conn, "account"), 1);
    }

    #[test]
    fn delete_main_category_trashes_children_and_restore_brings_them_back() {
        let db = temp_conn();
        let mains = list_categories(&db.conn)
            .unwrap()
            .into_iter()
            .filter(|c| c.parent_id.is_none())
            .collect::<Vec<_>>();
        let misc = mains
            .iter()
            .find(|c| c.preset_key.as_deref() == Some("preset.category.misc"))
            .cloned()
            .unwrap();
        let child_count = list_categories(&db.conn)
            .unwrap()
            .iter()
            .filter(|c| c.parent_id.as_deref() == Some(misc.id.as_str()))
            .count();
        assert!(child_count > 0);
        delete_category(&db.conn, &misc.id).unwrap();
        assert!(!list_categories(&db.conn)
            .unwrap()
            .iter()
            .any(|c| c.id == misc.id));
        let trash = list_trash(&db.conn).unwrap();
        assert!(trash.iter().any(|i| i.id == misc.id && i.item_kind == "category"));
        assert_eq!(
            trash
                .iter()
                .filter(|i| i.item_kind == "category" && i.parent_id.as_deref() == Some(misc.id.as_str()))
                .count(),
            child_count
        );
        restore_trash(&db.conn, "category", &misc.id).unwrap();
        assert_eq!(
            list_categories(&db.conn)
                .unwrap()
                .iter()
                .filter(|c| c.parent_id.as_deref() == Some(misc.id.as_str()))
                .count(),
            child_count
        );
    }
}
