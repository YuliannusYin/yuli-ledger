pub const SCHEMA: &str = r#"
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS account (
  id TEXT PRIMARY KEY,
  name TEXT,
  account_kind TEXT NOT NULL,
  opening_balance_minor INTEGER NOT NULL,
  opening_debt_minor INTEGER NOT NULL DEFAULT 0,
  opening_at TEXT NOT NULL,
  note TEXT,
  sort_order INTEGER NOT NULL,
  preset_key TEXT
);

CREATE TABLE IF NOT EXISTS category (
  id TEXT PRIMARY KEY,
  parent_id TEXT REFERENCES category(id),
  name TEXT,
  preset_key TEXT,
  sort_order INTEGER NOT NULL,
  color_hex TEXT
);

CREATE TABLE IF NOT EXISTS tag (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_tag_name_lower ON tag (lower(name));

CREATE TABLE IF NOT EXISTS ledger_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  currency_code TEXT NOT NULL,
  default_account_id TEXT NOT NULL REFERENCES account(id),
  schema_version INTEGER NOT NULL,
  ui_language TEXT,
  color_scheme TEXT,
  default_fee_category_id TEXT REFERENCES category(id),
  report_mode TEXT,
  report_side TEXT,
  report_custom_from TEXT,
  report_custom_to TEXT,
  last_kind_id TEXT,
  last_account_id TEXT,
  last_counter_account_id TEXT,
  last_category_id TEXT,
  last_fee_category_id TEXT,
  last_occurred_at TEXT
);

CREATE TABLE IF NOT EXISTS entry (
  id TEXT PRIMARY KEY,
  kind_id TEXT NOT NULL,
  amount_minor INTEGER NOT NULL,
  occurred_at TEXT NOT NULL,
  account_id TEXT NOT NULL REFERENCES account(id),
  counter_account_id TEXT REFERENCES account(id),
  counter_amount_minor INTEGER,
  category_id TEXT NOT NULL REFERENCES category(id),
  fee_category_id TEXT REFERENCES category(id),
  note TEXT,
  kind_payload TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS entry_tag (
  entry_id TEXT NOT NULL REFERENCES entry(id) ON DELETE CASCADE,
  tag_id TEXT NOT NULL REFERENCES tag(id) ON DELETE CASCADE,
  PRIMARY KEY (entry_id, tag_id)
);

CREATE INDEX IF NOT EXISTS idx_entry_occurred_at ON entry (occurred_at);
CREATE INDEX IF NOT EXISTS idx_entry_account_id ON entry (account_id);
CREATE INDEX IF NOT EXISTS idx_entry_counter_account_id ON entry (counter_account_id);
CREATE INDEX IF NOT EXISTS idx_entry_category_id ON entry (category_id);
CREATE INDEX IF NOT EXISTS idx_entry_fee_category_id ON entry (fee_category_id);
CREATE INDEX IF NOT EXISTS idx_entry_kind_id ON entry (kind_id);
CREATE INDEX IF NOT EXISTS idx_entry_tag_tag ON entry_tag (tag_id);
"#;

pub const SCHEMA_VERSION: i32 = 2;
