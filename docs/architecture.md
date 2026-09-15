# Architecture

Target runtime: **one Windows desktop process**, **no server**, **no network requirement**. The shipped product is a **normal Windows program**: double-click an `.exe` (or a Start Menu shortcut after setup). It is **not** a site you open in Chrome, and **not** a frontend + backend you deploy with the CLI.

**Developers** still install Node.js and Rust to *build* it. **You, using the finished app**, do not.

Product constraints: [product.md](product.md). Data shape: [domain-model.md](domain-model.md). Visual language: [ui.md](ui.md) (Metal zinc default, compact Record; named skins swap tokens). Do not pick Ant Design / MUI defaults.

## Stack

| Layer | Choice |
|-------|--------|
| Shell | **Tauri 2** (WebView2 on Windows, bundled with the OS on current Windows 10/11) |
| UI | **React** + **Vite** + **TypeScript** inside that WebView — never a browser tab the user launches |
| Core | **Rust** Tauri commands + kind registry + SQLite access |
| Persistence | **SQLite** via **rusqlite** with bundled SQLite (one file) |
| OS | Windows only for v1 |

No Electron. No hosted API. No `npm start` as the way to *use* the ledger. The UI must not talk to SQLite through a WebView filesystem hack; all reads and writes go through Tauri commands.

Charts: a small React-friendly library is allowed if it can do point-line, pie, and bars in [ui.md](ui.md) tokens (see that file). CSS is hand-written variables, not a component kit.

i18n catalogs live with the React app ([i18n.md](i18n.md)). A React i18n helper (for example i18next) is an implementation detail as long as English remains the source locale.

## What you click when it is done

v1 ships **both**:

1. **NSIS installer** (`YuliLedger_x.y.z_x64-setup.exe` or similar): Start Menu entry, uninstaller in Windows Settings.
2. **Portable folder**: unzip, double-click `Yuli Ledger.exe` (or `yuli-ledger.exe`). No install step. Database still lives under `%LOCALAPPDATA%\YuliLedger\` so portable and installed copies share one ledger unless we later add a portable-data mode (out of v1).

Neither artifact starts a terminal, a Node server, or a browser.

`tauri dev` is **only** for coding. It may open a window that looks like the app; it is not the release.

## Process shape

```mermaid
flowchart LR
  ui [Web_UI]
  cmds [Tauri_commands]
  domain [Kind_registry_and_rules]
  db [SQLite]
  ui --> cmds
  cmds --> domain
  cmds --> db
```

- **Web UI:** layout, forms, i18n, formatting of money and local time. No SQL.
- **Tauri commands:** grouped by use case — bookkeeping (create/update/delete entry, list accounts/categories/tags), ledger queries, report aggregations, settings. Commands return DTOs the UI can render.
- **Domain:** entry validation, kind registry lookup, balance/report bucket application. Lives on the Rust side so the UI cannot “forget” a kind and mis-total.
- **SQLite:** rows and indexes. It stores `kind_id` and `kind_payload` as data. Money and account FKs (including `counter_account_id`) are columns. SQLite does not interpret payload business meaning.

The kind registry is **canonical in core (Rust)**. The UI obtains implemented kinds and labels keys via a command (e.g. list kinds) so pickers stay in sync. Duplicating a TypeScript enum as the source of truth is not allowed.

## Database file

Use the Windows **local** app-data directory (does not roam):

`%LOCALAPPDATA%\YuliLedger\ledger.sqlite`

Tauri equivalent: `app_local_data_dir` plus a fixed file name. Create the directory on first run.

Backup and restore: **copy that file** while the app is closed (or after a flush). Settings should show the absolute path. Settings also offer **CSV and TXT entry export** and a **JSON backup** of ledger tables including pending rows (not a restore path). The Pending screen imports a **generic CSV** (`occurredAt`, `amount` only) into `pending_entry`; posting creates a normal `entry`. Do not invent a cloud backup channel.

Optional later: `VACUUM`; JSON **import**.

### SQLite notes

- Foreign keys on.
- WAL recommended for fewer readers/writers in one process.
- `kind_payload` as `TEXT` JSON.
- Indexes at least: `entry(occurred_at)`, `entry(account_id)`, `entry(counter_account_id)`, `entry(category_id)`, `entry(fee_category_id)`, `entry(kind_id)`, tag join table, `pending_entry(occurred_at)`.

Schema migrations: integer `schemaVersion` in ledger settings (or a `schema_migrations` table). Current is version 4 (`pending_entry` plus prior columns).

## Time and locale at the boundary

- Commands accept and return `occurredAt` in a documented form (prefer ISO-8601 UTC in the API).
- The UI converts to Windows local wall time for display and for “month” filter construction.
- Report grouping (day / month / year) must use the **local calendar**, not UTC `strftime`, unless the command converts first. Week = Monday–Sunday. Rules: [features.md](features.md), [domain-model.md](domain-model.md).

## i18n placement

UI strings live with the frontend catalogs ([i18n.md](i18n.md)). Rust errors destined for humans should be **stable error codes** plus optional parameters; the UI maps codes to English source strings and translations. Do not ship two independent copy decks for the same sentence.

## Security and privacy (local)

- No telemetry, no update ping required for v1.
- No password in v1 (the OS user profile is the boundary). File encryption is out of scope.
- Tauri: disable unnecessary webview APIs; no random outbound URL loads for the product to function.

## Testing (when code exists)

- Domain tests for amount parsing, balance derivation (including transfer source/dest), report buckets, transfer fees, unknown kinds.
- Command-level tests against a temp SQLite file.
- UI tests are optional for v1; they must not be the only place kind behavior is defined.

## Explicitly deferred

- Auto-updater, Authenticode **code signing** (decide when you first share the installer outside this PC)
- Plugin system for kinds (a compiled registry is enough)
- Multiple database files / switcher UI
- Portable mode that stores `ledger.sqlite` next to the exe (v1 always uses LocalAppData)
