use std::collections::HashMap;
use std::fs;
use std::path::Path;

use chrono::{SecondsFormat, Utc};

use crate::db;
use crate::error::{AppError, Result};
use crate::models::*;
use crate::money::format_minor_plain;
use crate::time_util;

const CSV_HEADERS: &[&str] = &[
    "csv.occurredAt",
    "csv.kindId",
    "csv.kind",
    "csv.amount",
    "csv.counterAmount",
    "csv.account",
    "csv.counterAccount",
    "csv.mainCategory",
    "csv.subCategory",
    "csv.feeCategory",
    "csv.tags",
    "csv.note",
    "csv.id",
];

const TXT_HEADERS: &[&str] = &[
    "csv.occurredAt",
    "csv.kind",
    "csv.amount",
    "csv.counterAmount",
    "csv.account",
    "csv.counterAccount",
    "csv.mainCategory",
    "csv.subCategory",
    "csv.feeCategory",
    "csv.tags",
    "csv.note",
];

pub fn write_entries_csv(
    conn: &rusqlite::Connection,
    path: &str,
    from_date: Option<String>,
    to_date: Option<String>,
    labels: &HashMap<String, String>,
) -> Result<()> {
    let body = build_entries_csv(conn, from_date, to_date, labels)?;
    write_path(path, body.as_bytes())
}

pub fn write_entries_txt(
    conn: &rusqlite::Connection,
    path: &str,
    from_date: Option<String>,
    to_date: Option<String>,
    labels: &HashMap<String, String>,
) -> Result<()> {
    let body = build_entries_txt(conn, from_date, to_date, labels)?;
    write_path(path, body.as_bytes())
}

pub fn write_backup_json(conn: &rusqlite::Connection, path: &str) -> Result<()> {
    let body = build_backup_json(conn)?;
    write_path(path, body.as_bytes())
}

pub(crate) fn write_path(path: &str, bytes: &[u8]) -> Result<()> {
    let path = path.trim();
    if path.is_empty() {
        return Err(AppError::new("error.io"));
    }
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, bytes)?;
    Ok(())
}

pub fn build_entries_csv(
    conn: &rusqlite::Connection,
    from_date: Option<String>,
    to_date: Option<String>,
    labels: &HashMap<String, String>,
) -> Result<String> {
    let lines = collect_entry_lines(conn, from_date, to_date, labels)?;
    let mut out = String::from("\u{FEFF}");
    out.push_str(
        &CSV_HEADERS
            .iter()
            .map(|k| csv_field(label(labels, k, k)))
            .collect::<Vec<_>>()
            .join(","),
    );
    out.push('\n');
    for line in lines {
        let row = [
            csv_field(&line.occurred_at),
            csv_field(&line.kind_id),
            csv_field(&line.kind),
            csv_field(&line.amount),
            csv_field(&line.counter_amount),
            csv_field(&line.account),
            csv_field(&line.counter_account),
            csv_field(&line.main),
            csv_field(&line.sub),
            csv_field(&line.fee),
            csv_field(&line.tags),
            csv_field(&line.note),
            csv_field(&line.id),
        ];
        out.push_str(&row.join(","));
        out.push('\n');
    }
    Ok(out)
}

pub fn build_entries_txt(
    conn: &rusqlite::Connection,
    from_date: Option<String>,
    to_date: Option<String>,
    labels: &HashMap<String, String>,
) -> Result<String> {
    let lines = collect_entry_lines(conn, from_date, to_date, labels)?;
    let mut out = String::from("\u{FEFF}");
    out.push_str(
        &TXT_HEADERS
            .iter()
            .map(|k| txt_field(label(labels, k, k)))
            .collect::<Vec<_>>()
            .join("\t"),
    );
    out.push('\n');
    for line in lines {
        let row = [
            txt_field(&line.occurred_at),
            txt_field(&line.kind),
            txt_field(&line.amount),
            txt_field(&line.counter_amount),
            txt_field(&line.account),
            txt_field(&line.counter_account),
            txt_field(&line.main),
            txt_field(&line.sub),
            txt_field(&line.fee),
            txt_field(&line.tags),
            txt_field(&line.note),
        ];
        out.push_str(&row.join("\t"));
        out.push('\n');
    }
    Ok(out)
}

struct EntryLine {
    occurred_at: String,
    kind_id: String,
    kind: String,
    amount: String,
    counter_amount: String,
    account: String,
    counter_account: String,
    main: String,
    sub: String,
    fee: String,
    tags: String,
    note: String,
    id: String,
}

