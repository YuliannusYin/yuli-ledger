# Features

v1 has three product surfaces: **bookkeeping** (write), **ledger** (read/filter), **reports** (aggregate). Account and category management exist only as much as bookkeeping needs.

Charts: **figures first**, thin bars, no pie as the primary view. Visual rules: [ui.md](ui.md).

Kind field requirements: [entry-kinds.md](entry-kinds.md).

## Bookkeeping

### Record an entry

The user picks a kind from the registry (`implemented: true`): `income`, `expense`, `repayment`, `prepayment`, `transfer`. Shared fields:

- Amount (CNY, two decimal places → integer minor units). For transfer this is the **source** amount (what left).
- Occurred at: local year, month, day, hour, minute (seconds not shown; stored as UTC, seconds zero)
- Primary account (pre-filled with the ledger default)
- Tags (optional, create-on-type)
- Note (optional)

Kind-specific:

| Kind | Extra required | Extra optional |
|------|----------------|----------------|
| `income`, `expense`, `repayment`, `prepayment` | Subcategory (after a main category) | — |
| `transfer` | Destination account; destination amount (defaults to source amount); **subcategory** (seed: WeChat withdrawal / Between accounts) | If destination is less than source: **fee category** (default `defaultFeeCategoryId`, seeded `preset.category.transfer.fee`) |

The UI must not hardcode a two-button income/expense-only control that cannot grow. Short copy on the form:

- Repayment / prepayment: does not count as spending; cash still leaves the account.
- Transfer: pick a subcategory (seed: WeChat withdrawal / Between accounts). Fee, if any, is on this same row (default fee category is Transfer fee, user-changeable).

Defaults: occurred at = now (local, to the minute); account = default account; transfer destination amount = source amount (fee 0).

Save validates through the **kind module** plus shared entry rules ([domain-model.md](domain-model.md)).

### Edit

Fields may be changed, including kind. Changing kind **clears** kind-specific columns that the new kind does not use (counterparty, fee category) and resets payload to `{ "v": 1 }`. The user must satisfy the new kind’s required fields before save.

Do not edit `createdAt`. Update `updatedAt` on save.

### Delete

v1 uses **hard delete** after an explicit confirm. There is no trash, undo stack, or `deletedAt`. A personal single-file ledger does not need soft delete yet; adding a flag later is possible without changing this v1 rule.

Deleting an entry removes its `EntryTag` rows. Tags themselves remain.

### Accounts and categories

First-class management, not a hidden settings dump. All lists come from the database.

**Accounts**

- Create, rename, set `accountKind`, opening balance / date, **note**, sort order, set default
- Delete if unused (no entries on either side), not the last account, and not the current default (pick another default first)

**Categories**

- Create, rename, reorder **mains** and **subs**
- Delete a sub if no entry uses it and it is not `defaultFeeCategoryId`
- Delete a main if all descendants are unused (children deleted with it)

No bulk recategorize in v1. Occupied rows stay until the user edits or deletes those entries.

## Ledger

The ledger is the chronological book of entries, newest `occurredAt` first. Tie-break: `createdAt` descending, then `id`. It opens listing **every** entry. Filters apply only when the user runs them, uses a preset, or jumps from Reports.

Each row shows enough to scan: occurred at (local), kind label, amount, account(s), category (and fee category when a transfer has a fee), tags, note excerpt.

Transfer row pattern: source account → destination account, source amount, destination amount if different, fee if any.

Opening a row shows the full entry. Edit and delete are available from detail (and may be available inline later; not required).

### Filters

Combine with AND:

| Filter | Behavior |
|--------|----------|
| Time range | Inclusive local-calendar range on `occurredAt`. Omitted bounds are unbounded (both empty = all time). One-sided ranges are allowed. Both ends set with from > to is invalid. “This month” fills the current local month and applies immediately. “All” clears every filter and lists all entries. |
| Kind | One or more implemented kinds. |
| Account | One or more accounts (include archived if they still have rows). An entry matches if `accountId` **or** `counterAccountId` is in the set. |
| Category | A main category (all its subs) or one subcategory. Matches `categoryId` or `feeCategoryId`. |
| Tag | Entry has this tag (v1: one tag is enough; multiple tags as OR or AND can wait). |
| Note contains | Case-insensitive substring on `note`. |

No search engine, no pinyin index, no full-text virtual table required in v1.

### Empty and error states

- No entries at all (no filters applied): explain how to record the first one.
- Applied filters match nothing: say so; do not reuse the first-run empty copy. Empty vs filter-empty follows the **last applied** filter set, not draft fields the user has not run yet.

## Reports

A report is a read-only view over a **local-calendar** range. An entry is included iff its `occurredAt` local date falls in that range. P&L rules: [entry-kinds.md](entry-kinds.md).

There is **no AI summary**, no generated prose, and no network. Offline figures only.

