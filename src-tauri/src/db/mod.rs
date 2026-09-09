mod schema;
mod seed;

use std::path::{Path, PathBuf};

use rusqlite::{params, params_from_iter, Connection, OptionalExtension};
use uuid::Uuid;

use crate::balance;
use crate::error::{AppError, Result};
use crate::kinds::{self, registry};
use crate::models::*;
use crate::time_util;

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
        "SELECT id, name, account_kind, opening_balance_minor, opening_at, note, sort_order, preset_key
         FROM account ORDER BY sort_order ASC, id ASC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, i64>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, Option<String>>(5)?,
            r.get::<_, i32>(6)?,
            r.get::<_, Option<String>>(7)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (id, name, account_kind, opening, opening_at, note, sort_order, preset_key) = row?;
        let balance_minor = balance::balance_for_account(opening, &opening_at, &id, &entries);
        out.push(AccountDto {
            id,
            name,
            account_kind,
            opening_balance_minor: opening,
            opening_at,
            note,
            sort_order,
            preset_key,
            balance_minor,
        });
    }
    Ok(out)
}

pub fn create_account(conn: &Connection, write: AccountWrite) -> Result<AccountDto> {
    if write.name.trim().is_empty() {
        return Err(AppError::new("error.accountNameRequired"));
    }
    if !account_kind_ok(&write.account_kind) {
        return Err(AppError::new("error.accountKindInvalid"));
    }
    let _ = time_util::parse_utc_minute(&write.opening_at)?;
    let max: i32 = conn
        .query_row("SELECT COALESCE(MAX(sort_order), -1) FROM account", [], |r| r.get(0))?;
    let id = new_id();
    conn.execute(
        "INSERT INTO account (id, name, account_kind, opening_balance_minor, opening_at, note, sort_order, preset_key)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)",
        params![
            id,
            write.name.trim(),
            write.account_kind,
            write.opening_balance_minor,
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
    if write.name.trim().is_empty() {
        return Err(AppError::new("error.accountNameRequired"));
    }
    if !account_kind_ok(&write.account_kind) {
        return Err(AppError::new("error.accountKindInvalid"));
    }
    let opening_at = time_util::format_utc_minute(time_util::parse_utc_minute(&write.opening_at)?);
    let n = conn.execute(
        "UPDATE account SET name = ?1, account_kind = ?2, opening_balance_minor = ?3, opening_at = ?4, note = ?5
         WHERE id = ?6",
        params![
            write.name.trim(),
            write.account_kind,
            write.opening_balance_minor,
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
    conn.query_row(
        "SELECT COUNT(*) FROM entry WHERE account_id = ?1 OR counter_account_id = ?1",
        [id],
        |r| r.get(0),
    )
    .map_err(Into::into)
}

pub fn delete_account(conn: &Connection, id: &str) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM account", [], |r| r.get(0))?;
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
    let n = conn.execute("DELETE FROM account WHERE id = ?1", [id])?;
    if n == 0 {
        return Err(AppError::new("error.accountNotFound"));
    }
    Ok(())
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
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

const PALETTE: &[&str] = &[
    "#5b7c99", "#b4532a", "#78716c", "#a16207", "#0e7490", "#4f46e5", "#3f6212",
    "#9f1239", "#52525b", "#be185d", "#0f766e", "#57534e", "#1e3a5f", "#6b7280",
    "#854d0e", "#115e59", "#6b21a8", "#9a3412", "#164e63",
];

fn next_main_color(conn: &Connection) -> Result<String> {
    let mut stmt = conn.prepare("SELECT color_hex FROM category WHERE parent_id IS NULL")?;
    let used: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(PALETTE
        .iter()
        .find(|c| !used.iter().any(|u| u.eq_ignore_ascii_case(c)))
        .copied()
        .unwrap_or("#6b7280")
        .to_string())
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
            "SELECT parent_id FROM category WHERE id = ?1",
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

pub fn rename_category(conn: &Connection, id: &str, name: &str) -> Result<Vec<CategoryDto>> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::new("error.categoryNameRequired"));
    }
    let n = conn.execute("UPDATE category SET name = ?1 WHERE id = ?2", params![name, id])?;
    if n == 0 {
        return Err(AppError::new("error.categoryNotFound"));
    }
    list_categories(conn)
}

pub fn category_usage(conn: &Connection, id: &str) -> Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM entry WHERE category_id = ?1 OR fee_category_id = ?1",
        [id],
        |r| r.get(0),
    )
    .map_err(Into::into)
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
        .query_row("SELECT parent_id FROM category WHERE id = ?1", [id], |r| r.get(0))
        .optional()?
        .ok_or_else(|| AppError::new("error.categoryNotFound"))?;

    if parent_id.is_none() {
        let mut child_ids = Vec::new();
        let mut stmt = conn.prepare("SELECT id FROM category WHERE parent_id = ?1")?;
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
            conn.execute("DELETE FROM category WHERE id = ?1", [child])?;
        }
        conn.execute("DELETE FROM category WHERE id = ?1", [id])?;
        Ok(())
    } else {
        if is_default_fee(conn, id)? {
            return Err(AppError::new("error.defaultFeeCategory"));
        }
        let used = category_usage(conn, id)?;
        if used > 0 {
            return Err(AppError::with_count("error.categoryInUse", used));
        }
        conn.execute("DELETE FROM category WHERE id = ?1", [id])?;
        Ok(())
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
    let n = conn.execute("DELETE FROM tag WHERE id = ?1", [id])?;
    if n == 0 {
        return Err(AppError::new("error.tagNotFound"));
    }
    Ok(())
}

