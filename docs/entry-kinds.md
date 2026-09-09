# Entry kinds

Entry kinds are an **open registry**, not a closed enum baked into every screen. v1 implements five kinds. Later kinds must not require new *required* core columns on `Entry`. Nullable **counterparty** fields exist so a two-account kind can be queried without JSON.

See [domain-model.md](domain-model.md) for shared fields and [features.md](features.md) for UI.

## Design rule

Do not scatter `if kindId == "income"` (or equivalent) through storage, list queries, and widgets.

Allowed:

- A **registry** of kind descriptors (data)
- A **kind module** per kind (payload validation, form fields, whether category / counterparty are required)

Storage loads and saves the same entry row for every kind. SQL that lists the ledger filters `kind_id` as a value, not as a rewritten schema.

## Registry descriptor

Each kind publishes a descriptor. Conceptual fields:

| Field | Meaning |
|-------|---------|
| `id` | Stable token. English, never localized. |
| `labelKey` | i18n key for the display name, e.g. `kind.income`. |
| `payloadSchemaVersion` | Integer the kind module understands today. |
| `balanceEffect` | `increase` \| `decrease` \| `transfer` \| `none` |
| `reportBucket` | `income` \| `expense` \| `none` — how **`amountMinor`** counts in period P&L |
| `feeReportBucket` | `expense` \| `none` — how a **transfer shortfall** counts (v1: `expense` on `transfer`, `none` on other kinds) |
| `categoryRequired` | Whether `categoryId` must be a subcategory |
| `counterpartyRequired` | Whether `counterAccountId` and `counterAmountMinor` must be set |
| `implemented` | `true` for shipped kinds |

Balance and reports **must** use these descriptor fields. That is how `repayment` can leave an account without counting as spending, and how `transfer` can move two accounts, without editing every screen’s kind list.

## Kind payload

`Entry.kindPayload` is a JSON object:

```json
{
  "v": 1
}
```

- `v` is required. It is the payload schema version for **that kind**, not the database schema version.
- v1 kinds store **money and account FKs on columns**, not in JSON (see [domain-model.md](domain-model.md) counterparty slot). Payload has no extra business fields.
- Extra keys must be ignored on read. Unknown `v`: show core fields; do not overwrite payload on a naive edit.

## How amounts hit accounts

`amountMinor` is always **non-negative**. Direction comes from `balanceEffect`:

| `balanceEffect` | Account `accountId` | Account `counterAccountId` |
|-----------------|---------------------|----------------------------|
| `increase` | += `amountMinor` | unused (null) |
| `decrease` | -= `amountMinor` | unused (null) |
| `transfer` | -= `amountMinor` | += `counterAmountMinor` |
| `none` | no change | no change |

v1 has no kind with `balanceEffect: none`.

## How amounts hit reports (P&L)

**Income / expense / net** (primary reports):

| Kind | Counts as |
|------|-----------|
| `income` | income += `amountMinor` |
| `expense` | expense += `amountMinor` |
| `repayment` | neither |
| `prepayment` | neither |
| `transfer` | expense += **fee** only, where `fee = amountMinor - counterAmountMinor` |

Fee is `>= 0` by validation. When fee is 0, the transfer is P&L-neutral (money moved between your own accounts). When fee is 1 fen, net worth fell by 1 fen; that fen is an expense under `feeCategoryId`.

**Secondary totals** (required in v1 so these rows are not invisible):

- Sum of `repayment` `amountMinor`
- Sum of `prepayment` `amountMinor`
- Transfer volume: sum of `counterAmountMinor` (what arrived)
- Transfer fees: sum of fees (must equal the expense attributed to transfer kinds)

**Net** remains `income - expense`. It does **not** subtract repayment or prepayment. Those reduced cash (account balances) but are not consumption in this product.

## v1 kinds

### `income`

| Descriptor | Value |
|------------|--------|
| `id` | `income` |
| `labelKey` | `kind.income` |
| `balanceEffect` | `increase` |
| `reportBucket` | `income` |
| `feeReportBucket` | `none` |
| `categoryRequired` | yes |
| `counterpartyRequired` | no |
| `payload` | `{ "v": 1 }` |

Money received into `accountId`. Amount is the gross amount recorded.

### `expense`

| Descriptor | Value |
|------------|--------|
| `id` | `expense` |
| `labelKey` | `kind.expense` |
| `balanceEffect` | `decrease` |
| `reportBucket` | `expense` |
| `feeReportBucket` | `none` |
| `categoryRequired` | yes |
| `counterpartyRequired` | no |
| `payload` | `{ "v": 1 }` |

Money leaving `accountId` as consumption. Amount is what left the account.

### `repayment`

Thin kind: cash (or another asset account) goes down; you describe **what** you repaid with **category + note**. No liability table, no remaining balance, no interest.

