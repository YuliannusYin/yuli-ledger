use crate::kinds::registry::{self, BalanceEffect, DebtEffect};
use crate::models::EntryRow;

pub fn balance_for_account(
    opening_balance_minor: i64,
    opening_at: &str,
    account_id: &str,
    entries: &[EntryRow],
) -> i64 {
    let mut total = opening_balance_minor;
    for e in entries {
        if e.occurred_at.as_str() < opening_at {
            continue;
        }
        let effect = registry::effect_or_none(&e.kind_id);
        match effect {
            BalanceEffect::Increase if e.account_id == account_id => {
                total += e.amount_minor;
            }
            BalanceEffect::Decrease if e.account_id == account_id => {
                total -= e.amount_minor;
            }
            BalanceEffect::Transfer => {
                if e.account_id == account_id {
                    total -= e.amount_minor;
                }
                if e.counter_account_id.as_deref() == Some(account_id) {
                    total += e.counter_amount_minor.unwrap_or(0);
                }
            }
            BalanceEffect::None | BalanceEffect::Increase | BalanceEffect::Decrease => {}
        }
    }
    total
}

pub fn debt_for_account(
    opening_debt_minor: i64,
    opening_at: &str,
    account_id: &str,
    entries: &[EntryRow],
) -> i64 {
    let mut total = opening_debt_minor;
    for e in entries {
        if e.occurred_at.as_str() < opening_at {
            continue;
        }
        match registry::debt_primary_or_none(&e.kind_id) {
            DebtEffect::Increase if e.account_id == account_id => {
                total += e.amount_minor;
            }
            DebtEffect::Decrease if e.account_id == account_id => {
                total -= e.amount_minor;
            }
            _ => {}
        }
        match registry::debt_counter_or_none(&e.kind_id) {
            DebtEffect::Increase if e.counter_account_id.as_deref() == Some(account_id) => {
                total += e.amount_minor;
            }
            DebtEffect::Decrease if e.counter_account_id.as_deref() == Some(account_id) => {
                total -= e.amount_minor;
            }
            _ => {}
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(
        kind: &str,
        amount: i64,
        account: &str,
        counter: Option<(&str, i64)>,
        occurred: &str,
    ) -> EntryRow {
        EntryRow {
            id: "1".into(),
            kind_id: kind.into(),
            amount_minor: amount,
            occurred_at: occurred.into(),
            account_id: account.into(),
            counter_account_id: counter.map(|(id, _)| id.into()),
            counter_amount_minor: counter.map(|(_, a)| a),
            category_id: "c".into(),
            fee_category_id: None,
            note: None,
            kind_payload: r#"{"v":1}"#.into(),
            created_at: occurred.into(),
            updated_at: occurred.into(),
        }
    }

    #[test]
    fn income_expense_and_transfer() {
        let entries = vec![
            row("income", 10000, "a", None, "2026-01-02T00:00:00Z"),
            row("expense", 3000, "a", None, "2026-01-03T00:00:00Z"),
            row(
                "transfer",
                5000,
                "a",
                Some(("b", 4900)),
                "2026-01-04T00:00:00Z",
            ),
            row("repayment", 1000, "a", None, "2026-01-05T00:00:00Z"),
            row("prepayment", 2000, "a", None, "2026-01-06T00:00:00Z"),
        ];
        let a = balance_for_account(1000, "1970-01-01T00:00:00Z", "a", &entries);
        let b = balance_for_account(0, "1970-01-01T00:00:00Z", "b", &entries);
        assert_eq!(a, 1000 + 10000 - 3000 - 5000 - 1000);
        assert_eq!(b, 4900);
    }

    #[test]
    fn debt_is_independent_of_balance() {
        let entries = vec![
            row("prepayment", 2000, "a", None, "2026-01-02T00:00:00Z"),
            row("repayment", 700, "cash", Some(("a", 0)), "2026-01-03T00:00:00Z"),
            row("repayment", 300, "cash", None, "2026-01-04T00:00:00Z"),
            row("expense", 50, "a", None, "2026-01-05T00:00:00Z"),
        ];
        let a_bal = balance_for_account(5000, "1970-01-01T00:00:00Z", "a", &entries);
        let cash_bal = balance_for_account(8000, "1970-01-01T00:00:00Z", "cash", &entries);
        let a_debt = debt_for_account(1000, "1970-01-01T00:00:00Z", "a", &entries);
        let cash_debt = debt_for_account(0, "1970-01-01T00:00:00Z", "cash", &entries);
        assert_eq!(a_bal, 5000 - 50);
        assert_eq!(cash_bal, 8000 - 700 - 300);
        assert_eq!(a_debt, 1000 + 2000 - 700);
        assert_eq!(cash_debt, 0);
    }

    #[test]
    fn debt_opening_at_excludes_earlier() {
        let entries = vec![row("prepayment", 2000, "a", None, "2026-01-01T00:00:00Z")];
        let debt = debt_for_account(0, "2026-01-02T00:00:00Z", "a", &entries);
        assert_eq!(debt, 0);
    }

    #[test]
    fn opening_at_excludes_earlier() {
        let entries = vec![row("income", 10000, "a", None, "2026-01-01T00:00:00Z")];
        let bal = balance_for_account(0, "2026-01-02T00:00:00Z", "a", &entries);
        assert_eq!(bal, 0);
    }

    #[test]
    fn unknown_kind_is_none() {
        let entries = vec![row("future_kind", 999, "a", None, "2026-01-01T00:00:00Z")];
        let bal = balance_for_account(5, "1970-01-01T00:00:00Z", "a", &entries);
        assert_eq!(bal, 5);
    }
}
