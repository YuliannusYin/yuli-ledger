use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BalanceEffect {
    Increase,
    Decrease,
    Transfer,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReportBucket {
    Income,
    Expense,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FeeReportBucket {
    Expense,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DebtEffect {
    Increase,
    Decrease,
    None,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KindDescriptor {
    pub id: &'static str,
    pub label_key: &'static str,
    pub hint_key: Option<&'static str>,
    pub payload_schema_version: i32,
    pub balance_effect: BalanceEffect,
    pub report_bucket: ReportBucket,
    pub fee_report_bucket: FeeReportBucket,
    pub debt_effect_primary: DebtEffect,
    pub debt_effect_counter: DebtEffect,
    pub category_required: bool,
    pub counter_account_required: bool,
    pub counter_amount_required: bool,
    pub counter_accounts_must_differ: bool,
    pub primary_account_label_key: Option<&'static str>,
    pub counter_account_label_key: Option<&'static str>,
    pub implemented: bool,
}

pub const EXPENSE: KindDescriptor = KindDescriptor {
    id: "expense",
    label_key: "kind.expense",
    hint_key: None,
    payload_schema_version: 1,
    balance_effect: BalanceEffect::Decrease,
    report_bucket: ReportBucket::Expense,
    fee_report_bucket: FeeReportBucket::None,
    debt_effect_primary: DebtEffect::None,
    debt_effect_counter: DebtEffect::None,
    category_required: true,
    counter_account_required: false,
    counter_amount_required: false,
    counter_accounts_must_differ: false,
    primary_account_label_key: None,
    counter_account_label_key: None,
    implemented: true,
};

pub const INCOME: KindDescriptor = KindDescriptor {
    id: "income",
    label_key: "kind.income",
    hint_key: None,
    payload_schema_version: 1,
    balance_effect: BalanceEffect::Increase,
    report_bucket: ReportBucket::Income,
    fee_report_bucket: FeeReportBucket::None,
    debt_effect_primary: DebtEffect::None,
    debt_effect_counter: DebtEffect::None,
    category_required: true,
    counter_account_required: false,
    counter_amount_required: false,
    counter_accounts_must_differ: false,
    primary_account_label_key: None,
    counter_account_label_key: None,
    implemented: true,
};

pub const PREPAYMENT: KindDescriptor = KindDescriptor {
    id: "prepayment",
    label_key: "kind.prepayment",
    hint_key: Some("kind.prepayment.hint"),
    payload_schema_version: 1,
    balance_effect: BalanceEffect::None,
    report_bucket: ReportBucket::Expense,
    fee_report_bucket: FeeReportBucket::None,
    debt_effect_primary: DebtEffect::Increase,
    debt_effect_counter: DebtEffect::None,
    category_required: true,
    counter_account_required: false,
    counter_amount_required: false,
    counter_accounts_must_differ: false,
    primary_account_label_key: None,
    counter_account_label_key: None,
    implemented: true,
};

pub const REPAYMENT: KindDescriptor = KindDescriptor {
    id: "repayment",
    label_key: "kind.repayment",
    hint_key: Some("kind.repayment.hint"),
    payload_schema_version: 1,
    balance_effect: BalanceEffect::Decrease,
    report_bucket: ReportBucket::None,
    fee_report_bucket: FeeReportBucket::None,
    debt_effect_primary: DebtEffect::None,
    debt_effect_counter: DebtEffect::Decrease,
    category_required: true,
    counter_account_required: true,
    counter_amount_required: false,
    counter_accounts_must_differ: true,
    primary_account_label_key: Some("field.repayFromAccount"),
    counter_account_label_key: Some("field.repaidAccount"),
    implemented: true,
};

pub const LOAN: KindDescriptor = KindDescriptor {
    id: "loan",
    label_key: "kind.loan",
    hint_key: Some("kind.loan.hint"),
    payload_schema_version: 1,
    balance_effect: BalanceEffect::Increase,
    report_bucket: ReportBucket::None,
    fee_report_bucket: FeeReportBucket::None,
    debt_effect_primary: DebtEffect::None,
    debt_effect_counter: DebtEffect::Increase,
    category_required: true,
    counter_account_required: true,
    counter_amount_required: false,
    counter_accounts_must_differ: false,
    primary_account_label_key: Some("field.loanToAccount"),
    counter_account_label_key: Some("field.loanDebtAccount"),
    implemented: true,
};

pub const TRANSFER: KindDescriptor = KindDescriptor {
    id: "transfer",
    label_key: "kind.transfer",
    hint_key: Some("kind.transfer.hint"),
    payload_schema_version: 1,
    balance_effect: BalanceEffect::Transfer,
    report_bucket: ReportBucket::None,
    fee_report_bucket: FeeReportBucket::Expense,
    debt_effect_primary: DebtEffect::None,
    debt_effect_counter: DebtEffect::None,
    category_required: true,
    counter_account_required: true,
    counter_amount_required: true,
    counter_accounts_must_differ: true,
    primary_account_label_key: None,
    counter_account_label_key: Some("field.destAccount"),
    implemented: true,
};

pub const ALL: &[KindDescriptor] = &[EXPENSE, INCOME, PREPAYMENT, REPAYMENT, LOAN, TRANSFER];

pub fn get(id: &str) -> Option<&'static KindDescriptor> {
    ALL.iter().find(|k| k.id == id)
}

pub fn implemented() -> Vec<KindDescriptor> {
    ALL.iter().copied().filter(|k| k.implemented).collect()
}

/// Missing registry entries behave as none / none / none.
pub fn effect_or_none(id: &str) -> BalanceEffect {
    get(id).map(|k| k.balance_effect).unwrap_or(BalanceEffect::None)
}

pub fn report_bucket_or_none(id: &str) -> ReportBucket {
    get(id).map(|k| k.report_bucket).unwrap_or(ReportBucket::None)
}

pub fn fee_report_bucket_or_none(id: &str) -> FeeReportBucket {
    get(id)
        .map(|k| k.fee_report_bucket)
        .unwrap_or(FeeReportBucket::None)
}

pub fn debt_primary_or_none(id: &str) -> DebtEffect {
    get(id)
        .map(|k| k.debt_effect_primary)
        .unwrap_or(DebtEffect::None)
}

pub fn debt_counter_or_none(id: &str) -> DebtEffect {
    get(id)
        .map(|k| k.debt_effect_counter)
        .unwrap_or(DebtEffect::None)
}

pub fn transfer_fee(amount_minor: i64, counter_amount_minor: i64) -> i64 {
    amount_minor - counter_amount_minor
}

pub fn pnl_amounts(kind_id: &str, amount_minor: i64, counter_amount_minor: Option<i64>) -> (i64, i64) {
    let mut income = 0i64;
    let mut expense = 0i64;
    match report_bucket_or_none(kind_id) {
        ReportBucket::Income => income += amount_minor,
        ReportBucket::Expense => expense += amount_minor,
        ReportBucket::None => {}
    }
    if fee_report_bucket_or_none(kind_id) == FeeReportBucket::Expense {
        if let Some(counter) = counter_amount_minor {
            let fee = transfer_fee(amount_minor, counter);
            if fee > 0 {
                expense += fee;
            }
        }
    }
    (income, expense)
}

pub fn ranking_amount(kind_id: &str, side: ReportBucket, amount_minor: i64, counter_amount_minor: Option<i64>) -> Option<i64> {
    match side {
        ReportBucket::Expense => match report_bucket_or_none(kind_id) {
            ReportBucket::Expense => Some(amount_minor),
            _ if fee_report_bucket_or_none(kind_id) == FeeReportBucket::Expense => {
                let fee = counter_amount_minor
                    .map(|c| transfer_fee(amount_minor, c))
                    .unwrap_or(0);
                if fee > 0 {
                    Some(fee)
                } else {
                    None
                }
            }
            _ => None,
        },
        ReportBucket::Income => {
            if report_bucket_or_none(kind_id) == ReportBucket::Income {
                Some(amount_minor)
            } else {
                None
            }
        }
        ReportBucket::None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implemented_order_is_expense_first() {
        let ids: Vec<&str> = implemented().iter().map(|k| k.id).collect();
        assert_eq!(
            ids,
            ["expense", "income", "prepayment", "repayment", "loan", "transfer"]
        );
    }

    #[test]
    fn prepayment_counts_as_expense_not_balance() {
        assert_eq!(PREPAYMENT.report_bucket, ReportBucket::Expense);
        assert_eq!(PREPAYMENT.balance_effect, BalanceEffect::None);
        assert_eq!(PREPAYMENT.debt_effect_primary, DebtEffect::Increase);
        let (inc, exp) = pnl_amounts("prepayment", 100, None);
        assert_eq!((inc, exp), (0, 100));
    }

    #[test]
    fn repayment_has_counter_account_not_amount() {
        assert!(REPAYMENT.counter_account_required);
        assert!(!REPAYMENT.counter_amount_required);
        assert_eq!(REPAYMENT.debt_effect_counter, DebtEffect::Decrease);
        let (inc, exp) = pnl_amounts("repayment", 5000, None);
        assert_eq!((inc, exp), (0, 0));
    }

    #[test]
    fn loan_raises_balance_and_counter_debt_not_pnl() {
        assert!(LOAN.counter_account_required);
        assert!(!LOAN.counter_amount_required);
        assert_eq!(LOAN.balance_effect, BalanceEffect::Increase);
        assert_eq!(LOAN.debt_effect_counter, DebtEffect::Increase);
        assert_eq!(LOAN.report_bucket, ReportBucket::None);
        assert!(!LOAN.counter_accounts_must_differ);
        assert!(REPAYMENT.counter_accounts_must_differ);
        let (inc, exp) = pnl_amounts("loan", 8000, None);
        assert_eq!((inc, exp), (0, 0));
    }
}
