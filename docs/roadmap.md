# Roadmap

This is a sequencing document, not a calendar. Implementation starts only after these design docs are accepted.

## v1 — done when

You can run a Windows desktop build (Tauri) that:

- Seeds Default account, the preset category tree in [preset-categories.md](preset-categories.md), and i18n keys. After seed, the user can add/rename/delete mains, subs, and accounts (delete only when unused).
- Records, edits, and hard-deletes **`income`**, **`expense`**, **`repayment`**, **`prepayment`**, and **`transfer`** entries with amount, local date-time to the minute, account(s), category or fee category as required, tags, note
- Transfer: two accounts, destination amount, optional fee on the same row (WeChat → bank is this kind)
- Lists those entries as a ledger with the filters in [features.md](features.md)
- Shows reports per [features.md](features.md): week / month / year / custom; point-line trend; pie + table; 8-period comparison bars; ranking
- Switches UI language between `en` and `zh-Hans`, named theme (`metal` / `claude` / `vscode` / `github` / `tiktok`), and color scheme light/dark/system
- UI follows [ui.md](ui.md) (zinc, compact Record loop)
- Stores everything in `%LOCALAPPDATA%\YuliLedger\ledger.sqlite` and shows that path for backup
- Can be built into an **NSIS installer** and a **portable folder**; both open a desktop window, not a browser

v1 is **not** done when the kind registry exists only on paper; the registry must be the runtime source for kinds, balance effects, and report buckets.

## v1 will not include

- Phone, web hosting, LAN sync, or any multi-device story
- Network features (login, telemetry, required updater)
- WeChat / Alipay / bank **import** (manual `transfer` is in)
- Liability remaining **entities**; prepaid-asset ledger; **settlement** of prepayment into expense. Per-account derived **debt** is in v1.
- A dedicated WeChat-withdrawal kind (use `transfer`)
- Multi-currency, budgets, attachments, recurring templates
- Soft delete, trash, or audit log UI
- Tag-based report breakdown (optional stretch, not a gate)
- Full double-entry chart of accounts

## After v1 (likely order)

Order can change; dependencies cannot.

1. **Quality of life** — more filters, tag report, recurring drafts, attachments. (CSV/TXT export and JSON backup are in Settings.)
2. **Named debts** — a dedicated liability entity beyond the per-account derived debt number; `repayment` payload may point at it; still `reportBucket: none`.
3. **Prepayment settlement** — consume a prepayment into `expense` (or a settlement kind) **without** moving cash a second time.
4. **Budgets** — sit on categories and time ranges; they must not alter posted entries.
5. **Extra currencies** — ledger-level or per-account; this is a deliberate schema project, not a column sneak-in.

Each new kind (if any) follows [entry-kinds.md](entry-kinds.md): registry row, payload schema, kind module, i18n keys. Prefer reusing the counterparty slot before adding required core columns.

## Documentation vs code

This `docs/` tree is the source of truth until an implementation ADR contradicts it. When code lands, update these files in the same change if behavior diverges — do not leave the design describing a paper app.
