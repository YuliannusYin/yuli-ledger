import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { open, save } from "@tauri-apps/plugin-dialog";
import type { AccountDto, CategoryDto, KindDto, PendingEntryDto, TagDto } from "../lib/types";
import {
  deletePendingEntry,
  importPendingCsv,
  listPendingEntries,
  postPendingEntry,
  updatePendingEntry,
  writePendingCsvTemplate,
} from "../lib/api";
import { formatMinor } from "../lib/money";
import { accountName, categoryName } from "../lib/names";
import { formatLocalDate, formatLocalTime, groupByLocalDate } from "../lib/time";
import ConfirmDialog from "../components/ConfirmDialog";
import EntryForm, {
  formFromPending,
  toPendingWrite,
  toWrite,
  type FormState,
} from "../components/EntryForm";

export default function PendingScreen({
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
  const [rows, setRows] = useState<PendingEntryDto[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [form, setForm] = useState<FormState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [confirm, setConfirm] = useState(false);
  const [loaded, setLoaded] = useState(false);

  const selectedRow = rows.find((r) => r.id === selected) ?? null;

  async function load(selectId?: string | null) {
    const list = await listPendingEntries();
    setRows(list);
    setLoaded(true);
    if (selectId !== undefined) {
      setSelected(selectId);
      return;
    }
    if (selected && !list.some((r) => r.id === selected)) {
      setSelected(list[0]?.id ?? null);
    } else if (!selected && list.length) {
      setSelected(list[0].id);
    }
  }

  useEffect(() => {
    void load().catch(() => setLoaded(true));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (!selectedRow) {
      setForm(null);
      return;
    }
    const tagNames = selectedRow.tagIds
      .map((id) => tags.find((tg) => tg.id === id)?.name)
      .filter((n): n is string => Boolean(n));
    setForm(formFromPending(selectedRow, categories, tagNames));
    setError(null);
  }, [selectedRow, tags, categories]);

  const catById = useMemo(() => new Map(categories.map((c) => [c.id, c])), [categories]);
  const accById = useMemo(() => new Map(accounts.map((a) => [a.id, a])), [accounts]);
  const dayGroups = useMemo(() => groupByLocalDate(rows), [rows]);

  async function saveDraft() {
    if (!selected || !form) return;
    const kind = kinds.find((k) => k.id === form.kindId);
    const { write, error: err } = toPendingWrite(form, kind);
    if (!write) {
      setError(err ?? "error.amountInvalid");
      return;
    }
    try {
      await updatePendingEntry(selected, write);
      await onChanged();
      await load(selected);
      setError(null);
      setStatus(null);
    } catch (ex) {
      setError(codeOf(ex));
    }
  }

  async function postRow() {
    if (!selected || !form) return;
    const kind = kinds.find((k) => k.id === form.kindId);
    const { write, error: err } = toWrite(form, kind);
    if (!write) {
      setError(err ?? "error.amountInvalid");
      return;
    }
    const draft = toPendingWrite(form, kind).write;
    if (draft) {
      try {
        await updatePendingEntry(selected, draft);
      } catch (ex) {
        setError(codeOf(ex));
        return;
      }
    }
    try {
      await postPendingEntry(selected);
      setSelected(null);
      setError(null);
      setStatus(null);
      await onChanged();
      await load(null);
    } catch (ex) {
      setError(codeOf(ex));
      await load(selected);
    }
  }

  async function doDiscard() {
    if (!selected) return;
    await deletePendingEntry(selected);
    setConfirm(false);
    setSelected(null);
    await onChanged();
    await load(null);
  }

  async function downloadTemplate() {
    const path = await save({
      defaultPath: "yuli-ledger-pending-template.csv",
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    if (!path) return;
    try {
      await writePendingCsvTemplate(path);
      setStatus(t("settings.export.ok"));
      setError(null);
    } catch (ex) {
      setError(codeOf(ex));
    }
  }

  async function importFile() {
    const picked = await open({
      multiple: false,
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    const path = Array.isArray(picked) ? picked[0] : picked;
    if (!path) return;
    try {
      const result = await importPendingCsv(path);
      setError(null);
      if (result.errors.length) {
        setStatus(
          t("pending.importPartial", { count: result.imported, errors: result.errors.length }),
        );
      } else {
        setStatus(t("pending.importOk", { count: result.imported }));
      }
      await onChanged();
      await load();
    } catch (ex) {
      setError(codeOf(ex));
    }
  }

  return (
    <div className="screen-fill">
      <h1>{t("nav.pending")}</h1>
      <div className="toolbar" style={{ marginBottom: 8 }}>
        <button type="button" className="btn" onClick={() => void downloadTemplate()}>
          {t("action.downloadTemplate")}
        </button>
        <button type="button" className="btn primary" onClick={() => void importFile()}>
          {t("action.import")}
        </button>
        <span className="muted">{t("pending.count", { count: rows.length })}</span>
      </div>
      {status && <p className="muted">{status}</p>}
      <div className="ledger-layout">
        <div className="surface">
          {loaded && rows.length === 0 && (
            <p className="muted" style={{ padding: 12 }}>
              {t("pending.empty")}
            </p>
          )}
          {rows.length > 0 && (
            <div className="ledger-days">
              <div className="ledger-cols ledger-head">
                <span>{t("ledger.time")}</span>
                <span>{t("ledger.kind")}</span>
                <span className="num">{t("ledger.amount")}</span>
                <span>{t("ledger.account")}</span>
                <span>{t("ledger.category")}</span>
                <span>{t("ledger.tags")}</span>
                <span>{t("ledger.note")}</span>
              </div>
              {dayGroups.map((group) => (
                <section key={group.date} className="day-card">
                  <header className="day-card-head">
                    <span>{formatLocalDate(group.date, locale)}</span>
                  </header>
                  {group.rows.map((row) => {
                    const kind = kinds.find((k) => k.id === row.kindId);
                    const src = row.accountId ? accById.get(row.accountId) : undefined;
                    const dst = row.counterAccountId ? accById.get(row.counterAccountId) : undefined;
                    const cat = row.categoryId ? catById.get(row.categoryId) : undefined;
                    return (
                      <div
                        key={row.id}
                        className={`ledger-cols ledger-row ${selected === row.id ? "selected" : ""}`}
                        onClick={() => setSelected(row.id)}
                      >
                        <span className="mono">{formatLocalTime(row.occurredAt)}</span>
                        <span>{kind ? t(kind.labelKey) : t("pending.unset")}</span>
                        <span className="num">{formatMinor(row.amountMinor, locale)}</span>
                        <span>
                          {row.accountId
                            ? row.counterAccountId && row.counterAccountId !== row.accountId
                              ? `${accountName(src, t)} → ${accountName(dst, t)}`
                              : accountName(src, t)
                            : t("pending.unset")}
                        </span>
                        <span>{row.categoryId ? categoryName(cat, t) : t("pending.unset")}</span>
                        <span>
                          {row.tagIds
                            .map((id) => tags.find((tg) => tg.id === id)?.name)
                            .filter(Boolean)
                            .join(", ")}
                        </span>
                        <span>{row.note?.slice(0, 40)}</span>
                      </div>
                    );
                  })}
                </section>
              ))}
            </div>
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
              allowEmpty
            />
            <div className="row">
              <button type="button" className="btn" onClick={() => void saveDraft()}>
                {t("action.saveDraft")}
              </button>
              <button type="button" className="btn primary" onClick={() => void postRow()}>
                {t("action.post")}
              </button>
              <button type="button" className="btn danger" onClick={() => setConfirm(true)}>
                {t("action.discard")}
              </button>
            </div>
          </aside>
        )}
      </div>
      {confirm && (
        <ConfirmDialog
          message={t("confirm.discardPending")}
          onCancel={() => setConfirm(false)}
          onConfirm={() => void doDiscard()}
        />
      )}
    </div>
  );
}

function codeOf(ex: unknown): string {
  return typeof ex === "object" && ex && "code" in ex ? String((ex as { code: string }).code) : "error.db";
}
