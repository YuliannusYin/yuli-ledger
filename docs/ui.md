# UI and UX

Visual language and interaction for the Windows desktop app. Metrics and field rules stay in [features.md](features.md). This file is the art direction: **cold metal instrument** as the default skin, compact, with a user-switchable dark scheme and four additional named skins.

Do not implement Material, Ant Design, Fluent, or “fintech purple on white.” The memorable cue is **zinc plates, hairline rules, tabular figures**.

## Direction

Yuli Ledger is a tool you sit at, not a consumer store listing. Surfaces look like **milled zinc**: cool gray, thin strokes, almost no fill decoration. One warm metal accent (**copper**) and one cool signal (**cyan**) mark expense-like vs income-like numbers. Everything else stays zinc.

| Axis | Choice |
|------|--------|
| Tone | Industrial / utilitarian, precision instrument |
| Density | Compact (more rows, tighter form). Recording is one screen, not a wizard. |
| Theme | Two independent axes, persisted in settings. **Skin** `uiTheme`: `metal` \| `claude` \| `vscode` \| `github` \| `tiktok` (null means `metal`). **Scheme** `colorScheme`: `light` \| `dark` \| `system` (null means `system`). Each skin has light and dark token sets. Same layout and density; fonts, radius, and rail contrast may follow the skin. |
| Motion | 80–160 ms opacity/color only. No bounce, no staggered page-load theatre. |
| Chrome | Custom thin title bar in `--titlebar` (Tauri decorations), not a stock Windows white frame fighting the UI. |

## Anti-patterns

- Inter, Roboto, Arial, Space Grotesk, or “system UI” as the only typeface
- Purple gradients, glassmorphism, giant 24px corner rounding, illustration mascots
- Card stacked on card; marketing hero blocks
- Candy rainbow / 3D pies; area-gradient “mountain” charts
- Encoding income/expense **only** in color (always show kind label and a sign)

## Layout

```mermaid
flowchart LR
  nav [Nav_rail]
  main [Main]
  nav --> main
```

- **Window:** default about 1280×800; minimum about 960×640; remember size and position.
- **Nav rail (left, ~200px, collapsible to icons):** Record, Pending, Ledger, Reports, Accounts, Categories, Settings. Current item: hairline + slight zinc fill, not a bright pill. Pending shows a count badge when the inbox is non-empty.
- **Main:** one working surface. No extra app header besides the window title bar.
- **Record** is its own nav item and the fastest path (`Ctrl+N` focuses Record even from elsewhere).
- Ledger **inspector:** selecting a row opens a right pane (~320px) for detail/edit/delete. Do not navigate away to a second full page for v1 detail.

Grid: **8px**. Default control height 28–32px. Table row ~32–36px. Corner radius from `--radius` (Metal **2px**; other skins may be rounder). Hairline borders `1px`.

## Typography

Metal (default) type:

| Role | Face | Use |
|------|------|-----|
| UI (Latin) | **IBM Plex Sans** | Labels, nav, buttons |
| UI (CJK) | **Noto Sans SC** | Chinese strings at the same size/weight as the Latin UI face; do not mix a serif 宋体 into chrome |
| Figures | **IBM Plex Mono** | All money and clock-like times; **tabular lining** figures |

Other skins swap `--font-ui` / `--font-mono` (see Built-in skins). CJK remains **Noto Sans SC** on every skin. Amounts stay `font-variant-numeric: tabular-nums`.

Weights: 400 body, 500 labels, 600 only for the focused amount on Record. Tracking slightly open on all-caps nav is allowed; do not use all-caps for body Chinese.

Money in lists: right-aligned mono, two fraction digits, grouping by locale ([i18n.md](i18n.md)).

## Color

Tokens (implement as CSS variables). Names are English; values are the source of truth for v1.

**Light** (Metal)

| Token | Hex | Role |
|-------|-----|------|
| `--bg` | `#f4f4f5` | Window (zinc-100) |
| `--surface` | `#fafafa` | Tables, forms |
| `--line` | `#d4d4d8` | Hairlines (zinc-300) |
| `--text` | `#18181b` | Primary text |
| `--muted` | `#71717a` | Secondary |
| `--accent-in` | `#0e7490` | Income / positive (cyan-700) |
| `--accent-out` | `#c2410c` | Expense / fee / prepayment amounts |
| `--accent-neutral` | `#52525b` | Transfer / repayment / loan amounts |
| `--danger` | `#b91c1c` | Destructive confirm only |
| `--focus` | `#3f3f46` | Focus ring (zinc, 2px) |
| `--fill` | `#e4e4e7` | Selected chrome |
| `--account-balance` | `#c9a227` | Current account balance (gold) |
| `--account-debt` | `#c2410c` | Current account debt (orange-red) |

**Dark** (Metal)

