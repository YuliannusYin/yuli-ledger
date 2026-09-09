本仓库是 **Yuli Ledger**：给自己用的 Windows 桌面记账软件。说明与代码标识用英语；软件界面以英语为源文案，简体中文（`zh-Hans`）为翻译。

当前还没有应用代码。先读 [docs/README.md](docs/README.md)。

# Yuli Ledger

Personal, offline bookkeeping for a single Windows PC. Not a phone app, not a cloud product, not a commercial money suite.

**Status:** design documentation only. Implementation (Tauri 2, web UI, SQLite) has not started.

## Docs

Start at [docs/README.md](docs/README.md). Reading order is listed there (product, domain, entry kinds, features, architecture, i18n, UI, roadmap).

## v1 in one line

Record `income`, `expense`, `repayment`, `prepayment`, and `transfer` entries; browse a ledger; run week / month / year / custom reports. Data stays in `%LOCALAPPDATA%\YuliLedger\ledger.sqlite`.

## Git messages

Commit and annotated-tag wording: see `.cursor/rules/git-message.mdc`.
