import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import type {
  AccountDto,
  CategoryDto,
  EntryDto,
  KindDto,
  LedgerJump,
  TagDto,
} from "../lib/types";
import { deleteEntry, listEntries, updateEntry } from "../lib/api";
import { formatMinor } from "../lib/money";
import { accountName, categoryName, mains, subsOf } from "../lib/names";
import { formatLocalDateTime, monthRange } from "../lib/time";
import ConfirmDialog from "../components/ConfirmDialog";
import EntryForm, { formFromWrite, toWrite, type FormState } from "../components/EntryForm";

export default function LedgerScreen({
  kinds,
  accounts,
  categories,
  tags,
  locale,
  jump,
  onJumpConsumed,
  onChanged,
}: {
  kinds: KindDto[];
  accounts: AccountDto[];
  categories: CategoryDto[];
  tags: TagDto[];
  locale: string;
  jump: LedgerJump | null;
  onJumpConsumed: () => void;
  onChanged: () => Promise<void>;
}) {
  const { t } = useTranslation();
  const [fromDate, setFromDate] = useState("");
  const [toDate, setToDate] = useState("");
  const [kindIds, setKindIds] = useState<string[]>([]);
  const [accountIds, setAccountIds] = useState<string[]>([]);
  const [categoryId, setCategoryId] = useState("");
  const [tagId, setTagId] = useState("");
  const [noteContains, setNoteContains] = useState("");
  const [applied, setApplied] = useState(emptyApplied);
  const [rows, setRows] = useState<EntryDto[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [form, setForm] = useState<FormState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [confirm, setConfirm] = useState(false);
  const [loaded, setLoaded] = useState(false);

  const selectedRow = rows.find((r) => r.id === selected) ?? null;

  async function load(next?: LoadOpts) {
    const f = next?.from ?? fromDate;
    const to = next?.to ?? toDate;
    const ks = next?.kinds ?? kindIds;
    const accIds = next?.accounts ?? accountIds;
    const cat = next?.category ?? categoryId;
    const tag = next?.tag ?? tagId;
    const note = next?.note ?? noteContains;
    const list = await listEntries({
      fromDate: f || null,
      toDate: to || null,
      kindIds: ks,
      accountIds: accIds,
      categoryId: cat || null,
      tagId: tag || null,
      noteContains: note || null,
    });
    setRows(list);
    setLoaded(true);
    setApplied({ from: f, to, kinds: ks, accounts: accIds, category: cat, tag, note });
    if (next?.select) setSelected(next.select);
  }

  useEffect(() => {
    if (jump) return;
    void load().catch(() => setLoaded(true));
    // initial
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (!jump) return;
    setFromDate(jump.fromDate);
    setToDate(jump.toDate);
    setKindIds(jump.kindIds);
    void load({ from: jump.fromDate, to: jump.toDate, kinds: jump.kindIds, select: jump.entryId }).catch(
      () => setLoaded(true),
    );
    onJumpConsumed();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [jump]);

  useEffect(() => {
    if (!selectedRow) {
      setForm(null);
      return;
    }
    const tagNames = selectedRow.tagIds
      .map((id) => tags.find((tg) => tg.id === id)?.name)
      .filter((n): n is string => Boolean(n));
    setForm(
      formFromWrite(
        {
          kindId: selectedRow.kindId,
          amountMinor: selectedRow.amountMinor,
          occurredAt: selectedRow.occurredAt,
          accountId: selectedRow.accountId,
          counterAccountId: selectedRow.counterAccountId,
          counterAmountMinor: selectedRow.counterAmountMinor,
          categoryId: selectedRow.categoryId,
          feeCategoryId: selectedRow.feeCategoryId,
          note: selectedRow.note,
          tagNames,
        },
        categories,
      ),
    );
    setError(null);
  }, [selectedRow, tags, categories]);

  const catById = useMemo(() => new Map(categories.map((c) => [c.id, c])), [categories]);
  const accById = useMemo(() => new Map(accounts.map((a) => [a.id, a])), [accounts]);

  async function saveInspector() {
    if (!selected || !form) return;
    const kind = kinds.find((k) => k.id === form.kindId);
    const { write, error: err } = toWrite(form, kind);
    if (!write) {
      setError(err ?? "error.amountInvalid");
      return;
    }
    try {
      await updateEntry(selected, write);
      await onChanged();
      await load();
      setError(null);
    } catch (ex) {
      setError(typeof ex === "object" && ex && "code" in ex ? String((ex as { code: string }).code) : "error.db");
    }
  }

  async function doDelete() {
    if (!selected) return;
    await deleteEntry(selected);
    setConfirm(false);
    setSelected(null);
    await onChanged();
    await load();
  }

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") setSelected(null);
      if (e.key === "Delete" && selected && !(e.target instanceof HTMLInputElement) && !(e.target instanceof HTMLTextAreaElement)) {
        setConfirm(true);
      }
      if (e.key === "Enter" && e.ctrlKey) void saveInspector();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  const emptyAll = loaded && rows.length === 0 && !appliedActive(applied);

  return (
    <div>
      <h1>{t("nav.ledger")}</h1>
      <div className="filters">
        <div className="field">
          <label>{t("ledger.from")}</label>
          <input type="date" value={fromDate} onChange={(e) => setFromDate(e.target.value)} />
        </div>
        <div className="field">
          <label>{t("ledger.to")}</label>
          <input type="date" value={toDate} onChange={(e) => setToDate(e.target.value)} />
        </div>
        <div className="field">
          <label>{t("ledger.kind")}</label>
          <select
            value={kindIds[0] ?? ""}
            onChange={(e) => setKindIds(e.target.value ? [e.target.value] : [])}
          >
            <option value="">{t("ledger.allKinds")}</option>
            {kinds.map((k) => (
              <option key={k.id} value={k.id}>
                {t(k.labelKey)}
              </option>
            ))}
          </select>
        </div>
        <div className="field">
          <label>{t("ledger.account")}</label>
          <select
            value={accountIds[0] ?? ""}
            onChange={(e) => setAccountIds(e.target.value ? [e.target.value] : [])}
          >
            <option value="">{t("ledger.allAccounts")}</option>
            {accounts.map((a) => (
              <option key={a.id} value={a.id}>
                {accountName(a, t)}
              </option>
            ))}
          </select>
        </div>
        <div className="field">
          <label>{t("ledger.category")}</label>
          <select value={categoryId} onChange={(e) => setCategoryId(e.target.value)}>
            <option value="">{t("ledger.allCategories")}</option>
            {mains(categories).map((m) => (
              <optgroup key={m.id} label={categoryName(m, t)}>
                <option value={m.id}>{categoryName(m, t)}</option>
                {subsOf(categories, m.id).map((s) => (
                  <option key={s.id} value={s.id}>
                    {categoryName(s, t)}
                  </option>
                ))}
              </optgroup>
            ))}
          </select>
        </div>
        <div className="field">
          <label>{t("ledger.tags")}</label>
          <select value={tagId} onChange={(e) => setTagId(e.target.value)}>
            <option value="">{t("ledger.allTags")}</option>
            {tags.map((tg) => (
              <option key={tg.id} value={tg.id}>
                {tg.name}
              </option>
            ))}
          </select>
        </div>
        <div className="field">
          <label>{t("ledger.noteContains")}</label>
          <input value={noteContains} onChange={(e) => setNoteContains(e.target.value)} />
        </div>
        <button type="button" className="btn" onClick={() => void load()}>
          {t("ledger.filters")}
        </button>
        <button
          type="button"
          className="btn"
          onClick={() => {
            const m = monthRange();
            setFromDate(m.from);
            setToDate(m.to);
            void load({ from: m.from, to: m.to });
          }}
        >
          {t("ledger.thisMonth")}
        </button>
        <button
          type="button"
          className="btn"
          onClick={() => {
            setFromDate("");
            setToDate("");
            setKindIds([]);
            setAccountIds([]);
            setCategoryId("");
            setTagId("");
            setNoteContains("");
            void load({
              from: "",
              to: "",
              kinds: [],
              accounts: [],
              category: "",
              tag: "",
              note: "",
            });
          }}
        >
          {t("ledger.all")}
        </button>
      </div>
      <div className="ledger-layout">
        <div className="surface" style={{ overflow: "auto" }}>
          {emptyAll && (
            <p className="muted" style={{ padding: 12 }}>
              {t("ledger.empty")}
            </p>
          )}
          {!emptyAll && loaded && rows.length === 0 && (
            <p className="muted" style={{ padding: 12 }}>
              {t("ledger.filterEmpty")}
            </p>
          )}
          {rows.length > 0 && (
            <table className="table">
              <thead>
                <tr>
                  <th>{t("ledger.time")}</th>
                  <th>{t("ledger.kind")}</th>
                  <th>{t("ledger.amount")}</th>
                  <th>{t("ledger.account")}</th>
                  <th>{t("ledger.category")}</th>
                  <th>{t("ledger.tags")}</th>
                  <th>{t("ledger.note")}</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((row) => {
                  const kind = kinds.find((k) => k.id === row.kindId);
                  const src = accById.get(row.accountId);
                  const dst = row.counterAccountId ? accById.get(row.counterAccountId) : undefined;
                  const cat = catById.get(row.categoryId);
                  const feeCat = row.feeCategoryId ? catById.get(row.feeCategoryId) : undefined;
                  const fee =
                    row.counterAmountMinor != null ? row.amountMinor - row.counterAmountMinor : 0;
                  return (
                    <tr
                      key={row.id}
                      className={selected === row.id ? "selected" : ""}
                      onClick={() => setSelected(row.id)}
                    >
                      <td className="mono">{formatLocalDateTime(row.occurredAt, locale)}</td>
                      <td>{kind ? t(kind.labelKey) : row.kindId}</td>
                      <td className={`num ${kindClass(row.kindId)}`}>
                        {formatAmount(row, locale)}
                        {fee > 0 && (
                          <div className="muted">
                            {formatMinor(row.counterAmountMinor ?? 0, locale)} / {formatMinor(fee, locale)}
                          </div>
                        )}
                      </td>
                      <td>
                        {row.counterAccountId
                          ? `${accountName(src, t)} → ${accountName(dst, t)}`
                          : accountName(src, t)}
                      </td>
                      <td>
                        {categoryName(cat, t)}
                        {feeCat && fee > 0 ? ` / ${categoryName(feeCat, t)}` : ""}
                      </td>
                      <td>
                        {row.tagIds
                          .map((id) => tags.find((tg) => tg.id === id)?.name)
                          .filter(Boolean)
                          .join(", ")}
                      </td>
                      <td>{row.note?.slice(0, 40)}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          )}
        </div>
        {selectedRow && form && (
          <aside className="inspector surface">
            <EntryForm
              form={form}
              setForm={setForm}
              kinds={kinds}
              accounts={accounts}
              categories={categories}
              error={error}
            />
            <div className="row">
              <button type="button" className="btn primary" onClick={() => void saveInspector()}>
                {t("action.save")}
              </button>
              <button type="button" className="btn danger" onClick={() => setConfirm(true)}>
                {t("action.delete")}
              </button>
            </div>
          </aside>
        )}
      </div>
      {confirm && (
        <ConfirmDialog
          message={t("confirm.deleteEntry")}
          onCancel={() => setConfirm(false)}
          onConfirm={() => void doDelete()}
        />
      )}
    </div>
  );
}

type AppliedFilter = {
  from: string;
  to: string;
  kinds: string[];
  accounts: string[];
  category: string;
  tag: string;
  note: string;
};

type LoadOpts = Partial<AppliedFilter> & { select?: string };

const emptyApplied: AppliedFilter = {
  from: "",
  to: "",
  kinds: [],
  accounts: [],
  category: "",
  tag: "",
  note: "",
};

function appliedActive(f: AppliedFilter): boolean {
  return Boolean(
    f.from || f.to || f.kinds.length || f.accounts.length || f.category || f.tag || f.note.trim(),
  );
}

function kindClass(kindId: string): string {
  if (kindId === "income") return "in";
  if (kindId === "expense") return "out";
  return "neutral";
}

function formatAmount(row: EntryDto, locale: string): string {
  if (row.kindId === "income") return formatMinor(row.amountMinor, locale, true);
  if (row.kindId === "expense") return `−${formatMinor(row.amountMinor, locale)}`;
  if (row.kindId === "repayment" || row.kindId === "prepayment")
    return `−${formatMinor(row.amountMinor, locale)}`;
  return formatMinor(row.amountMinor, locale);
}
