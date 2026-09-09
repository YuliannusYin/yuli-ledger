# Glossary

Stable vocabulary for design and, later, code. Prefer these English terms in identifiers and docs. UI labels use the same concepts but go through i18n ([i18n.md](i18n.md)).

| Term | Meaning |
|------|---------|
| **Ledger** | The single book this installation holds: all accounts, categories, tags, and entries. v1 has one ledger per database file, not multiple named books. |
| **Entry** | The smallest posted unit. One row: dated, categorized, tagged, noted, of one **entry kind**. Most kinds touch one account; **transfer** also has a counterparty. |
| **Entry kind** | A registered kind of entry. v1: `income`, `expense`, `repayment`, `prepayment`, `transfer`. Identified by a stable string id. Display names are i18n keys, never the id itself. |
| **Kind payload** | Versioned JSON on the entry for fields that only that kind understands. v1 kinds keep money and account FKs on columns; payload is `{ "v": 1 }`. |
| **Kind registry** | The catalog of kinds: ids, payload schema versions, balance effects, report buckets, and whether category / counterparty are required. Application behavior looks up the registry instead of hardcoding kinds. |
| **Account** | A named pot of money the user creates and edits (cash, bank, WeChat, …), with optional **note**. Every entry has a primary `accountId`. |
| **Account note** | Free text on an account, not on the entry. |
| **Counterparty account** | The second account on a `transfer` (`counterAccountId`): where money **arrives**. |
| **Transfer fee** | Derived: `amountMinor - counterAmountMinor` on a transfer. Counts as **expense** under `feeCategoryId`. Not a second entry. |
| **Opening balance** | Amount already in the account before the first entry that should affect it, as of an opening instant. |
| **Balance** | Derived: opening balance plus the signed effects of posted entries on that account (including transfer source and destination). Not a stored source of truth. |
| **Thin kind** | `repayment` and `prepayment` in v1: one outflow, category + note, no debt/prepaid entity, no remaining balance, no settlement. |
| **Category** | A node in a user-owned tree (mains and subs). Seeded on first run; afterwards add/rename/delete (when unused). Not a list compiled into the UI. |
| **Leaf category** | A subcategory used as `categoryId` or `feeCategoryId`. |
| **Tag** | A free-form label. An entry may have many tags. Tags are orthogonal to categories: one category path, many tags. |
| **Note** | Free text on an **entry**. Distinct from **account note**. |
| **Amount (minor units)** | Integer count of the smallest currency unit (fen for CNY). Never a binary floating-point money value. |
| **Occurred at** | When the economic event happened, precise to the minute. Distinct from when the row was created or edited. |
| **Report** | A read-only aggregation over a **week, month, year, or custom** local-calendar range, for the **expense** or **income** side. |
| **Report mode** | `week` (Mon–Sun), `month`, `year`, or `custom` (inclusive dates). |
| **Report side** | `expense` (including transfer fees) or `income`. |
| **Report bucket** | How a kind’s primary `amountMinor` contributes to P&L: `income`, `expense`, or `none`. |
| **Fee report bucket** | How a transfer shortfall contributes; v1 `transfer` uses `expense`. |
| **Balance effect** | How a kind changes accounts: `increase`, `decrease`, `transfer`, or `none`. |
| **Preset** | Seeded account or category whose stable id and i18n key ship with the app. User-created records are not presets. |

Do not use **transaction** for an entry. In accounting that word often means a double-entry bundle. This product’s atom is an entry.

Do not use **type** as the kind id field name in the model (`kindId` / `kind_id`). Reserve “type” for ordinary programming types.

Do not use **wechat_withdrawal** as a kind id. That event is a `transfer` entry, often with category WeChat withdrawal.
Do not confuse kind `transfer` with category key `preset.category.transfer`.
