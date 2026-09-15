use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{AppError, Result};
use crate::models::TrashItemDto;
use crate::time_util;

fn tags_for_entry(conn: &Connection, entry_id: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT tag_id FROM entry_tag WHERE entry_id = ?1")?;
    let rows = stmt.query_map([entry_id], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn tags_for_pending(conn: &Connection, pending_id: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT tag_id FROM pending_entry_tag WHERE pending_entry_id = ?1",
    )?;
    let rows = stmt.query_map([pending_id], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn trash_count(conn: &Connection) -> Result<i64> {
    let entry: i64 = conn.query_row(
        "SELECT COUNT(*) FROM entry WHERE deleted_at IS NOT NULL",
        [],
        |r| r.get(0),
    )?;
    let pending: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pending_entry WHERE deleted_at IS NOT NULL",
        [],
        |r| r.get(0),
    )?;
    let account: i64 = conn.query_row(
        "SELECT COUNT(*) FROM account WHERE deleted_at IS NOT NULL",
        [],
        |r| r.get(0),
    )?;
    let category: i64 = conn.query_row(
        "SELECT COUNT(*) FROM category WHERE deleted_at IS NOT NULL",
        [],
        |r| r.get(0),
    )?;
    Ok(entry + pending + account + category)
}

pub fn list_trash(conn: &Connection) -> Result<Vec<TrashItemDto>> {
    let mut items = Vec::new();

    let mut stmt = conn.prepare(
        "SELECT id, deleted_at, kind_id, amount_minor, occurred_at, account_id, counter_account_id,
                counter_amount_minor, category_id, fee_category_id, note
         FROM entry WHERE deleted_at IS NOT NULL",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(TrashItemDto {
            item_kind: "entry".into(),
            id: r.get(0)?,
            deleted_at: r.get(1)?,
            kind_id: Some(r.get(2)?),
            amount_minor: Some(r.get(3)?),
            occurred_at: Some(r.get(4)?),
            account_id: Some(r.get(5)?),
            counter_account_id: r.get(6)?,
            counter_amount_minor: r.get(7)?,
            category_id: Some(r.get(8)?),
            fee_category_id: r.get(9)?,
            note: r.get(10)?,
            name: None,
            parent_id: None,
            preset_key: None,
            account_kind: None,
            tag_ids: Vec::new(),
        })
    })?;
    for row in rows {
        let mut item = row?;
        item.tag_ids = tags_for_entry(conn, &item.id)?;
        items.push(item);
    }

    let mut stmt = conn.prepare(
        "SELECT id, deleted_at, kind_id, amount_minor, occurred_at, account_id, counter_account_id,
                counter_amount_minor, category_id, fee_category_id, note
         FROM pending_entry WHERE deleted_at IS NOT NULL",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(TrashItemDto {
            item_kind: "pending".into(),
            id: r.get(0)?,
            deleted_at: r.get(1)?,
            kind_id: r.get(2)?,
            amount_minor: Some(r.get(3)?),
            occurred_at: Some(r.get(4)?),
            account_id: r.get(5)?,
            counter_account_id: r.get(6)?,
            counter_amount_minor: r.get(7)?,
            category_id: r.get(8)?,
            fee_category_id: r.get(9)?,
            note: r.get(10)?,
            name: None,
            parent_id: None,
            preset_key: None,
            account_kind: None,
            tag_ids: Vec::new(),
        })
    })?;
    for row in rows {
        let mut item = row?;
        item.tag_ids = tags_for_pending(conn, &item.id)?;
        items.push(item);
    }

    let mut stmt = conn.prepare(
        "SELECT id, deleted_at, name, note, preset_key, account_kind
         FROM account WHERE deleted_at IS NOT NULL",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(TrashItemDto {
            item_kind: "account".into(),
            id: r.get(0)?,
            deleted_at: r.get(1)?,
            kind_id: None,
            amount_minor: None,
            occurred_at: None,
            account_id: None,
            counter_account_id: None,
            counter_amount_minor: None,
            category_id: None,
            fee_category_id: None,
            note: r.get(3)?,
            name: r.get(2)?,
            parent_id: None,
            preset_key: r.get(4)?,
            account_kind: Some(r.get(5)?),
            tag_ids: Vec::new(),
        })
    })?;
    for row in rows {
        items.push(row?);
    }

    let mut stmt = conn.prepare(
        "SELECT id, deleted_at, name, parent_id, preset_key
         FROM category WHERE deleted_at IS NOT NULL",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(TrashItemDto {
            item_kind: "category".into(),
            id: r.get(0)?,
            deleted_at: r.get(1)?,
            kind_id: None,
            amount_minor: None,
            occurred_at: None,
            account_id: None,
            counter_account_id: None,
            counter_amount_minor: None,
            category_id: None,
            fee_category_id: None,
            note: None,
            name: r.get(2)?,
            parent_id: r.get(3)?,
            preset_key: r.get(4)?,
            account_kind: None,
            tag_ids: Vec::new(),
        })
    })?;
    for row in rows {
        items.push(row?);
    }

    items.sort_by(|a, b| {
        b.deleted_at
            .cmp(&a.deleted_at)
            .then_with(|| b.id.cmp(&a.id))
    });
    Ok(items)
}

