import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";
import type {
  AccountDto,
  CategoryDto,
  KindDto,
  TagDto,
  TrashItemDto,
  TrashItemKind,
} from "../lib/types";
import { emptyTrash, listTrash, purgeTrash, restoreTrash } from "../lib/api";
import { formatMinor } from "../lib/money";
import { accountName, categoryName } from "../lib/names";
import { formatLocalDate, formatLocalTime, localDateKey } from "../lib/time";
import ConfirmDialog from "../components/ConfirmDialog";

const FILTERS: Array<"all" | TrashItemKind> = ["all", "entry", "pending", "account", "category"];

export default function TrashScreen({
  kinds,
  accounts,
  categories,
  tags,
  locale,
  onChanged,
}: {
  kinds: KindDto[];
  accounts: AccountDto[];
  categories: CategoryDto[];
  tags: TagDto[];
  locale: string;
  onChanged: () => Promise<void>;
}) {
  const { t } = useTranslation();
  const [rows, setRows] = useState<TrashItemDto[]>([]);
  const [filter, setFilter] = useState<(typeof FILTERS)[number]>("all");
  const [selected, setSelected] = useState<string | null>(null);
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [confirm, setConfirm] = useState<"purge" | "empty" | null>(null);

  const visible = useMemo(
    () => (filter === "all" ? rows : rows.filter((r) => r.itemKind === filter)),
    [rows, filter],
  );
  const selectedRow = visible.find((r) => keyOf(r) === selected) ?? null;
  const dayGroups = useMemo(() => groupByDeletedDate(visible), [visible]);

  async function load(selectKey?: string | null) {
    const list = await listTrash();
    setRows(list);
    setLoaded(true);
    if (selectKey !== undefined) {
      setSelected(selectKey);
      return;
    }
    const next = list.filter((r) => (filter === "all" ? true : r.itemKind === filter));
    if (selected && !next.some((r) => keyOf(r) === selected)) {
      setSelected(next[0] ? keyOf(next[0]) : null);
    } else if (!selected && next.length) {
      setSelected(keyOf(next[0]));
    }
  }

  useEffect(() => {
    void load().catch(() => setLoaded(true));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (selected && !visible.some((r) => keyOf(r) === selected)) {
      setSelected(visible[0] ? keyOf(visible[0]) : null);
    }
  }, [visible, selected]);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") setSelected(null);
      if (
        e.key === "Delete" &&
        selectedRow &&
        !(e.target instanceof HTMLInputElement) &&
        !(e.target instanceof HTMLTextAreaElement)
      ) {
        setConfirm("purge");
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  async function doRestore() {
    if (!selectedRow) return;
    try {
      await restoreTrash(selectedRow.itemKind, selectedRow.id);
      setError(null);
      await onChanged();
      await load(null);
    } catch (ex) {
      setError(codeOf(ex));
    }
  }

  async function doPurge() {
    if (!selectedRow) return;
    try {
      await purgeTrash(selectedRow.itemKind, selectedRow.id);
      setConfirm(null);
      setError(null);
      await onChanged();
      await load(null);
    } catch (ex) {
      setError(codeOf(ex));
    }
  }

  async function doEmpty() {
    try {
      await emptyTrash();
      setConfirm(null);
      setError(null);
      await onChanged();
      await load(null);
    } catch (ex) {
      setError(codeOf(ex));
    }
  }

  const accById = useMemo(() => new Map(accounts.map((a) => [a.id, a])), [accounts]);
  const catById = useMemo(() => new Map(categories.map((c) => [c.id, c])), [categories]);
  const trashCatById = useMemo(
    () => new Map(rows.filter((r) => r.itemKind === "category").map((r) => [r.id, r])),
    [rows],
  );

  const emptyAll = loaded && rows.length === 0;
  const filterEmpty = loaded && rows.length > 0 && visible.length === 0;

  return (
    <div className="screen-fill">
      <h1>{t("nav.trash")}</h1>
      <div className="toolbar">
        <div className="seg">
          {FILTERS.map((id) => (
            <button
              key={id}
              type="button"
              className={filter === id ? "active" : ""}
              onClick={() => setFilter(id)}
            >
              {t(`trash.${id}`)}
            </button>
          ))}
        </div>
        <button
          type="button"
          className="btn danger"
          disabled={rows.length === 0}
          onClick={() => setConfirm("empty")}
        >
          {t("action.emptyTrash")}
        </button>
      </div>
      {error && <p className="err">{t(error)}</p>}
      <div className="ledger-layout">
        <div className="surface">
          {emptyAll && (
            <p className="muted" style={{ padding: 12 }}>
              {t("trash.empty")}
            </p>
          )}
          {filterEmpty && (
            <p className="muted" style={{ padding: 12 }}>
              {t("trash.filterEmpty")}
            </p>
          )}
          {visible.length > 0 && (
            <div className="ledger-days">
              <div className="ledger-cols trash-cols ledger-head">
                <span>{t("ledger.time")}</span>
                <span>{t("trash.type")}</span>
                <span className="num">{t("ledger.amount")}</span>
                <span>{t("trash.summary")}</span>
                <span>{t("ledger.note")}</span>
              </div>
              {dayGroups.map((group) => (
                <section key={group.date} className="day-card">
                  <header className="day-card-head">
                    <span>{formatLocalDate(group.date, locale)}</span>
                  </header>
                  {group.rows.map((row) => {
                    const kind = row.kindId ? kinds.find((k) => k.id === row.kindId) : undefined;
                    return (
                      <div
                        key={keyOf(row)}
                        className={`ledger-cols trash-cols ledger-row ${selected === keyOf(row) ? "selected" : ""}`}
                        onClick={() => setSelected(keyOf(row))}
                      >
                        <span className="mono">{formatLocalTime(row.deletedAt)}</span>
                        <span>{t(`trash.${row.itemKind}`)}</span>
                        <span className={`num ${kindClass(kind)}`}>
                          {row.amountMinor != null ? formatMinor(row.amountMinor, locale) : "—"}
                        </span>
                        <span>{summaryOf(row, kinds, accById, catById, trashCatById, t)}</span>
                        <span>{row.note?.slice(0, 40)}</span>
                      </div>
                    );
                  })}
                </section>
              ))}
            </div>
          )}
        </div>
        {selectedRow && (
          <aside className="inspector surface">
            <TrashInspector
              row={selectedRow}
              kinds={kinds}
              accounts={accounts}
              categories={categories}
              tags={tags}
              trashCats={trashCatById}
              locale={locale}
            />
            <div className="row">
              <button type="button" className="btn primary" onClick={() => void doRestore()}>
                {t("action.restore")}
              </button>
              <button type="button" className="btn danger" onClick={() => setConfirm("purge")}>
                {t("action.purge")}
              </button>
            </div>
          </aside>
        )}
      </div>
      {confirm === "purge" && (
        <ConfirmDialog
          message={t("confirm.purgeTrash")}
          onCancel={() => setConfirm(null)}
          onConfirm={() => void doPurge()}
        />
      )}
      {confirm === "empty" && (
        <ConfirmDialog
          message={t("confirm.emptyTrash")}
          onCancel={() => setConfirm(null)}
          onConfirm={() => void doEmpty()}
        />
      )}
    </div>
  );
}

function TrashInspector({
  row,
  kinds,
  accounts,
  categories,
  tags,
  trashCats,
  locale,
}: {
  row: TrashItemDto;
  kinds: KindDto[];
  accounts: AccountDto[];
  categories: CategoryDto[];
  tags: TagDto[];
  trashCats: Map<string, TrashItemDto>;
  locale: string;
}) {
  const { t } = useTranslation();
  const kind = row.kindId ? kinds.find((k) => k.id === row.kindId) : undefined;
  const account = namedAccount(row.accountId, accounts, t);
  const counter = namedAccount(row.counterAccountId, accounts, t);
  const category = namedCategory(row.categoryId, categories, trashCats, t);
  const fee = namedCategory(row.feeCategoryId, categories, trashCats, t);
  const parent = namedCategory(row.parentId, categories, trashCats, t);
  const tagNames = row.tagIds
    .map((id) => tags.find((tg) => tg.id === id)?.name)
    .filter((n): n is string => Boolean(n))
    .join(", ");

  return (
    <div>
      <Meta label={t("trash.type")} value={t(`trash.${row.itemKind}`)} />
      <Meta label={t("trash.deletedAt")} value={formatLocalDate(row.deletedAt, locale) + " " + formatLocalTime(row.deletedAt)} />
      {(row.itemKind === "entry" || row.itemKind === "pending") && (
        <>
          {row.occurredAt && (
            <Meta
              label={t("field.occurredAt")}
              value={formatLocalDate(row.occurredAt, locale) + " " + formatLocalTime(row.occurredAt)}
            />
          )}
          <Meta label={t("ledger.kind")} value={kind ? t(kind.labelKey) : row.kindId || t("pending.unset")} />
          {row.amountMinor != null && (
            <Meta label={t("field.amount")} value={formatMinor(row.amountMinor, locale)} />
          )}
          {row.counterAmountMinor != null && (
            <Meta label={t("field.destAmount")} value={formatMinor(row.counterAmountMinor, locale)} />
          )}
          <Meta label={t("field.account")} value={account || t("pending.unset")} />
          {row.counterAccountId && <Meta label={t("field.destAccount")} value={counter} />}
          <Meta label={t("field.category")} value={category || t("pending.unset")} />
          {row.feeCategoryId && <Meta label={t("field.feeCategory")} value={fee} />}
          {tagNames && <Meta label={t("field.tags")} value={tagNames} />}
          {row.note && <Meta label={t("field.note")} value={row.note} />}
        </>
      )}
      {row.itemKind === "account" && (
        <>
          <Meta label={t("field.name")} value={itemName(row, t)} />
          {row.accountKind && <Meta label={t("field.accountKind")} value={t(`accountKind.${row.accountKind}`)} />}
          {row.note && <Meta label={t("field.accountNote")} value={row.note} />}
        </>
      )}
      {row.itemKind === "category" && (
        <>
          <Meta label={t("field.name")} value={itemName(row, t)} />
          <Meta label={t("field.mainCategory")} value={row.parentId ? parent : "—"} />
        </>
      )}
    </div>
  );
}

function Meta({ label, value }: { label: string; value: string }) {
  return (
    <div className="meta-row">
      <label>{label}</label>
      <div>{value}</div>
    </div>
  );
}

function keyOf(row: TrashItemDto): string {
  return `${row.itemKind}:${row.id}`;
}

function groupByDeletedDate(rows: TrashItemDto[]): { date: string; rows: TrashItemDto[] }[] {
  const map = new Map<string, TrashItemDto[]>();
  const order: string[] = [];
  for (const row of rows) {
    const date = localDateKey(row.deletedAt);
    const bucket = map.get(date);
    if (bucket) bucket.push(row);
    else {
      map.set(date, [row]);
      order.push(date);
    }
  }
  return order.map((date) => ({ date, rows: map.get(date)! }));
}

function itemName(row: TrashItemDto, t: TFunction): string {
  if (row.name) return row.name;
  if (row.presetKey) return t(row.presetKey);
  return row.id;
}

function namedAccount(id: string | null, accounts: AccountDto[], t: TFunction): string {
  if (!id) return "";
  const acc = accounts.find((a) => a.id === id);
  return acc ? accountName(acc, t) : id;
}

function namedCategory(
  id: string | null,
  categories: CategoryDto[],
  trashCats: Map<string, TrashItemDto>,
  t: TFunction,
): string {
  if (!id) return "";
  const cat = categories.find((c) => c.id === id);
  if (cat) return categoryName(cat, t);
  const trashed = trashCats.get(id);
  if (trashed) return itemName(trashed, t);
  return id;
}

function summaryOf(
  row: TrashItemDto,
  kinds: KindDto[],
  accById: Map<string, AccountDto>,
  catById: Map<string, CategoryDto>,
  trashCats: Map<string, TrashItemDto>,
  t: TFunction,
): string {
  if (row.itemKind === "account" || row.itemKind === "category") {
    const parent = row.parentId
      ? namedCategory(row.parentId, [...catById.values()], trashCats, t)
      : "";
    return parent ? `${parent} / ${itemName(row, t)}` : itemName(row, t);
  }
  const kind = row.kindId ? kinds.find((k) => k.id === row.kindId) : undefined;
  const kindLabel = kind ? t(kind.labelKey) : row.kindId ?? "";
  const acc = row.accountId ? accById.get(row.accountId) : undefined;
  const cat = row.categoryId ? catById.get(row.categoryId) : undefined;
  const bits = [kindLabel, acc ? accountName(acc, t) : "", cat ? categoryName(cat, t) : ""].filter(
    Boolean,
  );
  return bits.join(" · ") || row.id;
}

function kindClass(kind: KindDto | undefined): string {
  if (kind?.reportBucket === "income") return "in";
  if (kind?.reportBucket === "expense") return "out";
  return "neutral";
}

function codeOf(ex: unknown): string {
  return typeof ex === "object" && ex && "code" in ex ? String((ex as { code: string }).code) : "error.db";
}
