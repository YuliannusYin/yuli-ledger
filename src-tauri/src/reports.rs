use std::collections::BTreeMap;

use chrono::{Datelike, Duration, NaiveDate};

use crate::error::{AppError, Result};
use crate::kinds::registry::{self, DebtEffect, ReportBucket};
use crate::models::*;
use crate::time_util;

fn side_totals(entries: &[EntryRow]) -> (i64, i64) {
    let mut income = 0i64;
    let mut expense = 0i64;
    for e in entries {
        let (i, x) = registry::pnl_amounts(&e.kind_id, e.amount_minor, e.counter_amount_minor);
        income += i;
        expense += x;
    }
    (income, expense)
}

fn side_value(side: ReportBucket, income: i64, expense: i64) -> i64 {
    match side {
        ReportBucket::Income => income,
        ReportBucket::Expense => expense,
        ReportBucket::None => 0,
    }
}

fn parse_side(s: &str) -> Result<ReportBucket> {
    match s {
        "income" => Ok(ReportBucket::Income),
        "expense" => Ok(ReportBucket::Expense),
        _ => Err(AppError::new("error.reportSideInvalid")),
    }
}

fn period_bounds(query: &ReportQuery) -> Result<(NaiveDate, NaiveDate, Option<u32>)> {
    match query.mode.as_str() {
        "week" => {
            let anchor = time_util::parse_local_date(&query.anchor_date)?;
            let (from, to) = time_util::week_bounds(anchor);
            Ok((from, to, Some(time_util::iso_week_number(from))))
        }
        "month" => {
            let anchor = time_util::parse_local_date(&query.anchor_date)?;
            let (from, to) = time_util::month_bounds(anchor);
            Ok((from, to, None))
        }
        "year" => {
            let anchor = time_util::parse_local_date(&query.anchor_date)?;
            let (from, to) = time_util::year_bounds(anchor);
            Ok((from, to, None))
        }
        "custom" => {
            let from = time_util::parse_local_date(
                query
                    .custom_from
                    .as_deref()
                    .ok_or_else(|| AppError::new("error.invalidRange"))?,
            )?;
            let to = time_util::parse_local_date(
                query
                    .custom_to
                    .as_deref()
                    .ok_or_else(|| AppError::new("error.invalidRange"))?,
            )?;
            time_util::custom_range_ok(from, to)?;
            Ok((from, to, None))
        }
        _ => Err(AppError::new("error.reportModeInvalid")),
    }
}

fn previous_bounds(mode: &str, from: NaiveDate, to: NaiveDate) -> Option<(NaiveDate, NaiveDate)> {
    match mode {
        "week" => {
            let prev_end = from - Duration::days(1);
            let (a, b) = time_util::week_bounds(prev_end);
            Some((a, b))
        }
        "month" => {
            let prev = time_util::shift_month(from, -1);
            Some(time_util::month_bounds(prev))
        }
        "year" => {
            let prev = NaiveDate::from_ymd_opt(from.year() - 1, 1, 1)?;
            Some(time_util::year_bounds(prev))
        }
        _ => {
            let _ = to;
            None
        }
    }
}

fn filter_range<'a>(entries: &'a [EntryRow], from: NaiveDate, to: NaiveDate) -> Vec<&'a EntryRow> {
    entries
        .iter()
        .filter(|e| {
            time_util::in_local_range(&e.occurred_at, from, to).unwrap_or(false)
        })
        .collect()
}

fn add_named(map: &mut BTreeMap<String, i64>, id: &str, amount: i64) {
    *map.entry(id.to_string()).or_insert(0) += amount;
}

