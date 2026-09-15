use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::db::schema::{PENDING_TABLES, SCHEMA_VERSION};
use crate::error::Result;

pub fn apply(conn: &Connection) -> Result<()> {
    add_column_if_missing(
        conn,
        "account",
        "opening_debt_minor",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    add_column_if_missing(conn, "ledger_settings", "last_kind_id", "TEXT")?;
    add_column_if_missing(conn, "ledger_settings", "last_account_id", "TEXT")?;
    add_column_if_missing(conn, "ledger_settings", "last_counter_account_id", "TEXT")?;
    add_column_if_missing(conn, "ledger_settings", "last_category_id", "TEXT")?;
    add_column_if_missing(conn, "ledger_settings", "last_fee_category_id", "TEXT")?;
    add_column_if_missing(conn, "ledger_settings", "last_occurred_at", "TEXT")?;
    add_column_if_missing(conn, "ledger_settings", "ui_theme", "TEXT")?;
    conn.execute_batch(PENDING_TABLES)?;
    seed_finance_loan(conn)?;
    conn.execute(
        "UPDATE ledger_settings SET schema_version = ?1 WHERE id = 1",
        params![SCHEMA_VERSION],
    )?;
    Ok(())
}

fn seed_finance_loan(conn: &Connection) -> Result<()> {
    let exists: Option<String> = conn
        .query_row(
            "SELECT id FROM category WHERE preset_key = 'preset.category.finance.loan'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    if exists.is_some() {
        return Ok(());
    }
    let finance_id: Option<String> = conn
        .query_row(
            "SELECT id FROM category WHERE preset_key = 'preset.category.finance' AND parent_id IS NULL",
            [],
            |r| r.get(0),
        )
        .optional()?;
    let Some(parent_id) = finance_id else {
        return Ok(());
    };
    let other: Option<(String, i32)> = conn
        .query_row(
            "SELECT id, sort_order FROM category
             WHERE parent_id = ?1 AND preset_key = 'preset.category.finance.other'",
            [&parent_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let sort_order = if let Some((other_id, other_sort)) = other {
        conn.execute(
            "UPDATE category SET sort_order = sort_order + 1 WHERE id = ?1",
            [other_id],
        )?;
        other_sort
    } else {
        let max: i32 = conn.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM category WHERE parent_id = ?1",
            [&parent_id],
            |r| r.get(0),
        )?;
        max + 1
    };
    conn.execute(
        "INSERT INTO category (id, parent_id, name, preset_key, sort_order, color_hex)
         VALUES (?1, ?2, NULL, 'preset.category.finance.loan', ?3, NULL)",
        params![Uuid::new_v4().to_string(), parent_id, sort_order],
    )?;
    Ok(())
}

fn add_column_if_missing(conn: &Connection, table: &str, name: &str, decl: &str) -> Result<()> {
    if column_exists(conn, table, name)? {
        return Ok(());
    }
    conn.execute(
        &format!("ALTER TABLE {table} ADD COLUMN {name} {decl}"),
        [],
    )?;
    Ok(())
}

fn column_exists(conn: &Connection, table: &str, name: &str) -> Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(1))?;
    for row in rows {
        if row? == name {
            return Ok(true);
        }
    }
    Ok(false)
}
