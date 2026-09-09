use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, TimeZone, Timelike, Utc};
use chrono::offset::LocalResult;

use crate::error::{AppError, Result};

pub const OPENING_EPOCH: &str = "1970-01-01T00:00:00Z";

pub fn now_utc_minute() -> String {
    format_utc_minute(Utc::now())
}

pub fn format_utc_minute(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:00Z").to_string()
}

pub fn parse_utc_minute(s: &str) -> Result<DateTime<Utc>> {
    let dt = DateTime::parse_from_rfc3339(s)
        .map_err(|_| AppError::new("error.timeInvalid"))?
        .with_timezone(&Utc);
    let reset = dt
        .with_second(0)
        .and_then(|d| d.with_nanosecond(0))
        .ok_or_else(|| AppError::new("error.timeInvalid"))?;
    Ok(reset)
}

pub fn parse_local_date(s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| AppError::new("error.dateInvalid"))
}

pub fn format_local_date(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

pub fn utc_to_local_date(utc: DateTime<Utc>) -> NaiveDate {
    utc.with_timezone(&Local).date_naive()
}

pub fn occurred_local_date(occurred_at: &str) -> Result<NaiveDate> {
    Ok(utc_to_local_date(parse_utc_minute(occurred_at)?))
}

fn naive_local_to_utc(naive: NaiveDateTime) -> DateTime<Utc> {
    match Local.from_local_datetime(&naive) {
        LocalResult::Single(dt) => dt.with_timezone(&Utc),
        LocalResult::Ambiguous(a, _) => a.with_timezone(&Utc),
        LocalResult::None => Local.from_utc_datetime(&naive).with_timezone(&Utc),
    }
}

pub fn local_date_start_utc(date: NaiveDate) -> DateTime<Utc> {
    naive_local_to_utc(date.and_hms_opt(0, 0, 0).unwrap())
}

pub fn local_date_end_utc(date: NaiveDate) -> DateTime<Utc> {
    naive_local_to_utc(date.and_hms_opt(23, 59, 59).unwrap())
}

pub fn week_bounds(anchor: NaiveDate) -> (NaiveDate, NaiveDate) {
    let offset = anchor.weekday().num_days_from_monday() as i64;
    let start = anchor - Duration::days(offset);
    let end = start + Duration::days(6);
    (start, end)
}

pub fn month_bounds(anchor: NaiveDate) -> (NaiveDate, NaiveDate) {
    let start = NaiveDate::from_ymd_opt(anchor.year(), anchor.month(), 1).unwrap();
    let end = if anchor.month() == 12 {
        NaiveDate::from_ymd_opt(anchor.year() + 1, 1, 1).unwrap() - Duration::days(1)
    } else {
        NaiveDate::from_ymd_opt(anchor.year(), anchor.month() + 1, 1).unwrap() - Duration::days(1)
    };
    (start, end)
}

pub fn year_bounds(anchor: NaiveDate) -> (NaiveDate, NaiveDate) {
    let start = NaiveDate::from_ymd_opt(anchor.year(), 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(anchor.year(), 12, 31).unwrap();
    (start, end)
}

pub fn shift_month(anchor: NaiveDate, delta: i32) -> NaiveDate {
    let mut year = anchor.year();
    let mut month = anchor.month() as i32 + delta;
    while month < 1 {
        month += 12;
        year -= 1;
    }
    while month > 12 {
        month -= 12;
        year += 1;
    }
    let last = month_bounds(NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap()).1.day();
    let day = anchor.day().min(last);
    NaiveDate::from_ymd_opt(year, month as u32, day).unwrap()
}

pub fn iso_week_number(date: NaiveDate) -> u32 {
    date.iso_week().week()
}


pub fn custom_range_ok(from: NaiveDate, to: NaiveDate) -> Result<()> {
    if from > to {
        return Err(AppError::new("error.invalidRange"));
    }
    let days = (to - from).num_days() + 1;
    if days > 3653 {
        return Err(AppError::new("error.rangeTooLong"));
    }
    Ok(())
}

pub fn in_local_range(occurred_at: &str, from: NaiveDate, to: NaiveDate) -> Result<bool> {
    let d = occurred_local_date(occurred_at)?;
    Ok(d >= from && d <= to)
}

/// Instant compare documented as `occurred_at >= opening_at`.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn week_starts_monday() {
        let wed = NaiveDate::from_ymd_opt(2026, 9, 9).unwrap();
        let (start, end) = week_bounds(wed);
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 9, 7).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 9, 13).unwrap());
        assert_eq!(start.weekday(), chrono::Weekday::Mon);
    }

    #[test]
    fn seconds_zeroed() {
        let dt = parse_utc_minute("2026-09-09T12:30:45Z").unwrap();
        assert_eq!(format_utc_minute(dt), "2026-09-09T12:30:00Z");
    }
}