fn undelete(conn: &Connection, table: &str, id: &str) -> Result<usize> {
    let n = conn.execute(
        &format!("UPDATE {table} SET deleted_at = NULL WHERE id = ?1 AND deleted_at IS NOT NULL"),
        [id],
    )?;
    Ok(n)
}

fn restore_category(conn: &Connection, id: &str) -> Result<()> {
    let row: Option<(Option<String>, Option<String>)> = conn
        .query_row(
            "SELECT parent_id, deleted_at FROM category WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((parent_id, deleted_at)) = row else {
        return Err(AppError::new("error.categoryNotFound"));
    };
    if deleted_at.is_none() {
        return Err(AppError::new("error.trashNotFound"));
    }
    if let Some(parent_id) = parent_id {
        let parent_deleted: Option<Option<String>> = conn
            .query_row(
                "SELECT deleted_at FROM category WHERE id = ?1",
                [&parent_id],
                |r| r.get(0),
            )
            .optional()?;
        match parent_deleted {
            None => return Err(AppError::new("error.categoryNotFound")),
            Some(Some(_)) => return restore_category(conn, &parent_id),
            Some(None) => {}
        }
        if undelete(conn, "category", id)? == 0 {
            return Err(AppError::new("error.trashNotFound"));
        }
        Ok(())
    } else {
        undelete(conn, "category", id)?;
        conn.execute(
            "UPDATE category SET deleted_at = NULL WHERE parent_id = ?1 AND deleted_at IS NOT NULL",
            [id],
        )?;
        Ok(())
    }
}

pub fn restore_trash(conn: &Connection, item_kind: &str, id: &str) -> Result<()> {
    match item_kind {
        "entry" => {
            if undelete(conn, "entry", id)? == 0 {
                return Err(AppError::new("error.trashNotFound"));
            }
            Ok(())
        }
        "pending" => {
            if undelete(conn, "pending_entry", id)? == 0 {
                return Err(AppError::new("error.trashNotFound"));
            }
            Ok(())
        }
        "account" => {
            if undelete(conn, "account", id)? == 0 {
                return Err(AppError::new("error.trashNotFound"));
            }
            Ok(())
        }
        "category" => restore_category(conn, id),
        _ => Err(AppError::new("error.trashKindInvalid")),
    }
}

fn purge_row(conn: &Connection, table: &str, id: &str, not_found: &str) -> Result<()> {
    let n = conn.execute(
        &format!("DELETE FROM {table} WHERE id = ?1 AND deleted_at IS NOT NULL"),
        [id],
    )?;
    if n == 0 {
        return Err(AppError::new(not_found));
    }
    Ok(())
}

fn purge_category(conn: &Connection, id: &str) -> Result<()> {
    let row: Option<(Option<String>, Option<String>)> = conn
        .query_row(
            "SELECT parent_id, deleted_at FROM category WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((parent_id, deleted_at)) = row else {
        return Err(AppError::new("error.categoryNotFound"));
    };
    if deleted_at.is_none() {
        return Err(AppError::new("error.trashNotFound"));
    }
    if parent_id.is_none() {
        let live: i64 = conn.query_row(
            "SELECT COUNT(*) FROM category WHERE parent_id = ?1 AND deleted_at IS NULL",
            [id],
            |r| r.get(0),
        )?;
        if live > 0 {
            return Err(AppError::with_count("error.categoryInUse", live));
        }
        conn.execute(
            "DELETE FROM category WHERE parent_id = ?1 AND deleted_at IS NOT NULL",
            [id],
        )?;
    }
    conn.execute(
        "DELETE FROM category WHERE id = ?1 AND deleted_at IS NOT NULL",
        [id],
    )?;
    Ok(())
}

pub fn purge_trash(conn: &Connection, item_kind: &str, id: &str) -> Result<()> {
    match item_kind {
        "entry" => purge_row(conn, "entry", id, "error.trashNotFound"),
        "pending" => purge_row(conn, "pending_entry", id, "error.trashNotFound"),
        "account" => purge_row(conn, "account", id, "error.trashNotFound"),
        "category" => purge_category(conn, id),
        _ => Err(AppError::new("error.trashKindInvalid")),
    }
}

pub fn empty_trash(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM entry WHERE deleted_at IS NOT NULL", [])?;
    tx.execute("DELETE FROM pending_entry WHERE deleted_at IS NOT NULL", [])?;
    tx.execute(
        "DELETE FROM category WHERE deleted_at IS NOT NULL AND parent_id IS NOT NULL",
        [],
    )?;
    tx.execute(
        "DELETE FROM category WHERE deleted_at IS NOT NULL AND parent_id IS NULL",
        [],
    )?;
    tx.execute("DELETE FROM account WHERE deleted_at IS NOT NULL", [])?;
    tx.commit()?;
    Ok(())
}

pub fn mark_deleted(conn: &Connection, table: &str, id: &str, not_found: &str) -> Result<()> {
    let now = time_util::now_utc_minute();
    let n = conn.execute(
        &format!("UPDATE {table} SET deleted_at = ?1 WHERE id = ?2 AND deleted_at IS NULL"),
        params![now, id],
    )?;
    if n == 0 {
        return Err(AppError::new(not_found));
    }
    Ok(())
}