| Descriptor | Value |
|------------|--------|
| `id` | `repayment` |
| `labelKey` | `kind.repayment` |
| `balanceEffect` | `decrease` |
| `reportBucket` | `none` |
| `feeReportBucket` | `none` |
| `categoryRequired` | yes |
| `counterpartyRequired` | no |
| `payload` | `{ "v": 1 }` |

Use this when the debt is **not** tracked as an account. If the credit card *is* an account in the ledger, paying it is a **`transfer`** into that account, not a `repayment`.

Do not fake repayment as `expense` plus a tag.

### `prepayment`

Thin kind: money leaves now for something consumed later (advance rent, a deposit, a membership). Category + note name the item. No prepaid-asset table, **no later settlement** that turns this into `expense`.

| Descriptor | Value |
|------------|--------|
| `id` | `prepayment` |
| `labelKey` | `kind.prepayment` |
| `balanceEffect` | `decrease` |
| `reportBucket` | `none` |
| `feeReportBucket` | `none` |
| `categoryRequired` | yes |
| `counterpartyRequired` | no |
| `payload` | `{ "v": 1 }` |

**Cash already left.** Do not post a second `expense` for the same money when the month “arrives”; that would subtract cash twice. v1 therefore **understates consumption** in later months for prepaid goods. That is the accepted tradeoff until a settlement kind exists.

### `transfer`

Move value between **two of your accounts**. WeChat balance withdrawn to a bank card is this kind: source = WeChat wallet account, destination = bank account. The same kind covers Alipay withdrawal, cash deposited to bank, and similar.

| Descriptor | Value |
|------------|--------|
| `id` | `transfer` |
| `labelKey` | `kind.transfer` |
| `balanceEffect` | `transfer` |
| `reportBucket` | `none` |
| `feeReportBucket` | `expense` |
| `categoryRequired` | yes (classify the move; any subcategory, not hardcoded to the Transfer main) |
| `counterpartyRequired` | yes |
| `payload` | `{ "v": 1 }` |

Columns (not payload):

| Field | Role |
|-------|------|
| `accountId` | Source (money leaves). Example: WeChat. |
| `amountMinor` | Amount **leaving** the source. Example: 500.00 CNY → `50000`. |
| `counterAccountId` | Destination (money arrives). Example: bank card. Must differ from `accountId`. |
| `counterAmountMinor` | Amount **arriving** at the destination. Example: 499.00 CNY → `49900`. |
| `categoryId` | Required. Seeded examples: WeChat withdrawal, Between accounts. |
| `feeCategoryId` | Required iff `amountMinor > counterAmountMinor`; must be a subcategory. Null when fee is 0. Default from `defaultFeeCategoryId` (seeded Transfer fee). |

Rules:

- `0 < counterAmountMinor <= amountMinor`
- `fee = amountMinor - counterAmountMinor`
- If `fee == 0`, `feeCategoryId` must be null (no fee line)
- If `fee > 0`, `feeCategoryId` is required; that fee is the only P&L impact
- There is **no nested second entry** for the fee; one transfer row carries the fee

Example: withdraw 500.00 from WeChat, 499.00 hits the bank, 1.00 fee → source −500, dest +499, expense 1.00 under the fee category.

This is **not** a full double-entry chart of accounts. It is one entry with a source and a destination so net worth only moves by the fee.

## How to add a kind later (contract)

A new kind is:

1. A new `id` in the registry
2. Descriptor flags (`balanceEffect`, report buckets, category/counterparty required)
3. Payload JSON schema if columns are not enough
4. A kind module: form, validation, optional links to new tables
5. i18n keys for labels

**Not** required: new columns for money, time, tags, or note. A new *entity* (a loan, a prepaid asset for settlement) is a new table referenced from payload, plus possibly reusing the counterparty slot.

Likely later: liability remaining balances; prepayment **settlement** into `expense` without moving cash again.

## Unknown kinds

An older database opened by a newer app: fine.

A newer database (or a hand-edited kind id) opened by an older app:

- List and detail show core columns
- Kind label falls back to the raw `kindId`
- Balance/report: treat missing registry entries as `balanceEffect: none`, `reportBucket: none`, `feeReportBucket: none`
- Editing kind-specific fields is disabled until the kind module exists

## Anti-patterns

- Signed amounts (`-50` expense, `+50` income) instead of unsigned amount + registry effect
- `CHECK (kind_id IN ('income','expense'))` on the table
- Parallel tables per kind as the system of record
- Encoding kind in category names only (`category = "Repayment"`)
- A dedicated `wechat_withdrawal` kind (use `transfer` + Transfer category)
- Hardcoding the seeded category tree in the UI
- Recording transfer fees as a **separate** `expense` row that also decreases the source account (that would double-count the fee against cash)