fn composition_maps(
    entries: &[&EntryRow],
    side: ReportBucket,
    categories: &[CategoryDto],
) -> (BTreeMap<String, i64>, BTreeMap<String, i64>, BTreeMap<String, i64>) {
    let parent_of: BTreeMap<String, Option<String>> = categories
        .iter()
        .map(|c| (c.id.clone(), c.parent_id.clone()))
        .collect();
    let color_of: BTreeMap<String, Option<String>> = categories
        .iter()
        .map(|c| (c.id.clone(), c.color_hex.clone()))
        .collect();

    let mut by_sub = BTreeMap::new();
    let mut by_main = BTreeMap::new();
    let mut by_account = BTreeMap::new();

    for e in entries {
        match side {
            ReportBucket::Expense => {
                if registry::report_bucket_or_none(&e.kind_id) == ReportBucket::Expense {
                    add_named(&mut by_sub, &e.category_id, e.amount_minor);
                    add_named(&mut by_account, &e.account_id, e.amount_minor);
                    if let Some(Some(main)) = parent_of.get(&e.category_id) {
                        add_named(&mut by_main, main, e.amount_minor);
                    }
                }
                if registry::fee_report_bucket_or_none(&e.kind_id)
                    == registry::FeeReportBucket::Expense
                {
                    if let Some(counter) = e.counter_amount_minor {
                        let fee = registry::transfer_fee(e.amount_minor, counter);
                        if fee > 0 {
                            if let Some(fee_cat) = &e.fee_category_id {
                                add_named(&mut by_sub, fee_cat, fee);
                                if let Some(Some(main)) = parent_of.get(fee_cat) {
                                    add_named(&mut by_main, main, fee);
                                }
                            }
                            add_named(&mut by_account, &e.account_id, fee);
                        }
                    }
                }
            }
            ReportBucket::Income => {
                if registry::report_bucket_or_none(&e.kind_id) == ReportBucket::Income {
                    add_named(&mut by_sub, &e.category_id, e.amount_minor);
                    add_named(&mut by_account, &e.account_id, e.amount_minor);
                    if let Some(Some(main)) = parent_of.get(&e.category_id) {
                        add_named(&mut by_main, main, e.amount_minor);
                    }
                }
            }
            ReportBucket::None => {}
        }
    }
    let _ = color_of;
    (by_main, by_sub, by_account)
}

fn named_list(
    map: BTreeMap<String, i64>,
    total: i64,
    categories: &[CategoryDto],
) -> Vec<NamedAmount> {
    let color_of: BTreeMap<_, _> = categories
        .iter()
        .map(|c| {
            let color = c.color_hex.clone().or_else(|| {
                categories
                    .iter()
                    .find(|p| Some(&p.id) == c.parent_id.as_ref())
                    .and_then(|p| p.color_hex.clone())
            });
            (c.id.clone(), color)
        })
        .collect();
    let mut items: Vec<(String, i64)> = map.into_iter().filter(|(_, a)| *a > 0).collect();
    items.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let mut out = Vec::new();
    let mut assigned = 0i32;
    for (i, (id, amount)) in items.iter().enumerate() {
        let percent_bp = if total <= 0 {
            0
        } else if i + 1 == items.len() {
            10000 - assigned
        } else {
            let p = ((*amount as i128) * 10000 / total as i128) as i32;
            assigned += p;
            p
        };
        out.push(NamedAmount {
            id: id.clone(),
            amount_minor: *amount,
            color_hex: color_of.get(id).cloned().flatten(),
            percent_bp,
        });
    }
    out
}

fn trend_points(mode: &str, from: NaiveDate, to: NaiveDate, entries: &[&EntryRow], side: ReportBucket) -> Vec<TrendPoint> {
    let mut keys: Vec<NaiveDate> = Vec::new();
    let daily = match mode {
        "week" | "month" => true,
        "year" => false,
        "custom" => (to - from).num_days() + 1 <= 31,
        _ => true,
    };
    if daily {
        let mut d = from;
        while d <= to {
            keys.push(d);
            d += Duration::days(1);
        }
    } else if mode == "year" {
        for m in 1..=12 {
            keys.push(NaiveDate::from_ymd_opt(from.year(), m, 1).unwrap());
        }
    } else {
        let mut cursor = NaiveDate::from_ymd_opt(from.year(), from.month(), 1).unwrap();
        let end = NaiveDate::from_ymd_opt(to.year(), to.month(), 1).unwrap();
        while cursor <= end {
            keys.push(cursor);
            cursor = time_util::shift_month(cursor, 1);
            cursor = NaiveDate::from_ymd_opt(cursor.year(), cursor.month(), 1).unwrap();
        }
    }

    keys.into_iter()
        .map(|key| {
            let (bucket_from, bucket_to) = if daily {
                (key, key)
            } else {
                time_util::month_bounds(key)
            };
            let slice: Vec<&EntryRow> = entries
                .iter()
                .copied()
                .filter(|e| {
                    time_util::in_local_range(&e.occurred_at, bucket_from, bucket_to).unwrap_or(false)
                })
                .collect();
            let owned: Vec<EntryRow> = slice.into_iter().cloned().collect();
            let (income, expense) = side_totals(&owned);
            TrendPoint {
                key: time_util::format_local_date(key),
                amount_minor: side_value(side, income, expense),
            }
        })
        .collect()
}