fn collect_entry_lines(
    conn: &rusqlite::Connection,
    from_date: Option<String>,
    to_date: Option<String>,
    labels: &HashMap<String, String>,
) -> Result<Vec<EntryLine>> {
    let entries = db::list_entries(
        conn,
        &LedgerFilter {
            from_date,
            to_date,
            kind_ids: Vec::new(),
            account_ids: Vec::new(),
            category_id: None,
            tag_id: None,
            note_contains: None,
        },
    )?;
    let accounts = db::list_accounts(conn)?;
    let categories = db::list_categories(conn)?;
    let tags = db::list_tags(conn)?;
    let tag_name: HashMap<String, String> = tags.into_iter().map(|t| (t.id, t.name)).collect();
    let account_by: HashMap<String, &AccountDto> = accounts.iter().map(|a| (a.id.clone(), a)).collect();
    let cat_by: HashMap<String, &CategoryDto> = categories.iter().map(|c| (c.id.clone(), c)).collect();

    let mut lines = Vec::with_capacity(entries.len());
    for e in entries {
        let account = account_by.get(&e.account_id).copied();
        let counter = e
            .counter_account_id
            .as_ref()
            .and_then(|id| account_by.get(id).copied());
        let (main, sub) = category_pair(&cat_by, &e.category_id, labels);
        let fee = e
            .fee_category_id
            .as_ref()
            .map(|id| fee_label(&cat_by, id, labels))
            .unwrap_or_default();
        let tag_cell = e
            .tag_ids
            .iter()
            .filter_map(|id| tag_name.get(id).cloned())
            .collect::<Vec<_>>()
            .join(";");
        let kind_key = format!("kind.{}", e.kind_id);
        lines.push(EntryLine {
            occurred_at: time_util::format_local_datetime(&e.occurred_at)?,
            kind_id: e.kind_id.clone(),
            kind: label(labels, &kind_key, &e.kind_id).to_string(),
            amount: format_minor_plain(e.amount_minor),
            counter_amount: e
                .counter_amount_minor
                .map(format_minor_plain)
                .unwrap_or_default(),
            account: account_label(account, labels),
            counter_account: account_label(counter, labels),
            main,
            sub,
            fee,
            tags: tag_cell,
            note: e.note.unwrap_or_default(),
            id: e.id,
        });
    }
    Ok(lines)
}

