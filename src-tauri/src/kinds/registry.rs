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
    pub category_required: bool,
    pub counterparty_required: bool,
    pub implemented: bool,
}

pub const INCOME: KindDescriptor = KindDescriptor {
    id: "income",
    label_key: "kind.income",
    hint_key: None,
    payload_schema_version: 1,
    balance_effect: BalanceEffect::Increase,
    report_bucket: ReportBucket::Income,
    fee_report_bucket: FeeReportBucket::None,
    category_required: true,
    counterparty_required: false,
    implemented: true,
};

pub const EXPENSE: KindDescriptor = KindDescriptor {
    id: "expense",
    label_key: "kind.expense",
    hint_key: None,
    payload_schema_version: 1,
    balance_effect: BalanceEffect::Decrease,
    report_bucket: ReportBucket::Expense,
    fee_report_bucket: FeeReportBucket::None,
    category_required: true,
    counterparty_required: false,
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
    category_required: true,
    counterparty_required: false,
    implemented: true,
};

pub const PREPAYMENT: KindDescriptor = KindDescriptor {
    id: "prepayment",
    label_key: "kind.prepayment",
    hint_key: Some("kind.prepayment.hint"),
    payload_schema_version: 1,
    balance_effect: BalanceEffect::Decrease,
    report_bucket: ReportBucket::None,
    fee_report_bucket: FeeReportBucket::None,
    category_required: true,
    counterparty_required: false,
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
    category_required: true,
    counterparty_required: true,
    implemented: true,
};

pub const ALL: &[KindDescriptor] = &[INCOME, EXPENSE, REPAYMENT, PREPAYMENT, TRANSFER];

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
