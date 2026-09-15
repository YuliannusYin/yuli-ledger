# Glossary

Stable vocabulary for design and, later, code. Prefer these English terms in identifiers and docs. UI labels use the same concepts but go through i18n ([i18n.md](i18n.md)).

| Term | Meaning |
|------|---------|
| **Ledger** | The single book this installation holds: all accounts, categories, tags, and entries. v1 has one ledger per database file, not multiple named books. |
| **Entry** | The smallest posted unit. One row: dated, categorized, tagged, noted, of one **entry kind**. Most kinds touch one account; **transfer**, **repayment**, and **loan** also have a counterparty. |
| **Entry kind** | A registered kind of entry. Shipped: `income`, `expense`, `repayment`, `prepayment`, `loan`, `transfer`. Identified by a stable string id. Display names are i18n keys, never the id itself. Display order: expense, income, prepayment, repayment, loan, transfer. |
| **Kind payload** | Versioned JSON on the entry for fields that only that kind understands. v1 kinds keep money and account FKs on columns; payload is `{ "v": 1 }`. |
| **Kind registry** | The catalog of kinds: ids, payload schema versions, balance effects, debt effects, report buckets, and whether category / counter-account / counter-amount are required. Application behavior looks up the registry instead of hardcoding kinds. |
| **Account** | A named pot the user creates and edits (cash, bank, WeChat, …), with optional **note**, an opening **balance**, and an opening **debt**. Every entry has a primary `accountId`. |
| **Account note** | Free text on an account, not on the entry. |
| **Counterparty account** | The second account on a two-account kind (`counterAccountId`): transfer destination, the account being repaid, or the loan debt account. |
| **Transfer fee** | Derived: `amountMinor - counterAmountMinor` on a transfer. Counts as **expense** under `feeCategoryId`. Not a second entry. |
| **Opening balance** | Amount already in the account before the first entry that should affect **balance**, as of an opening instant. |
| **Opening debt** | Amount already in the account’s **debt** before the first entry that should affect debt, as of the same opening instant. |
| **Balance** | Derived: opening balance plus the signed **balanceEffect** of posted entries on that account. Independent of debt. Not a stored source of truth. |
| **Debt** | Derived: opening debt plus `debtEffectPrimary` / `debtEffectCounter` of posted entries. Independent of balance. Not a stored source of truth. |
| **Thin kind** | v1 `prepayment` still has no prepaid-asset *table* and no settlement; `repayment` still has no named-debt *entity*. Each account’s derived debt is the remaining-balance number. |
| **Category** | A node in a user-owned tree (mains and subs). Seeded on first run; afterwards add/rename/delete (when unused). Not a list compiled into the UI. |
| **Leaf category** | A subcategory used as `categoryId` or `feeCategoryId`. |
| **Tag** | A free-form label. An entry may have many tags. Tags are orthogonal to categories: one category path, many tags. |
| **Pending entry** | An imported draft that is not yet posted. Lives in `pending_entry`. Does not affect balance, debt, or reports until **Post**. |
| **Note** | Free text on an **entry**. Distinct from **account note**. |
| **Amount (minor units)** | Integer count of the smallest currency unit (fen for CNY). Never a binary floating-point money value. |
| **Occurred at** | When the economic event happened, precise to the minute. Distinct from when the row was created or edited. |
| **Report** | A read-only aggregation over a **week, month, year, or custom** local-calendar range, for the **expense** or **income** side. |
| **Report mode** | `week` (Mon–Sun), `month`, `year`, or `custom` (inclusive dates). |
| **Report side** | `expense` (including prepayment and transfer fees) or `income`. |
| **Report bucket** | How a kind’s primary `amountMinor` contributes to P&L: `income`, `expense`, or `none`. |
| **Fee report bucket** | How a transfer shortfall contributes; v1 `transfer` uses `expense`. |
| **Balance effect** | How a kind changes accounts: `increase`, `decrease`, `transfer`, or `none`. |
| **Preset** | Seeded account or category whose stable id and i18n key ship with the app. User-created records are not presets. |

Do not use **transaction** for an entry. In accounting that word often means a double-entry bundle. This product’s atom is an entry.

Do not use **type** as the kind id field name in the model (`kindId` / `kind_id`). Reserve “type” for ordinary programming types.

Do not use **wechat_withdrawal** as a kind id. That event is a `transfer` entry, often with category WeChat withdrawal.
Do not confuse kind `transfer` with category key `preset.category.transfer`.
