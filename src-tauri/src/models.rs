use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KindDto {
    pub id: String,
    pub label_key: String,
    pub hint_key: Option<String>,
    pub payload_schema_version: i32,
    pub balance_effect: String,
    pub report_bucket: String,
    pub fee_report_bucket: String,
    pub category_required: bool,
    pub counterparty_required: bool,
    pub implemented: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDto {
    pub id: String,
    pub name: Option<String>,
    pub account_kind: String,
    pub opening_balance_minor: i64,
    pub opening_at: String,
    pub note: Option<String>,
    pub sort_order: i32,
    pub preset_key: Option<String>,
    pub balance_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryDto {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: Option<String>,
    pub preset_key: Option<String>,
    pub sort_order: i32,
    pub color_hex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagDto {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    pub currency_code: String,
    pub default_account_id: String,
    pub schema_version: i32,
    pub ui_language: Option<String>,
    pub color_scheme: Option<String>,
    pub default_fee_category_id: Option<String>,
    pub report_mode: Option<String>,
    pub report_side: Option<String>,
    pub report_custom_from: Option<String>,
    pub report_custom_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryDto {
    pub id: String,
    pub kind_id: String,
    pub amount_minor: i64,
    pub occurred_at: String,
    pub account_id: String,
    pub counter_account_id: Option<String>,
    pub counter_amount_minor: Option<i64>,
    pub category_id: String,
    pub fee_category_id: Option<String>,
    pub note: Option<String>,
    pub kind_payload: String,
    pub created_at: String,
    pub updated_at: String,
    pub tag_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryWrite {
    pub kind_id: String,
    pub amount_minor: i64,
    pub occurred_at: String,
    pub account_id: String,
    pub counter_account_id: Option<String>,
    pub counter_amount_minor: Option<i64>,
    pub category_id: String,
    pub fee_category_id: Option<String>,
    pub note: Option<String>,
    pub tag_names: Vec<String>,
    #[serde(default)]
    pub kind_payload: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerFilter {
    #[serde(default)]
    pub from_date: Option<String>,
    #[serde(default)]
    pub to_date: Option<String>,
    #[serde(default)]
    pub kind_ids: Vec<String>,
    #[serde(default)]
    pub account_ids: Vec<String>,
    pub category_id: Option<String>,
    pub tag_id: Option<String>,
    pub note_contains: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountWrite {
    pub name: String,
    pub account_kind: String,
    pub opening_balance_minor: i64,
    pub opening_at: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportQuery {
    pub mode: String,
    pub side: String,
    pub anchor_date: String,
    pub custom_from: Option<String>,
    pub custom_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedAmount {
    pub id: String,
    pub amount_minor: i64,
    pub color_hex: Option<String>,
    pub percent_bp: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendPoint {
    pub key: String,
    pub amount_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonBar {
    pub key: String,
    pub amount_minor: i64,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankingRow {
    pub entry_id: String,
    pub occurred_at: String,
    pub kind_id: String,
    pub category_id: String,
    pub note: Option<String>,
    pub amount_minor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportDto {
    pub range_from: String,
    pub range_to: String,
    pub iso_week: Option<u32>,
    pub side_total: i64,
    pub income_total: i64,
    pub expense_total: i64,
    pub net: i64,
    pub average: i64,
    pub previous_total: Option<i64>,
    pub delta: Option<i64>,
    pub secondary_repayment: i64,
    pub secondary_prepayment: i64,
    pub secondary_transfer_volume: i64,
    pub secondary_transfer_fees: i64,
    pub trend: Vec<TrendPoint>,
    pub composition_main: Vec<NamedAmount>,
    pub composition_sub: Vec<NamedAmount>,
    pub by_account: Vec<NamedAmount>,
    pub comparison: Option<Vec<ComparisonBar>>,
    pub ranking: Vec<RankingRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapDto {
    pub settings: SettingsDto,
    pub accounts: Vec<AccountDto>,
    pub categories: Vec<CategoryDto>,
    pub tags: Vec<TagDto>,
    pub kinds: Vec<KindDto>,
    pub db_path: String,
    pub resolved_language: String,
    pub system_language: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EntryRow {
    pub id: String,
    pub kind_id: String,
    pub amount_minor: i64,
    pub occurred_at: String,
    pub account_id: String,
    pub counter_account_id: Option<String>,
    pub counter_amount_minor: Option<i64>,
    pub category_id: String,
    pub fee_category_id: Option<String>,
    pub note: Option<String>,
    pub kind_payload: String,
    pub created_at: String,
    pub updated_at: String,
}

impl KindDto {
    pub fn from_desc(d: &crate::kinds::registry::KindDescriptor) -> Self {
        Self {
            id: d.id.to_string(),
            label_key: d.label_key.to_string(),
            hint_key: d.hint_key.map(|s| s.to_string()),
            payload_schema_version: d.payload_schema_version,
            balance_effect: effect_name(d.balance_effect),
            report_bucket: bucket_name(d.report_bucket),
            fee_report_bucket: fee_bucket_name(d.fee_report_bucket),
            category_required: d.category_required,
            counterparty_required: d.counterparty_required,
            implemented: d.implemented,
        }
    }
}

fn effect_name(e: crate::kinds::registry::BalanceEffect) -> String {
    match e {
        crate::kinds::registry::BalanceEffect::Increase => "increase",
        crate::kinds::registry::BalanceEffect::Decrease => "decrease",
        crate::kinds::registry::BalanceEffect::Transfer => "transfer",
        crate::kinds::registry::BalanceEffect::None => "none",
    }
    .into()
}

fn bucket_name(e: crate::kinds::registry::ReportBucket) -> String {
    match e {
        crate::kinds::registry::ReportBucket::Income => "income",
        crate::kinds::registry::ReportBucket::Expense => "expense",
        crate::kinds::registry::ReportBucket::None => "none",
    }
    .into()
}

fn fee_bucket_name(e: crate::kinds::registry::FeeReportBucket) -> String {
    match e {
        crate::kinds::registry::FeeReportBucket::Expense => "expense",
        crate::kinds::registry::FeeReportBucket::None => "none",
    }
    .into()
}
