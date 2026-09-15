use std::fs;

use crate::db;
use crate::error::{AppError, Result};
use crate::export;
use crate::models::{ImportPendingResult, ImportRowError};
use crate::money;
use crate::time_util;

const TEMPLATE: &str = "\u{FEFF}occurredAt,amount\n2026-09-14 12:30,12.50\n";

pub fn write_pending_template(path: &str) -> Result<()> {
    export::write_path(path, TEMPLATE.as_bytes())
}

pub fn import_pending_csv(conn: &mut rusqlite::Connection, path: &str) -> Result<ImportPendingResult> {
    let path = path.trim();
    if path.is_empty() {
        return Err(AppError::new("error.io"));
    }
    let raw = fs::read(path)?;
    let text = decode_csv_bytes(&raw)?;
    parse_and_insert(conn, &text)
}

fn decode_csv_bytes(raw: &[u8]) -> Result<String> {
    let bytes = if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &raw[3..]
    } else {
        raw
    };
    String::from_utf8(bytes.to_vec()).map_err(|_| AppError::new("error.io"))
}

fn parse_and_insert(conn: &mut rusqlite::Connection, text: &str) -> Result<ImportPendingResult> {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes());
    let headers = rdr
        .headers()
        .map_err(|_| AppError::new("error.importHeaders"))?
        .clone();
    let occurred_idx = find_column(&headers, "occurredat");
    let amount_idx = find_column(&headers, "amount");
    let (Some(occurred_idx), Some(amount_idx)) = (occurred_idx, amount_idx) else {
        return Err(AppError::new("error.importHeaders"));
    };

    let mut imported = 0i64;
    let mut errors = Vec::new();
    for (i, rec) in rdr.records().enumerate() {
        let line = (i as i32) + 2;
        let record = match rec {
            Ok(r) => r,
            Err(_) => {
                errors.push(ImportRowError {
                    line,
                    code: "error.io".into(),
                });
                continue;
            }
        };
        let occurred_raw = record.get(occurred_idx).unwrap_or("").trim();
        let amount_raw = record.get(amount_idx).unwrap_or("").trim();
        if occurred_raw.is_empty() && amount_raw.is_empty() {
            continue;
        }
        let occurred = match time_util::parse_local_datetime_to_utc_iso(occurred_raw) {
            Ok(v) => v,
            Err(_) => {
                errors.push(ImportRowError {
                    line,
                    code: "error.timeInvalid".into(),
                });
                continue;
            }
        };
        let amount = match money::parse_cny_to_minor(amount_raw) {
            Ok(v) if v > 0 => v,
            _ => {
                errors.push(ImportRowError {
                    line,
                    code: "error.amountInvalid".into(),
                });
                continue;
            }
        };
        db::insert_pending_row(conn, amount, &occurred)?;
        imported += 1;
    }
    Ok(ImportPendingResult { imported, errors })
}

fn find_column(headers: &csv::StringRecord, want: &str) -> Option<usize> {
    headers
        .iter()
        .position(|h| h.trim().eq_ignore_ascii_case(want))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::tempdir;

    #[test]
    fn imports_amount_and_time_only() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("ledger.sqlite");
        let mut conn = db::open_at(&db_path).unwrap();
        let csv_path = dir.path().join("in.csv");
        fs::write(
            &csv_path,
            "occurredAt,amount,note\n2026-09-14 12:30,12.50,ignored\n2026-09-14,0\nbad,x\n",
        )
        .unwrap();
        let result = import_pending_csv(&mut conn, csv_path.to_str().unwrap()).unwrap();
        assert_eq!(result.imported, 1);
        assert_eq!(result.errors.len(), 2);
        let pending = db::list_pending_entries(&conn).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].amount_minor, 1250);
        assert!(pending[0].kind_id.is_none());
        assert!(pending[0].account_id.is_none());
        assert_eq!(db::list_all_entry_rows(&conn).unwrap().len(), 0);
    }
}