Visual treatment: [ui.md](ui.md). Charts: **point-line** trend, **pie** composition plus a table, **bar** comparison of the previous eight periods. No AI, no 3D, no marketing cards.

### Period modes

Tabs: **Week**, **Month**, **Year**, **Custom**. Persist the last mode (and custom bounds) in ledger settings.

| Mode | Range | Prev / next | Trend points | Comparison bars |
|------|--------|-------------|--------------|-----------------|
| Week | Monday 00:00 through Sunday end-of-day, local. Display the two dates (and optional ISO week number). | Adjacent weeks | One point per **day** (7) | **8 weeks** ending with the current week |
| Month | Calendar month, local | Adjacent months | One point per **day** in the month | **8 months** ending with the current month |
| Year | Calendar year 1 Jan–31 Dec, local | Adjacent years | One point per **month** (12) | **8 years** ending with the current year |
| Custom | Inclusive local **dates** `from` … `to` (`from <= to`). Time-of-day ignored for the bounds; the whole local day is in. Cap range at **10 years**. | None (user edits dates) | If `to - from + 1 ≤ 31` days: **daily**; else **calendar months** that intersect the range | **None** (arbitrary range) |

Week starts **Monday** (common in zh-CN; not Sunday). The first/last week of a year may include days of the adjacent year; that is correct.

### Side: expense vs income

A two-state control: **Expense** | **Income**. The page shows **one side at a time**.

| Side | Totals and charts use |
|------|------------------------|
| Expense | `expense` amounts + **transfer fees** |
| Income | `income` amounts |

Net (`income − expense`) is a single muted figure that stays visible on both sides so the page does not hide the other side entirely.

**Secondary** (not in the side total): repayment, prepayment, transfer arrival volume. Always a compact zinc line under the figures, both sides.

### Sections (required)

Order on the page:

1. **Mode tabs** + period chrome + side toggle  
2. **Figures** — side total; average (week/month/custom: **per day**; year: **per month**); week/month: vs previous period (环比); year: vs last year (同比) as a number  
3. **Trend** — **point-line chart** (line + dots, no area fill) over the trend points in the table above. Hover shows that bucket’s amount (mono). Expense `--accent-out`, income `--accent-in`. Empty buckets at 0.  
4. **Composition** — toggle **by main category** / **by subcategory**. A **pie** (2D, no 3D, no donut hole required) **and** a table of amount + percent (like a legend with numbers). Sort table by amount descending; percent of the **side total**; omit zeros. Slice color = category `colorHex` ([ui.md](ui.md)). Transfer fees use `feeCategoryId`. If the pie would have **more than 10** slices, draw the largest 9 plus an **Other** slice; the table still lists every row.  
5. **By account** — same side, same period (table; not a second pie)  
6. **Comparison bars** — Week / Month / Year only: **eight** vertical bars, oldest on the left, **current period included** as the last bar (current + seven previous). Height = that bucket’s **side** total. Current bar: hairline emphasis. Missing history is a bar of 0, not a skipped slot.  
7. **Ranking** — entries in this period that belong to the side, highest amount first (cap **20**). Expense side: `expense` rows by `amountMinor`, plus `transfer` rows with fee > 0 ranked by **fee**. Income side: `income` rows. Columns: occurred at, kind, category, note excerpt, amount. Click opens the ledger inspector on that entry. “More” applies the same period + kind filter on **Ledger**

Empty period: muted “No entries in this range”, keep chrome.

### Explicitly not required in v1

- Tag breakdown  
- Budgets vs actual  
- Export CSV/PDF  
- Generated commentary  
- 3D charts, area-gradient under the trend line  
- A comparison-bar strip on **Custom** mode

### Consistency

- Side total = sum of that side’s definition above  
- Composition percents add to 100% of the side total (rounding: last row absorbs 1 fen)  
- Ranking amounts are the same numbers that feed the side total for those rows  

Ledger list vs reports: an `expense` filter on the ledger does not include transfer fees; the report expense total does. The report definition wins for P&L.

## Information architecture (logical screens)

Do not treat this as a wireframe or component library.

1. **Record** — compose and save an entry
2. **Ledger** — list, filter, detail
3. **Reports** — week / month / year / custom; expense or income side; sections above
4. **Accounts** — list, create, edit (including note), delete-when-unused
5. **Categories** — mains and subs, create, rename, delete-when-unused
6. **Settings (minimal)** — default account, default fee category, UI language, color scheme ([ui.md](ui.md)), path to the database file (read-only display) so backup is obvious

Navigation is a **left rail** (Record, Ledger, Reports, Accounts, Categories, Settings). Layout, density, and chrome: [ui.md](ui.md).

## Non-features in these surfaces

- Recurring entry templates
- Attachments / receipt photos
- Multi-select batch delete (optional later)
- Notifications or calendar sync
- Settling a prepayment into a later expense without moving cash
- Tracking remaining loan / credit balances
