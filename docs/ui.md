# UI and UX

Visual language and interaction for the Windows desktop app. Metrics and field rules stay in [features.md](features.md). This file is the art direction: **cold metal instrument**, compact, light by default with a dark theme.

Do not implement Material, Ant Design, Fluent, or “fintech purple on white.” The memorable cue is **zinc plates, hairline rules, tabular figures**.

## Direction

Yuli Ledger is a tool you sit at, not a consumer store listing. Surfaces look like **milled zinc**: cool gray, thin strokes, almost no fill decoration. One warm metal accent (**copper**) and one cool signal (**cyan**) mark expense-like vs income-like numbers. Everything else stays zinc.

| Axis | Choice |
|------|--------|
| Tone | Industrial / utilitarian, precision instrument |
| Density | Compact (more rows, tighter form). Recording is one screen, not a wizard. |
| Theme | Light default; user-switchable dark. Same layout, inverted zinc. Persist with `uiLanguage` in settings as a separate `colorScheme`: `light` \| `dark` \| `system`. |
| Motion | 80–160 ms opacity/color only. No bounce, no staggered page-load theatre. |
| Chrome | Custom thin title bar in the same zinc as the app (Tauri decorations), not a stock Windows white frame fighting the UI. |

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
- **Nav rail (left, ~200px, collapsible to icons):** Record, Ledger, Reports, Accounts, Categories, Settings. Current item: hairline + slight zinc fill, not a bright pill.
- **Main:** one working surface. No extra app header besides the window title bar.
- **Record** is its own nav item and the fastest path (`Ctrl+N` focuses Record even from elsewhere).
- Ledger **inspector:** selecting a row opens a right pane (~320px) for detail/edit/delete. Do not navigate away to a second full page for v1 detail.

Grid: **8px**. Default control height 28–32px. Table row ~32–36px. Corner radius **2–4px** (almost square). Hairline borders `1px`.

## Typography

| Role | Face | Use |
|------|------|-----|
| UI (Latin) | **IBM Plex Sans** | Labels, nav, buttons |
| UI (CJK) | **Noto Sans SC** | Chinese strings at the same size/weight as Plex; do not mix a serif 宋体 into chrome |
| Figures | **IBM Plex Mono** | All money and clock-like times; **tabular lining** figures |

Weights: 400 body, 500 labels, 600 only for the focused amount on Record. Tracking slightly open on all-caps nav is allowed; do not use all-caps for body Chinese.

Money in lists: right-aligned mono, two fraction digits, grouping by locale ([i18n.md](i18n.md)).

## Color

Tokens (implement as CSS variables). Names are English; values are the source of truth for v1.

**Light**

| Token | Hex | Role |
|-------|-----|------|
| `--bg` | `#f4f4f5` | Window (zinc-100) |
| `--surface` | `#fafafa` | Tables, forms |
| `--line` | `#d4d4d8` | Hairlines (zinc-300) |
| `--text` | `#18181b` | Primary text |
| `--muted` | `#71717a` | Secondary |
| `--accent-in` | `#0e7490` | Income / positive (cyan-700) |
| `--accent-out` | `#c2410c` | Expense / fee (copper) |
| `--accent-neutral` | `#52525b` | Transfer / repayment / prepayment amounts |
| `--danger` | `#b91c1c` | Destructive confirm only |
| `--focus` | `#3f3f46` | Focus ring (zinc, 2px) |

**Dark**

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

Trend and comparison series use `--accent-in` / `--accent-out` only. **Composition pie** uses per-category colors stored on the main category (`colorHex`). Subcategory slices use the parent hue at stepped lightness (sibling `sortOrder`). Do not use a random rainbow; use the muted palette below.

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

User-created mains: assign the next unused color from this list, then overflow: `#6b7280`, `#854d0e`, `#115e59`, `#6b21a8`, `#9a3412`, `#164e63`. Persist `colorHex` so slices do not shuffle. v1 may auto-assign only (no color picker).

