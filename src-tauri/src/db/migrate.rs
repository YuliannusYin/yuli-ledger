use rusqlite::{params, Connection};

use crate::db::schema::SCHEMA_VERSION;
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
    conn.execute(
        "UPDATE ledger_settings SET schema_version = ?1 WHERE id = 1",
        params![SCHEMA_VERSION],
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
