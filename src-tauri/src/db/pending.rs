use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{AppError, Result};
use crate::kinds;
use crate::models::*;
use crate::time_util;

use super::{create_entry, empty_to_none, new_id, require_account, require_subcategory};

pub fn pending_count(conn: &Connection) -> Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM pending_entry", [], |r| r.get(0))
        .map_err(Into::into)
}

pub fn list_pending_entries(conn: &Connection) -> Result<Vec<PendingEntryDto>> {
    let mut stmt = conn.prepare(
        "SELECT id, amount_minor, occurred_at, kind_id, account_id, counter_account_id,
                counter_amount_minor, category_id, fee_category_id, note, created_at, updated_at
         FROM pending_entry
         ORDER BY occurred_at DESC, created_at DESC, id DESC",
    )?;
    let rows = stmt.query_map([], map_pending_row)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(to_pending_dto(conn, row?)?);
    }
    Ok(out)
}

pub fn get_pending_entry(conn: &Connection, id: &str) -> Result<PendingEntryDto> {
    let mut stmt = conn.prepare(
        "SELECT id, amount_minor, occurred_at, kind_id, account_id, counter_account_id,
                counter_amount_minor, category_id, fee_category_id, note, created_at, updated_at
         FROM pending_entry WHERE id = ?1",
    )?;
    let row = stmt
        .query_row([id], map_pending_row)
        .optional()?
        .ok_or_else(|| AppError::new("error.pendingNotFound"))?;
    to_pending_dto(conn, row)
}

pub fn insert_pending_row(conn: &Connection, amount_minor: i64, occurred_at: &str) -> Result<String> {
    if amount_minor <= 0 {
        return Err(AppError::new("error.amountInvalid"));
    }
    let id = new_id();
    let now = time_util::now_utc_minute();
    let occurred = time_util::format_utc_minute(time_util::parse_utc_minute(occurred_at)?);
    conn.execute(
        "INSERT INTO pending_entry (
            id, amount_minor, occurred_at, kind_id, account_id, counter_account_id,
            counter_amount_minor, category_id, fee_category_id, note, created_at, updated_at
         ) VALUES (?1, ?2, ?3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, ?4, ?4)",
        params![id, amount_minor, occurred, now],
    )?;
    Ok(id)
}

pub fn update_pending_entry(
    conn: &mut Connection,
    id: &str,
    mut input: PendingEntryWrite,
) -> Result<PendingEntryDto> {
    normalize_pending(&mut input);
    validate_pending_draft(conn, &input)?;
    let occurred = time_util::format_utc_minute(time_util::parse_utc_minute(&input.occurred_at)?);
    let now = time_util::now_utc_minute();
    let tx = conn.transaction()?;
    let n = tx.execute(
        "UPDATE pending_entry SET
            amount_minor = ?1, occurred_at = ?2, kind_id = ?3, account_id = ?4,
            counter_account_id = ?5, counter_amount_minor = ?6, category_id = ?7,
            fee_category_id = ?8, note = ?9, updated_at = ?10
         WHERE id = ?11",
        params![
            input.amount_minor,
            occurred,
            input.kind_id,
            input.account_id,
            input.counter_account_id,
            input.counter_amount_minor,
            input.category_id,
            input.fee_category_id,
            input.note,
            now,
            id
        ],
    )?;
    if n == 0 {
        return Err(AppError::new("error.pendingNotFound"));
    }
    upsert_pending_tags(&tx, id, &input.tag_names)?;
    tx.commit()?;
    get_pending_entry(conn, id)
}

pub fn delete_pending_entry(conn: &Connection, id: &str) -> Result<()> {
    let n = conn.execute("DELETE FROM pending_entry WHERE id = ?1", [id])?;
    if n == 0 {
        return Err(AppError::new("error.pendingNotFound"));
    }
    Ok(())
}

pub fn post_pending_entry(conn: &mut Connection, id: &str) -> Result<EntryDto> {
    let pending = get_pending_entry(conn, id)?;
    let write = pending_to_entry_write(conn, &pending)?;
    let dto = create_entry(conn, write)?;
    delete_pending_entry(conn, id)?;
    Ok(dto)
}

fn pending_to_entry_write(conn: &Connection, pending: &PendingEntryDto) -> Result<EntryWrite> {
    let kind_id = pending
        .kind_id
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::new("error.kindRequired"))?;
    let account_id = pending
        .account_id
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::new("error.accountRequired"))?;
    let category_id = pending
        .category_id
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::new("error.categoryRequired"))?;
    let tag_names = pending_tag_names(conn, &pending.id)?;
    Ok(EntryWrite {
        kind_id,
        amount_minor: pending.amount_minor,
        occurred_at: pending.occurred_at.clone(),
        account_id,
        counter_account_id: pending.counter_account_id.clone(),
        counter_amount_minor: pending.counter_amount_minor,
        category_id,
        fee_category_id: pending.fee_category_id.clone(),
        note: pending.note.clone(),
        tag_names,
        kind_payload: None,
    })
}