| Token | Hex |
|-------|-----|
| `--bg` | `#18181b` |
| `--surface` | `#27272a` |
| `--line` | `#3f3f46` |
| `--text` | `#fafafa` |
| `--muted` | `#a1a1aa` |
| `--accent-in` | `#22d3ee` |
| `--accent-out` | `#fb923c` |
| `--accent-neutral` | `#a1a1aa` |
| `--danger` | `#f87171` |
| `--focus` | `#d4d4d8` |
| `--account-balance` | `#e4c15a` |
| `--account-debt` | `#f97316` |

Shared chrome tokens (every skin sets these names):

| Token | Role |
|-------|------|
| `--accent` | Brand focus / selected rail hairline (not a substitute for kind labels) |
| `--radius` | Control and plate corner radius |
| `--font-ui` / `--font-mono` | UI and figures |
| `--rail-bg` / `--rail-text` | Nav rail (may contrast with `--bg`) |
| `--shadow` | Optional surface elevation (Metal: none) |
| `--titlebar` | Custom window title bar |

Trend and comparison series use `--accent-in` / `--accent-out` only. Trend and comparison charts draw **X/Y ticks** (dates / amounts) and a zinc hover tooltip (`date + amount` in mono). **Composition pie** uses per-category colors stored on the main category (`colorHex`). Subcategory slices use the parent hue at stepped lightness (sibling `sortOrder`). Do not use a random rainbow; use the muted palette below.

### Category palette (mains)

Light theme hex (dark theme: same hue, lift lightness so slices stay distinct on `--surface`):

| Preset key | Hex |
|------------|-----|
| `preset.category.transport` | `#5b7c99` |
| `preset.category.food` | `#b4532a` |
| `preset.category.housing` | `#78716c` |
| `preset.category.entertainment` | `#a16207` |
| `preset.category.telecom` | `#0e7490` |
| `preset.category.education` | `#4f46e5` |
| `preset.category.investment` | `#3f6212` |
| `preset.category.shopping` | `#9f1239` |
| `preset.category.misc` | `#52525b` |
| `preset.category.medical` | `#be185d` |
| `preset.category.income` | `#0f766e` |
| `preset.category.transfer` | `#57534e` |
| `preset.category.finance` | `#1e3a5f` |

User-created mains: assign the next unused color from the **96-color built-in palette** (the 13 seed colors, then overflow `#6b7280`, `#854d0e`, `#115e59`, `#6b21a8`, `#9a3412`, `#164e63`, then further muted hues). Persist `colorHex` so slices do not shuffle. The Categories list shows a swatch before each main; clicking that swatch opens an 8-column palette popover and writes the hex immediately. Only palette colors are allowed; duplicates across mains are allowed. Subs still store `null` and inherit the parent hue.

## Built-in skins

Palettes are **inspired by** those products (no official logos). Layout, nav structure, and Record density do not change. Apply with `html[data-skin][data-theme]`.

| Skin | `uiTheme` | Latin UI / mono | Radius | Light direction | Dark direction |
|------|-----------|-----------------|--------|-----------------|----------------|
| Metal | `metal` | IBM Plex Sans / Mono | 2px | Existing zinc tables above | Existing zinc tables above |
| Claude | `claude` | Source Sans 3 / Source Code Pro | 4px | Warm paper `#faf9f5`, terracotta `#d97757` | Warm black `#1f1e1d`, same terracotta |
| VSCode | `vscode` | Segoe UI / Cascadia Code | 2px | Light+ white, sidebar `#f3f3f3`, `#007acc` | Dark+ `#1e1e1e`, sidebar `#252526`, `#007acc` |
| GitHub | `github` | Mona Sans / IBM Plex Mono | 6px | Primer light, income green / expense red | Primer dark `#0d1117` / `#161b22` |
| TikTok | `tiktok` | Outfit / IBM Plex Mono | 8px | Light gray, pink `#fe2c55` (expense darkened for contrast) | Black `#000` / `#121212`, pink + cyan `#25f4ee` |

Income/expense still use `--accent-in` / `--accent-out` plus kind label and sign. Category pie colors stay on `colorHex`.

## Recording (speed)

Job: under a minute, preferably **one glance + keyboard**.