fn comparison_bars(mode: &str, from: NaiveDate, all: &[EntryRow], side: ReportBucket) -> Option<Vec<ComparisonBar>> {
    let mut periods = Vec::new();
    match mode {
        "week" => {
            for i in (0..8).rev() {
                let start = from - Duration::days(7 * i);
                let end = start + Duration::days(6);
                periods.push((start, end, i == 0));
            }
        }
        "month" => {
            for i in (0..8).rev() {
                let anchor = time_util::shift_month(from, -i);
                let (a, b) = time_util::month_bounds(anchor);
                periods.push((a, b, i == 0));
            }
        }
        "year" => {
            for i in (0..8).rev() {
                let year = from.year() - i;
                let anchor = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
                let (a, b) = time_util::year_bounds(anchor);
                periods.push((a, b, i == 0));
            }
        }
        _ => return None,
    }
    Some(
        periods
            .into_iter()
            .map(|(a, b, is_current)| {
                let slice: Vec<EntryRow> = filter_range(all, a, b).into_iter().cloned().collect();
                let (income, expense) = side_totals(&slice);
                ComparisonBar {
                    key: time_util::format_local_date(a),
                    amount_minor: side_value(side, income, expense),
                    is_current,
                }
            })
            .collect(),
    )
}

fn ranking(entries: &[&EntryRow], side: ReportBucket) -> Vec<RankingRow> {
    let mut rows: Vec<RankingRow> = Vec::new();
    for e in entries {
        if let Some(amount) =
            registry::ranking_amount(&e.kind_id, side, e.amount_minor, e.counter_amount_minor)
        {
            let category_id = if registry::fee_report_bucket_or_none(&e.kind_id)
                == registry::FeeReportBucket::Expense
                && registry::report_bucket_or_none(&e.kind_id) != ReportBucket::Expense
            {
                e.fee_category_id.clone().unwrap_or_else(|| e.category_id.clone())
            } else {
                e.category_id.clone()
            };
            rows.push(RankingRow {
                entry_id: e.id.clone(),
                occurred_at: e.occurred_at.clone(),
                kind_id: e.kind_id.clone(),
                category_id,
                note: e.note.clone(),
                amount_minor: amount,
            });
        }
    }
    rows.sort_by(|a, b| {
        b.amount_minor
            .cmp(&a.amount_minor)
            .then(b.occurred_at.cmp(&a.occurred_at))
    });
    rows.truncate(20);
    rows
}

fn secondary(entries: &[&EntryRow]) -> (i64, i64, i64, i64, i64) {
    let mut repayment = 0i64;
    let mut loan = 0i64;
    let mut volume = 0i64;
    let mut fees = 0i64;
    for e in entries {
        match registry::debt_counter_or_none(&e.kind_id) {
            DebtEffect::Decrease => repayment += e.amount_minor,
            DebtEffect::Increase => loan += e.amount_minor,
            DebtEffect::None => {}
        }
        if registry::effect_or_none(&e.kind_id) == registry::BalanceEffect::Transfer {
            if let Some(c) = e.counter_amount_minor {
                volume += c;
                let fee = registry::transfer_fee(e.amount_minor, c);
                if fee > 0 {
                    fees += fee;
                }
            }
        }
    }
    (repayment, 0, loan, volume, fees)
}