fn pending_tag_names(conn: &Connection, id: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT t.name FROM pending_entry_tag p
         JOIN tag t ON t.id = p.tag_id
         WHERE p.pending_entry_id = ?1",
    )?;
    let rows = stmt.query_map([id], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn normalize_pending(input: &mut PendingEntryWrite) {
    input.kind_id = empty_to_none(input.kind_id.take());
    input.account_id = empty_to_none(input.account_id.take());
    input.counter_account_id = empty_to_none(input.counter_account_id.take());
    input.category_id = empty_to_none(input.category_id.take());
    input.fee_category_id = empty_to_none(input.fee_category_id.take());
    input.note = empty_to_none(input.note.take()).map(|s| {
        let t = s.trim();
        if t.is_empty() {
            String::new()
        } else {
            t.to_string()
        }
    });
    if input.note.as_ref().is_some_and(|s| s.is_empty()) {
        input.note = None;
    }
    if let Some(kind_id) = input.kind_id.clone() {
        if let Some(desc) = kinds::registry::get(&kind_id) {
            if !desc.counter_account_required {
                input.counter_account_id = None;
            }
            if !desc.counter_amount_required {
                input.counter_amount_minor = None;
                input.fee_category_id = None;
            }
        }
    }
}

fn validate_pending_draft(conn: &Connection, input: &PendingEntryWrite) -> Result<()> {
    if input.amount_minor <= 0 {
        return Err(AppError::new("error.amountInvalid"));
    }
    time_util::parse_utc_minute(&input.occurred_at)?;
    if let Some(kind_id) = &input.kind_id {
        kinds::require_implemented(kind_id)?;
    }
    if let Some(account_id) = &input.account_id {
        require_account(conn, account_id)?;
    }
    if let Some(counter) = &input.counter_account_id {
        require_account(conn, counter)?;
    }
    if let Some(category_id) = &input.category_id {
        require_subcategory(conn, category_id)?;
    }
    if let Some(fee) = &input.fee_category_id {
        require_subcategory(conn, fee)?;
    }
    Ok(())
}

fn upsert_pending_tags(conn: &Connection, pending_id: &str, names: &[String]) -> Result<()> {
    conn.execute(
        "DELETE FROM pending_entry_tag WHERE pending_entry_id = ?1",
        [pending_id],
    )?;
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
            "INSERT OR IGNORE INTO pending_entry_tag (pending_entry_id, tag_id) VALUES (?1, ?2)",
            params![pending_id, tag_id],
        )?;
    }
    Ok(())
}

struct PendingRow {
    id: String,
    amount_minor: i64,
    occurred_at: String,
    kind_id: Option<String>,
    account_id: Option<String>,
    counter_account_id: Option<String>,
    counter_amount_minor: Option<i64>,
    category_id: Option<String>,
    fee_category_id: Option<String>,
    note: Option<String>,
    created_at: String,
    updated_at: String,
}

fn map_pending_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<PendingRow> {
    Ok(PendingRow {
        id: r.get(0)?,
        amount_minor: r.get(1)?,
        occurred_at: r.get(2)?,
        kind_id: r.get(3)?,
        account_id: r.get(4)?,
        counter_account_id: r.get(5)?,
        counter_amount_minor: r.get(6)?,
        category_id: r.get(7)?,
        fee_category_id: r.get(8)?,
        note: r.get(9)?,
        created_at: r.get(10)?,
        updated_at: r.get(11)?,
    })
}

fn to_pending_dto(conn: &Connection, row: PendingRow) -> Result<PendingEntryDto> {
    let mut stmt = conn.prepare(
        "SELECT tag_id FROM pending_entry_tag WHERE pending_entry_id = ?1",
    )?;
    let tag_ids = stmt
        .query_map([&row.id], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(PendingEntryDto {
        id: row.id,
        amount_minor: row.amount_minor,
        occurred_at: row.occurred_at,
        kind_id: row.kind_id,
        account_id: row.account_id,
        counter_account_id: row.counter_account_id,
        counter_amount_minor: row.counter_amount_minor,
        category_id: row.category_id,
        fee_category_id: row.fee_category_id,
        note: row.note,
        created_at: row.created_at,
        updated_at: row.updated_at,
        tag_ids,
    })
}