- Single column form on `--surface`, not stepped screens. Record form about 480px wide, left-aligned.
- **Amount** is the first and largest field (mono, ~28–32px). Transfer: source and destination amounts on **one row**; destination defaults to the source value; if they differ, show fee on the next line in `--accent-out`.
- **Occurred at:** native `date` + `time` (minute precision) plus a **Now** control that fills the current local minute. After save, keep the last recorded time (do not auto-reset to now).
- **Kind:** horizontal set of compact hairline buttons (content width, not stretched across the row) generated from the kind registry (must grow past two). Selected = zinc fill + label, not a colorful rainbow per kind.
- Account (and counterparty when the kind needs it) on one row; category **main | sub** two selects on one row, not a deep tree widget. Tags, note: compact rows below.
- Primary action: **Save** (`Ctrl+Enter`), full width of the Record form. After a successful save, **stay on Record**, clear amount/note/tags, keep kind/account/category/**time**. Persist that kept configuration in ledger settings so a restart restores it.
- Validation: inline under the field, zinc + `--danger` text, no modal for ordinary errors.
- Delete is not on this screen (edit/delete from ledger inspector).

## Ledger

- **Day plates**, not a marketing card feed. Group by local calendar date of `occurredAt` (newest day first). One shared column header above the list; each day is a zinc hairline plate.
- Plate header: local date on the left; that day’s expense total (`reportBucket` expense, including prepayment) and income total on the right in mono. Transfer and repayment do not enter those two sums.
- Inner rows: time (hour:minute only), kind, amount, account(s), category, tags, note excerpt.
- Transfer cells: `Source → Dest` on one line; amount column shows source, and dest/fee if different, in muted mono.
- Sticky filter bar: period, kind, account, category, tag, note contains. Compact inputs; “this month” and “all” are text controls, not a large calendar hero. Date fields start empty (no time bound) until the user filters.
- Row hover: slight surface shift. Selected row: hairline inside the row + inspector open.
- Empty: one muted sentence + control to go to Record. No illustration.

## Pending

Same zinc list + inspector as Ledger. Toolbar: download CSV template, import CSV, pending count. Rows start with amount and time only; kind/account/category may show “unset”. Inspector uses the Record form with empty options. Actions: save draft, post, discard. Import lives here, not in Settings.

## Reports

Information architecture: [features.md](features.md).

- One working column, not a stack of rounded marketing cards.
- Top: hairline **tabs** (Week, Month, Year, Custom) + prev/next or date pair + **Expense | Income** segmented control.
- **Figures first** (mono).
- **Trend:** point-line (stroke 1.5–2px, small square or circle marks, **no** filled area). Draw X (dates) and Y (amounts) ticks; hover shows that bucket’s date and amount in mono.
- **Composition:** 2D pie + table (swatch, name, amount, percent). Pie hole optional (donut is allowed if the center shows the side total in mono); no 3D, no slice explode.
- **Comparison:** eight zinc bars, current period outlined; fill `--accent-out` or `--accent-in` by side. Same X/Y ticks and hover as the trend chart.
- **Ranking:** ledger-like table.
- Secondary repayment / loan / transfer volume / fees: one muted line.
- Click a ranking row: Ledger + inspector.

## Accounts and categories

- **Split list:** left list of accounts or mains; right editor (name, kind, opening balance, opening debt, **note** for accounts; children for a main). Main color is chosen from a popover opened by the swatch in front of the main, not a permanently expanded grid. Account current figures on the list and in the editor summary: signed **balance** (gold `--account-balance`) `|` signed **debt** (orange-red `--account-debt`), no extra words.
- Delete: enabled only when rules in [features.md](features.md) allow; otherwise disabled with a one-line reason (e.g. “Used by 12 entries”).
- Reorder: right-click a row for **Move up** / **Move down** (accounts, mains, and subs). First/last row disables the blocked direction. Left-click still selects.

## Settings

Quiet list: language, **theme (five preview cards)**, color scheme, default account, default fee category, **read-only database path** (copy button), **export** (CSV entries and **TXT entries**, same optional date range; JSON full backup). CSV **import** is on Pending, not here. No account-cloud banners.

## Keyboard (v1)

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | Go to Record, focus amount |
| `Ctrl+Enter` | Save on Record / inspector edit |
| `Ctrl+1` … `Ctrl+7` | Nav: Record, Pending, Ledger, Reports, Accounts, Categories, Settings |
| `[` / `]` | Reports: previous / next period (week, month, year) |
| `Esc` | Close inspector; clear filter popovers |
| `Delete` | Delete in inspector after the same confirm as the button |

Windows conventions: `Ctrl`, not `Cmd`.

## Focus and contrast

- Visible focus ring (`--focus`). Tab order follows visual order on Record.
- Text on `--bg`/`--surface` meets WCAG **AA** for body and figures.
- Kind + sign + color together. Example: expense `−12.00` in copper and kind label “Expense”.

## Implementation notes (still no app code)

- Tokens in one stylesheet (or equivalent); skins and schemes swap the same names via `data-skin` and `data-theme`.
- Self-authored components. If a library is used later, restyle it to these tokens until it does not look like the library demo.
- Charts: a small library is allowed if it can render point-line, pie, and bars in these tokens without default theme chrome.

When code lands, screenshots in a later `docs/ui-gallery.md` are optional; this file remains the constraint.