pub fn build_backup_json(conn: &rusqlite::Connection) -> Result<String> {
    let settings = db::get_settings(conn)?;
    let backup = BackupFile {
        format: "yuli-ledger-backup".into(),
        version: 1,
        exported_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        schema_version: settings.schema_version,
        settings,
        accounts: db::list_accounts_including_deleted(conn)?,
        categories: db::list_categories_including_deleted(conn)?,
        tags: db::list_tags(conn)?,
        entries: db::list_entries_including_deleted(conn)?,
        pending_entries: db::list_pending_including_deleted(conn)?,
    };
    Ok(serde_json::to_string_pretty(&backup)?)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BackupFile {
    format: String,
    version: i32,
    exported_at: String,
    schema_version: i32,
    settings: SettingsDto,
    accounts: Vec<AccountDto>,
    categories: Vec<CategoryDto>,
    tags: Vec<TagDto>,
    entries: Vec<EntryDto>,
    pending_entries: Vec<PendingEntryDto>,
}

fn label<'a>(labels: &'a HashMap<String, String>, key: &str, fallback: &'a str) -> &'a str {
    labels.get(key).map(|s| s.as_str()).unwrap_or(fallback)
}

fn resolve_named(
    labels: &HashMap<String, String>,
    name: &Option<String>,
    preset: &Option<String>,
    id: &str,
) -> String {
    if let Some(n) = name {
        let t = n.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    if let Some(key) = preset {
        return label(labels, key, key).to_string();
    }
    id.to_string()
}

fn account_label(account: Option<&AccountDto>, labels: &HashMap<String, String>) -> String {
    match account {
        Some(a) => resolve_named(labels, &a.name, &a.preset_key, &a.id),
        None => String::new(),
    }
}

fn category_pair(
    cat_by: &HashMap<String, &CategoryDto>,
    id: &str,
    labels: &HashMap<String, String>,
) -> (String, String) {
    let Some(cat) = cat_by.get(id).copied() else {
        return (String::new(), String::new());
    };
    if let Some(pid) = &cat.parent_id {
        let main = cat_by
            .get(pid)
            .map(|m| resolve_named(labels, &m.name, &m.preset_key, &m.id))
            .unwrap_or_default();
        let sub = resolve_named(labels, &cat.name, &cat.preset_key, &cat.id);
        (main, sub)
    } else {
        (
            resolve_named(labels, &cat.name, &cat.preset_key, &cat.id),
            String::new(),
        )
    }
}

fn fee_label(
    cat_by: &HashMap<String, &CategoryDto>,
    id: &str,
    labels: &HashMap<String, String>,
) -> String {
    let (main, sub) = category_pair(cat_by, id, labels);
    if main.is_empty() {
        sub
    } else if sub.is_empty() {
        main
    } else {
        format!("{main} / {sub}")
    }
}

fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn txt_field(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, open_at};
    use crate::time_util::OPENING_EPOCH;

    fn temp_conn() -> (tempfile::TempDir, rusqlite::Connection) {
        let dir = tempfile::tempdir().unwrap();
        let conn = open_at(&dir.path().join("ledger.sqlite")).unwrap();
        (dir, conn)
    }

    fn labels() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("csv.occurredAt".into(), "Occurred at".into());
        m.insert("csv.kindId".into(), "Kind id".into());
        m.insert("csv.kind".into(), "Kind".into());
        m.insert("csv.amount".into(), "Amount".into());
        m.insert("csv.counterAmount".into(), "Counter amount".into());
        m.insert("csv.account".into(), "Account".into());
        m.insert("csv.counterAccount".into(), "Counter account".into());
        m.insert("csv.mainCategory".into(), "Main".into());
        m.insert("csv.subCategory".into(), "Sub".into());
        m.insert("csv.feeCategory".into(), "Fee".into());
        m.insert("csv.tags".into(), "Tags".into());
        m.insert("csv.note".into(), "Note".into());
        m.insert("csv.id".into(), "Id".into());
        m.insert("kind.expense".into(), "Expense".into());
        m.insert("preset.category.food".into(), "Food".into());
        m.insert("preset.category.food.dining".into(), "Dining out".into());
        m.insert("preset.account.default".into(), "Default".into());
        m
    }

    #[test]
    fn csv_includes_header_and_filtered_row() {
        let (_dir, mut conn) = temp_conn();
        let account = db::list_accounts(&conn).unwrap().pop().unwrap();
        let food: String = conn
            .query_row(
                "SELECT id FROM category WHERE preset_key = ?1",
                ["preset.category.food.dining"],
                |r| r.get(0),
            )
            .unwrap();
        db::create_entry(
            &mut conn,
            EntryWrite {
                kind_id: "expense".into(),
                amount_minor: 1250,
                occurred_at: "2026-09-09T12:00:00Z".into(),
                account_id: account.id.clone(),
                counter_account_id: None,
                counter_amount_minor: None,
                category_id: food,
                fee_category_id: None,
                note: Some("lunch, \"quoted\"".into()),
                tag_names: vec!["Work".into()],
                kind_payload: None,
            },
        )
        .unwrap();

        let csv = build_entries_csv(&conn, Some("2026-09-09".into()), Some("2026-09-09".into()), &labels())
            .unwrap();
        assert!(csv.starts_with('\u{FEFF}'));
        assert!(csv.contains("Occurred at"));
        assert!(csv.contains("Expense"));
        assert!(csv.contains("12.50"));
        assert!(csv.contains("\"lunch, \"\"quoted\"\"\""));
        assert!(csv.contains("Work"));
        assert!(csv.contains("Food"));
        assert!(csv.contains("Dining out"));

        let empty = build_entries_csv(&conn, Some("2026-01-01".into()), Some("2026-01-02".into()), &labels())
            .unwrap();
        assert_eq!(empty.lines().count(), 1);

        let txt = build_entries_txt(&conn, Some("2026-09-09".into()), Some("2026-09-09".into()), &labels())
            .unwrap();
        assert!(txt.starts_with('\u{FEFF}'));
        assert!(txt.contains("Occurred at"));
        assert!(txt.contains("Expense"));
        assert!(txt.contains("12.50"));
        assert!(txt.contains("lunch, \"quoted\""));
        assert!(!txt.contains("kindId"));
        assert_eq!(txt.lines().next().unwrap().matches('\t').count(), 10);

        let json = build_backup_json(&conn).unwrap();
        assert!(json.contains("yuli-ledger-backup"));
        assert!(json.contains("\"entries\""));
        assert!(json.contains(&account.id));

        let out = _dir.path().join("out.csv");
        write_entries_csv(
            &conn,
            out.to_str().unwrap(),
            None,
            None,
            &labels(),
        )
        .unwrap();
        assert!(out.exists());

        let txt_path = _dir.path().join("out.txt");
        write_entries_txt(
            &conn,
            txt_path.to_str().unwrap(),
            None,
            None,
            &labels(),
        )
        .unwrap();
        assert!(txt_path.exists());

        let _ = OPENING_EPOCH;
    }
}
