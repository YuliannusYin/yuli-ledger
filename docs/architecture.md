# Architecture

Target runtime: **one Windows desktop process**, **no server**, **no network requirement**. Coding has not started; this document constrains the later Tauri app. It does not pick a CSS framework or a concrete ORM.

Product constraints: [product.md](product.md). Data shape: [domain-model.md](domain-model.md). Visual language: [ui.md](ui.md) (zinc, compact, IBM Plex / Noto Sans SC). Do not pick Ant Design / MUI defaults.

## Stack

| Layer | Choice |
|-------|--------|
| Shell | Tauri 2 |
| UI | Web frontend (TypeScript) inside the WebView |
| Persistence | SQLite, one file |
| OS | Windows; other desktops are non-goals for v1 |

No Electron. No hosted API. The UI must not talk to SQLite through a WebView filesystem hack; all reads and writes go through Tauri commands.

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

Backup and restore for v1: **copy that file** while the app is closed (or after a flush). Settings should show the absolute path. Do not invent a cloud backup channel.

Optional later: `VACUUM`, export JSON; not required to start.

### SQLite notes

- Foreign keys on.
- WAL recommended for fewer readers/writers in one process.
- `kind_payload` as `TEXT` JSON.
- Indexes at least: `entry(occurred_at)`, `entry(account_id)`, `entry(counter_account_id)`, `entry(category_id)`, `entry(fee_category_id)`, `entry(kind_id)`, tag join table.

Schema migrations: integer `schemaVersion` in ledger settings (or a `schema_migrations` table). v1 is version 1.

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

- Auto-updater, code signing policy (decide at first distribution)
- Plugin system for kinds (a compiled registry is enough)
- Multiple database files / switcher UI
