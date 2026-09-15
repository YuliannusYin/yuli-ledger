# Product

Yuli Ledger is a personal bookkeeping app for one person: the owner of this repository. It exists because built-in phone wallets and commercial money apps are the wrong shape — too much product, too little control, and data that does not live in a file you can copy.

The product is a **Windows desktop, offline, single-ledger** tool. You record money events as **entries**, browse them as a **ledger**, and summarize them as **reports**.

## Who it is for

Only you. There is no multi-user model, no household sharing, no login, and no cloud account. Design for daily use at a desk, not for a marketplace or a demo.

## Job to be done

In under a minute, record that money moved: how much, when (to the minute), which account(s), classification, optional tags and a note. Later, answer “what happened this month?” without exporting to a spreadsheet first — including spending that did not move cash (`prepayment`), “this was not spending” (`repayment`), and “I moved WeChat to the bank” (`transfer`).

## Principles

1. **Entry is the atom.** Everything you post is an entry. Features compose around entries, not around a zoo of unrelated screens.
2. **Kinds stay open.** The model must accept later kinds without rewriting required entry columns or scattering `if kind == income` through the app.
3. **Local file is the source of truth.** No network requirement. Backup means copying the database file.
4. **English is the project language; Chinese is a locale.** Identifiers, kind ids, and source UI strings are English. Simplified Chinese is a translation. User-typed names stay as typed.
5. **Prefer a thin v1 over a complete finance suite.** Ship recording, ledger, and reports. Leave investments, invoices, bank import, and named-debt *entities* for later or never.
6. **Derived numbers over cached truth.** Account balances and report totals are computed from opening balances plus posted entries, not a second write path that can drift.

## v1 scope

In:

- Create, edit, and delete entries of kinds `income`, `expense`, `repayment`, `prepayment`, `loan`, `transfer`
- User-owned **accounts** (seed Default; add/rename/delete; optional note)
- User-owned **category tree** (seed [preset-categories.md](preset-categories.md); add/rename/delete mains and subs)
- Tags and notes
- Ledger list with filters and a detail view
- **Pending** inbox: generic CSV import of amount + occurred-at; review then post
- Reports: week / month / year / custom; expense or income side; trend, composition, ranking; annual year-comparison ([features.md](features.md))

Out (see [roadmap.md](roadmap.md) for later):

- Phone or web-hosted clients
- Network, login, sync, or multi-device
- Import from WeChat, Alipay, or banks (generic CSV into Pending is in)
- Named debt *entities* and prepayment settlement into expense
- Full double-entry chart of accounts, multi-currency, budgets, attachments, recurring templates

## Non-goals

These are not “later maybe” so much as **not this product**:

- Competing with commercial apps on OCR, social sharing, or marketplace integrations
- Being a general accounting package for a company
- Requiring the internet to open last month’s lunch

## Success for v1

You can sit at a Windows PC, record today’s income, expense, repayment, prepayment, loan, or a WeChat-to-bank transfer (with optional fee), find that row in the ledger, and see totals that match the rows. Account balances move as specified. The database file can be copied to a backup disk and opened again on the same app version.