fn upsert_tags(conn: &Connection, entry_id: &str, names: &[String]) -> Result<Vec<String>> {
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

fn require_subcategory(conn: &Connection, id: &str) -> Result<()> {
    let parent: Option<String> = conn
        .query_row("SELECT parent_id FROM category WHERE id = ?1", [id], |r| r.get(0))
        .optional()?
        .ok_or_else(|| AppError::new("error.categoryNotFound"))?;
    if parent.is_none() {
        return Err(AppError::new("error.notSubcategory"));
    }
    Ok(())
}

fn require_account(conn: &Connection, id: &str) -> Result<()> {
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM account WHERE id = ?1", [id], |r| r.get(0))?;
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
                created_at, updated_at FROM entry",
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
    })
}

pub fn get_entry(conn: &Connection, id: &str) -> Result<EntryDto> {
    let mut stmt = conn.prepare(
        "SELECT id, kind_id, amount_minor, occurred_at, account_id, counter_account_id,
                counter_amount_minor, category_id, fee_category_id, note, kind_payload,
                created_at, updated_at FROM entry WHERE id = ?1",
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
            "UPDATE entry SET note = ?1, updated_at = ?2 WHERE id = ?3",
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
         WHERE id = ?12",
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
    let n = conn.execute("DELETE FROM entry WHERE id = ?1", [id])?;
    if n == 0 {
        return Err(AppError::new("error.entryNotFound"));
    }
    Ok(())
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
                created_at, updated_at FROM entry",
    );
    let mut binds: Vec<String> = Vec::new();
    if let Some(from) = from {
        let start = time_util::format_utc_minute(time_util::local_date_start_utc(from));
        binds.push(start);
        sql.push_str(&format!(" WHERE occurred_at >= ?{}", binds.len()));
    }
    if let Some(to) = to {
        let end = time_util::local_date_end_utc(to)
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        binds.push(end);
        if binds.len() == 1 {
            sql.push_str(" WHERE occurred_at <= ?1");
        } else {
            sql.push_str(" AND occurred_at <= ?2");
        }
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

fn category_match_ids(conn: &Connection, id: &str) -> Result<Vec<String>> {
    let parent: Option<String> = conn
        .query_row("SELECT parent_id FROM category WHERE id = ?1", [id], |r| r.get(0))
        .optional()?
        .ok_or_else(|| AppError::new("error.categoryNotFound"))?;
    if parent.is_some() {
        return Ok(vec![id.to_string()]);
    }
    let mut stmt = conn.prepare("SELECT id FROM category WHERE parent_id = ?1")?;
    let rows = stmt.query_map([id], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_settings(conn: &Connection) -> Result<SettingsDto> {
    conn.query_row(
        "SELECT currency_code, default_account_id, schema_version, ui_language, color_scheme,
                default_fee_category_id, report_mode, report_side, report_custom_from, report_custom_to
         FROM ledger_settings WHERE id = 1",
        [],
        |r| {
            Ok(SettingsDto {
                currency_code: r.get(0)?,
                default_account_id: r.get(1)?,
                schema_version: r.get(2)?,
                ui_language: r.get(3)?,
                color_scheme: r.get(4)?,
                default_fee_category_id: r.get(5)?,
                report_mode: r.get(6)?,
                report_side: r.get(7)?,
                report_custom_from: r.get(8)?,
                report_custom_to: r.get(9)?,
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
    conn.execute(
        "UPDATE ledger_settings SET
            default_account_id = ?1, ui_language = ?2, color_scheme = ?3,
            default_fee_category_id = ?4, report_mode = ?5, report_side = ?6,
            report_custom_from = ?7, report_custom_to = ?8
         WHERE id = 1",
        params![
            patch.default_account_id,
            patch.ui_language,
            patch.color_scheme,
            patch.default_fee_category_id,
            patch.report_mode,
            patch.report_side,
            patch.report_custom_from,
            patch.report_custom_to
        ],
    )?;
    get_settings(conn)
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
        assert_eq!(settings.schema_version, 1);
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
}
