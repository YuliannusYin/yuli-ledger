本文档是项目的**英文源设计文档**。代码标识符、条目种类 id、提交说明也使用英语。软件界面以英语为源语言，简体中文（`zh-Hans`）作为翻译，不要求你用英语记账。

These files are the source of truth for Yuli Ledger. They describe *what* to build and *why*. They do not include application code, UI kits, or function signatures.

## Reading order

1. [product.md](product.md) — why this app exists, principles, and non-goals
2. [glossary.md](glossary.md) — terms used everywhere else
3. [domain-model.md](domain-model.md) — entities, money, time, categories, tags, accounts
4. [preset-categories.md](preset-categories.md) — seeded main / sub category tree
5. [entry-kinds.md](entry-kinds.md) — extensible entry kinds; v1 five kinds
6. [features.md](features.md) — bookkeeping, ledger, reports
7. [architecture.md](architecture.md) — Windows desktop, Tauri, SQLite, local files
8. [i18n.md](i18n.md) — English source strings, Chinese as a locale
9. [ui.md](ui.md) — visual language, layout, Record/Ledger UX
10. [roadmap.md](roadmap.md) — v1 done-when, later work

## v1 at a glance

| Area | v1 |
|------|----|
| Platform | Windows desktop only, offline, single machine |
| Stack (when coding starts) | Tauri 2, React, Vite, TypeScript, rusqlite; NSIS installer + portable folder |
| Smallest unit | **Entry** |
| Entry kinds | `income`, `expense`, `repayment`, `prepayment`, `transfer` (registry, not a closed enum) |
| Classification | User-owned two-level tree, seeded then editable ([preset-categories.md](preset-categories.md)) |
| Money | Integer minor units, single currency CNY |
| Time | Year / month / day / hour / minute |
| Accounts | First-class; transfer also has a destination account |
| Features | Record, ledger, reports (week/month/year/custom), accounts, categories |
| Look | Cold zinc instrument, compact, light + dark ([ui.md](ui.md)) |

Out of v1: phone clients, network, sync, bill import, full double-entry, extra currencies, budgets, debt/prepaid *remaining-balance* ledgers, prepayment settlement.