pub fn build_report(
    all: &[EntryRow],
    categories: &[CategoryDto],
    query: &ReportQuery,
) -> Result<ReportDto> {
    let side = parse_side(&query.side)?;
    let (from, to, iso_week) = period_bounds(query)?;
    let current = filter_range(all, from, to);
    let owned: Vec<EntryRow> = current.iter().copied().cloned().collect();
    let (income_total, expense_total) = side_totals(&owned);
    let side_total = side_value(side, income_total, expense_total);
    let net = income_total - expense_total;
    let days = (to - from).num_days() + 1;
    let average = if query.mode == "year" {
        side_total / 12
    } else if days > 0 {
        side_total / days
    } else {
        0
    };

    let (previous_total, delta) = if let Some((pf, pt)) = previous_bounds(&query.mode, from, to) {
        let prev_owned: Vec<EntryRow> = filter_range(all, pf, pt).into_iter().cloned().collect();
        let (pi, px) = side_totals(&prev_owned);
        let prev = side_value(side, pi, px);
        (Some(prev), Some(side_total - prev))
    } else {
        (None, None)
    };

    let (by_main, by_sub, by_account) = composition_maps(&current, side, categories);
    let (
        secondary_repayment,
        secondary_prepayment,
        secondary_loan,
        secondary_transfer_volume,
        secondary_transfer_fees,
    ) = secondary(&current);

    Ok(ReportDto {
        range_from: time_util::format_local_date(from),
        range_to: time_util::format_local_date(to),
        iso_week,
        side_total,
        income_total,
        expense_total,
        net,
        average,
        previous_total,
        delta,
        secondary_repayment,
        secondary_prepayment,
        secondary_loan,
        secondary_transfer_volume,
        secondary_transfer_fees,
        trend: trend_points(&query.mode, from, to, &current, side),
        composition_main: named_list(by_main, side_total, categories),
        composition_sub: named_list(by_sub, side_total, categories),
        by_account: named_list(by_account, side_total, categories),
        comparison: comparison_bars(&query.mode, from, all, side),
        ranking: ranking(&current, side),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(kind: &str, amount: i64, counter: Option<i64>, occurred: &str, cat: &str, fee_cat: Option<&str>) -> EntryRow {
        EntryRow {
            id: occurred.to_string() + kind,
            kind_id: kind.into(),
            amount_minor: amount,
            occurred_at: occurred.into(),
            account_id: "a".into(),
            counter_account_id: counter.map(|_| "b".into()),
            counter_amount_minor: counter,
            category_id: cat.into(),
            fee_category_id: fee_cat.map(|s| s.into()),
            note: None,
            kind_payload: r#"{"v":1}"#.into(),
            created_at: occurred.into(),
            updated_at: occurred.into(),
        }
    }

    #[test]
    fn repayment_not_in_expense_transfer_fee_is() {
        let (inc, exp) = registry::pnl_amounts("repayment", 5000, None);
        assert_eq!(inc, 0);
        assert_eq!(exp, 0);
        let (inc, exp) = registry::pnl_amounts("transfer", 50000, Some(49900));
        assert_eq!(inc, 0);
        assert_eq!(exp, 100);
        let (inc, exp) = registry::pnl_amounts("expense", 200, None);
        assert_eq!(inc, 0);
        assert_eq!(exp, 200);
        let (inc, exp) = registry::pnl_amounts("prepayment", 300, None);
        assert_eq!(inc, 0);
        assert_eq!(exp, 300);
        let (inc, exp) = registry::pnl_amounts("mystery", 9, None);
        assert_eq!(inc, 0);
        assert_eq!(exp, 0);
        let (inc, exp) = registry::pnl_amounts("loan", 8000, None);
        assert_eq!(inc, 0);
        assert_eq!(exp, 0);
        let _ = row("expense", 1, None, "2026-01-01T00:00:00Z", "c", None);
    }
}