## Recording (speed)

Job: under a minute, preferably **one glance + keyboard**.

- Single column form on `--surface`, not stepped screens.
- **Amount** is the first and largest field (mono, ~28–32px). Transfer: source amount first; destination amount on the next row, defaulting to the same value; if they differ, show fee on a third row in `--accent-out`.
- **Kind:** horizontal set of equal hairline buttons generated from the kind registry (must grow past two). Selected = zinc fill + label, not a colorful rainbow per kind.
- Account, category (main then sub), tags, note: compact rows. Category: two selects, not a deep tree widget.
- Primary action: **Save** (`Ctrl+Enter`). After a successful save, **stay on Record**, clear amount/note/tags, keep kind/account/time (time may snap to now). That is the daily loop.
- Validation: inline under the field, zinc + `--danger` text, no modal for ordinary errors.
- Delete is not on this screen (edit/delete from ledger inspector).

## Ledger

- **Table**, not a feed of cards. Columns: time, kind, amount, account(s), category, tags, note excerpt.
- Transfer cells: `Source → Dest` on one line; amount column shows source, and dest/fee if different, in muted mono.
- Sticky filter bar: period, kind, account, category, tag, note contains. Compact inputs; “this month” and “all” are text controls, not a large calendar hero. Date fields start empty (no time bound) until the user filters.
- Row hover: slight surface shift. Selected row: hairline inside the row + inspector open.
- Empty: one muted sentence + control to go to Record. No illustration.

## Reports

Information architecture: [features.md](features.md).

- One working column, not a stack of rounded marketing cards.
- Top: hairline **tabs** (Week, Month, Year, Custom) + prev/next or date pair + **Expense | Income** segmented control.
- **Figures first** (mono).
- **Trend:** point-line (stroke 1.5–2px, small square or circle marks, **no** filled area).
- **Composition:** 2D pie + table (swatch, name, amount, percent). Pie hole optional (donut is allowed if the center shows the side total in mono); no 3D, no slice explode.
- **Comparison:** eight zinc bars, current period outlined; fill `--accent-out` or `--accent-in` by side.
- **Ranking:** ledger-like table.
- Secondary repayment / prepayment / transfer volume: one muted line.
- Click a ranking row: Ledger + inspector.

## Accounts and categories

- **Split list:** left list of accounts or mains; right editor (name, kind, opening, **note** for accounts; children for a main).
- Delete: enabled only when rules in [features.md](features.md) allow; otherwise disabled with a one-line reason (e.g. “Used by 12 entries”).
- Reorder: simple up/down or drag; visual = hairline grab, not colorful chips.

## Settings

Quiet list: language, color scheme, default account, default fee category, **read-only database path** (copy button). No account-cloud banners.

## Keyboard (v1)

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | Go to Record, focus amount |
| `Ctrl+Enter` | Save on Record / inspector edit |
| `Ctrl+1` … `Ctrl+6` | Nav: Record … Settings in rail order |
| `[` / `]` | Reports: previous / next period (week, month, year) |
| `Esc` | Close inspector; clear filter popovers |
| `Delete` | Delete in inspector after the same confirm as the button |

Windows conventions: `Ctrl`, not `Cmd`.

## Focus and contrast

- Visible focus ring (`--focus`). Tab order follows visual order on Record.
- Text on `--bg`/`--surface` meets WCAG **AA** for body and figures.
- Kind + sign + color together. Example: expense `−12.00` in copper and kind label “Expense”.

## Implementation notes (still no app code)

- Tokens in one stylesheet (or equivalent); themes swap the same names.
- Self-authored components. If a library is used later, restyle it to these tokens until it does not look like the library demo.
- Charts: a small library is allowed if it can render point-line, pie, and bars in these tokens without default theme chrome.

When code lands, screenshots in a later `docs/ui-gallery.md` are optional; this file remains the constraint.
