import { useMemo, type RefObject } from "react";
import { useTranslation } from "react-i18next";
import type { AccountDto, CategoryDto, EntryWrite, KindDto, PendingEntryDto, PendingEntryWrite, SettingsDto } from "../lib/types";
import { parseCny } from "../lib/money";
import { accountName, categoryName, mains, subsOf } from "../lib/names";
import {
  applyDateInput,
  applyTimeInput,
  fromUtcIso,
  localParts,
  partsToDateInput,
  partsToTimeInput,
  toUtcIso,
} from "../lib/time";
import TagInput from "./TagInput";

export type FormState = {
  kindId: string;
  amount: string;
  destAmount: string;
  year: number;
  month: number;
  day: number;
  hour: number;
  minute: number;
  accountId: string;
  counterAccountId: string;
  mainId: string;
  categoryId: string;
  feeCategoryId: string;
  tags: string[];
  note: string;
};

function knownId(ids: string[], value: string | null | undefined, fallback: string): string {
  if (value && ids.includes(value)) return value;
  return fallback;
}

function draftTime(iso: string | null | undefined) {
  if (!iso) return localParts();
  const parts = fromUtcIso(iso);
  if (!Number.isFinite(parts.year) || parts.year < 1970) return localParts();
  return parts;
}

export function emptyForm(
  kinds: KindDto[],
  accounts: AccountDto[],
  categories: CategoryDto[],
  settings: SettingsDto,
  kindId?: string,
): FormState {
  const accountIds = accounts.map((a) => a.id);
  const kind = kindId ?? knownId(kinds.map((k) => k.id), settings.lastKindId, kinds[0]?.id ?? "expense");
  const accountId = knownId(
    accountIds,
    settings.lastAccountId,
    settings.defaultAccountId || accounts[0]?.id || "",
  );
  const counterFallback = accounts.find((a) => a.id !== accountId)?.id ?? accounts[0]?.id ?? "";
  const lastCat = categories.find((c) => c.id === settings.lastCategoryId && c.parentId);
  const mainList = mains(categories);
  const main = lastCat
    ? categories.find((c) => c.id === lastCat.parentId) ?? mainList[0]
    : mainList[0];
  const subs = main ? subsOf(categories, main.id) : [];
  const categoryId = lastCat?.id ?? subs[0]?.id ?? "";
  const feeId = knownId(
    categories.filter((c) => c.parentId).map((c) => c.id),
    settings.lastFeeCategoryId ?? settings.defaultFeeCategoryId,
    settings.defaultFeeCategoryId ?? "",
  );
  return {
    kindId: kind,
    amount: "",
    destAmount: "",
    ...draftTime(settings.lastOccurredAt),
    accountId,
    counterAccountId: knownId(accountIds, settings.lastCounterAccountId, counterFallback),
    mainId: main?.id ?? "",
    categoryId,
    feeCategoryId: feeId,
    tags: [],
    note: "",
  };
}

export function formFromWrite(
  write: EntryWrite,
  categories: CategoryDto[],
): FormState {
  const time = fromUtcIso(write.occurredAt);
  const cat = categories.find((c) => c.id === write.categoryId);
  return {
    kindId: write.kindId,
    amount: write.amountMinor ? (write.amountMinor / 100).toFixed(2) : "",
    destAmount: write.counterAmountMinor != null ? (write.counterAmountMinor / 100).toFixed(2) : "",
    ...time,
    accountId: write.accountId,
    counterAccountId: write.counterAccountId ?? "",
    mainId: cat?.parentId ?? "",
    categoryId: write.categoryId,
    feeCategoryId: write.feeCategoryId ?? "",
    tags: write.tagNames,
    note: write.note ?? "",
  };
}

export function formFromPending(
  pending: PendingEntryDto,
  categories: CategoryDto[],
  tagNames: string[],
): FormState {
  return formFromWrite(
    {
      kindId: pending.kindId ?? "",
      amountMinor: pending.amountMinor,
      occurredAt: pending.occurredAt,
      accountId: pending.accountId ?? "",
      counterAccountId: pending.counterAccountId,
      counterAmountMinor: pending.counterAmountMinor,
      categoryId: pending.categoryId ?? "",
      feeCategoryId: pending.feeCategoryId,
      note: pending.note,
      tagNames,
    },
    categories,
  );
}

export function toPendingWrite(form: FormState, kind: KindDto | undefined): { write?: PendingEntryWrite; error?: string } {
  const amountMinor = parseCny(form.amount);
  if (amountMinor == null || amountMinor <= 0) return { error: "error.amountInvalid" };
  const occurredAt = toUtcIso(form);
  let counterAmountMinor: number | null = null;
  let feeCategoryId: string | null = null;
  if (kind?.counterAmountRequired) {
    const dest = parseCny(form.destAmount || form.amount);
    counterAmountMinor = dest != null && dest > 0 ? dest : null;
    if (counterAmountMinor != null && amountMinor - counterAmountMinor > 0) {
      feeCategoryId = form.feeCategoryId || null;
    }
  }
  return {
    write: {
      amountMinor,
      occurredAt,
      kindId: form.kindId || null,
      accountId: form.accountId || null,
      counterAccountId: kind?.counterAccountRequired ? form.counterAccountId || null : null,
      counterAmountMinor: kind?.counterAmountRequired ? counterAmountMinor : null,
      categoryId: form.categoryId || null,
      feeCategoryId: kind?.counterAmountRequired ? feeCategoryId : null,
      note: form.note.trim() || null,
      tagNames: form.tags,
    },
  };
}

export function toWrite(form: FormState, kind: KindDto | undefined): { write?: EntryWrite; error?: string } {
  if (!form.kindId) return { error: "error.kindRequired" };
  if (!form.accountId) return { error: "error.accountRequired" };
  if (!form.categoryId) return { error: "error.categoryRequired" };
  const amountMinor = parseCny(form.amount);
  if (amountMinor == null || amountMinor <= 0) return { error: "error.amountInvalid" };
  const occurredAt = toUtcIso(form);
  const base = {
    kindId: form.kindId,
    amountMinor,
    occurredAt,
    accountId: form.accountId,
    categoryId: form.categoryId,
    note: form.note.trim() || null,
    tagNames: form.tags,
  };
  if (kind?.counterAmountRequired) {
    const dest = parseCny(form.destAmount || form.amount);
    if (dest == null || dest <= 0) return { error: "error.counterAmountInvalid" };
    const fee = amountMinor - dest;
    return {
      write: {
        ...base,
        counterAccountId: form.counterAccountId,
        counterAmountMinor: dest,
        feeCategoryId: fee > 0 ? form.feeCategoryId || null : null,
      },
    };
  }
  if (kind?.counterAccountRequired) {
    if (!form.counterAccountId) return { error: "error.counterAccountRequired" };
    if (kind.counterAccountsMustDiffer && form.counterAccountId === form.accountId) {
      return { error: "error.accountsMustDiffer" };
    }
    return {
      write: {
        ...base,
        counterAccountId: form.counterAccountId,
        counterAmountMinor: null,
        feeCategoryId: null,
      },
    };
  }
  return {
    write: {
      ...base,
      counterAccountId: null,
      counterAmountMinor: null,
      feeCategoryId: null,
    },
  };
}

export default function EntryForm({
  form,
  setForm,
  kinds,
  accounts,
  categories,
  error,
  amountRef,
  allowEmpty = false,
}: {
  form: FormState;
  setForm: (next: FormState) => void;
  kinds: KindDto[];
  accounts: AccountDto[];
  categories: CategoryDto[];
  error: string | null;
  amountRef?: RefObject<HTMLInputElement | null>;
  allowEmpty?: boolean;
}) {
  const { t } = useTranslation();
  const kind = kinds.find((k) => k.id === form.kindId);
  const mainList = mains(categories);
  const subList = form.mainId ? subsOf(categories, form.mainId) : [];
  const feeMains = mainList;
  const srcAmt = parseCny(form.amount);
  const dstAmt = parseCny(form.destAmount || form.amount);
  const fee = srcAmt != null && dstAmt != null ? srcAmt - dstAmt : 0;
  const accountLabel = kind?.primaryAccountLabelKey ? t(kind.primaryAccountLabelKey) : t("field.account");
  const counterLabel = kind?.counterAccountLabelKey ? t(kind.counterAccountLabelKey) : t("field.destAccount");

  const feeSubs = useMemo(() => {
    const feeCat = categories.find((c) => c.id === form.feeCategoryId);
    const parent = feeCat?.parentId ?? feeMains[0]?.id;
    return parent ? subsOf(categories, parent) : [];
  }, [categories, form.feeCategoryId, feeMains]);

  const feeMainId =
    categories.find((c) => c.id === form.feeCategoryId)?.parentId ?? feeMains[0]?.id ?? "";

  return (
    <>
      <div className="kind-row">
        {kinds.map((k) => (
          <button
            key={k.id}
            type="button"
            className={`kind-btn ${form.kindId === k.id ? "active" : ""}`}
            onClick={() => setForm({ ...form, kindId: k.id })}
          >
            {t(k.labelKey)}
          </button>
        ))}
      </div>
      {kind?.hintKey && <p className="hint">{t(kind.hintKey)}</p>}
      <div className="row">
        <div className="field">
          <label>{kind?.counterAmountRequired ? t("field.sourceAmount") : t("field.amount")}</label>
          <input
            ref={amountRef}
            className="amount-input mono"
            value={form.amount}
            onChange={(e) => {
              const amount = e.target.value;
              setForm({
                ...form,
                amount,
                destAmount:
                  kind?.counterAmountRequired && (!form.destAmount || form.destAmount === form.amount)
                    ? amount
                    : form.destAmount,
              });
            }}
          />
        </div>
        {kind?.counterAmountRequired && (
          <div className="field">
            <label>{t("field.destAmount")}</label>
            <input
              className="amount-input mono"
              value={form.destAmount}
              onChange={(e) => setForm({ ...form, destAmount: e.target.value })}
            />
          </div>
        )}
      </div>
      {kind?.counterAmountRequired && fee > 0 && (
        <p className="muted out mono">
          {t("field.fee")}: {(fee / 100).toFixed(2)}
        </p>
      )}
      <div className="field">
        <label>{t("field.occurredAt")}</label>
        <div className="row">
          <input
            type="date"
            className="mono"
            value={partsToDateInput(form)}
            onChange={(e) => setForm({ ...form, ...applyDateInput(form, e.target.value) })}
          />
          <input
            type="time"
            className="mono"
            step={60}
            value={partsToTimeInput(form)}
            onChange={(e) => setForm({ ...form, ...applyTimeInput(form, e.target.value) })}
          />
          <button type="button" className="btn" onClick={() => setForm({ ...form, ...localParts() })}>
            {t("action.now")}
          </button>
        </div>
      </div>
      <div className="row">
        <div className="field">
          <label>{accountLabel}</label>
          <select
            value={form.accountId}
            onChange={(e) => setForm({ ...form, accountId: e.target.value })}
          >
            {allowEmpty && <option value="">{t("pending.unset")}</option>}
            {accounts.map((a) => (
              <option key={a.id} value={a.id}>
                {accountName(a, t)}
              </option>
            ))}
          </select>
        </div>
        {kind?.counterAccountRequired && (
          <div className="field">
            <label>{counterLabel}</label>
            <select
              value={form.counterAccountId}
              onChange={(e) => setForm({ ...form, counterAccountId: e.target.value })}
            >
              {allowEmpty && <option value="">{t("pending.unset")}</option>}
              {accounts.map((a) => (
                <option key={a.id} value={a.id}>
                  {accountName(a, t)}
                </option>
              ))}
            </select>
          </div>
        )}
      </div>
      <div className="row">
        <div className="field">
          <label>{t("field.mainCategory")}</label>
          <select
            value={form.mainId}
            onChange={(e) => {
              const mainId = e.target.value;
              const first = subsOf(categories, mainId)[0];
              setForm({ ...form, mainId, categoryId: first?.id ?? "" });
            }}
          >
            {allowEmpty && <option value="">{t("pending.unset")}</option>}
            {mainList.map((m) => (
              <option key={m.id} value={m.id}>
                {categoryName(m, t)}
              </option>
            ))}
          </select>
        </div>
        <div className="field">
          <label>{t("field.subCategory")}</label>
          <select
            value={form.categoryId}
            onChange={(e) => setForm({ ...form, categoryId: e.target.value })}
          >
            {allowEmpty && <option value="">{t("pending.unset")}</option>}
            {subList.map((s) => (
              <option key={s.id} value={s.id}>
                {categoryName(s, t)}
              </option>
            ))}
          </select>
        </div>
      </div>
      {kind?.counterAmountRequired && fee > 0 && (
        <div className="row">
          <div className="field">
            <label>{t("field.feeCategory")}</label>
            <select
              value={feeMainId}
              onChange={(e) => {
                const first = subsOf(categories, e.target.value)[0];
                setForm({ ...form, feeCategoryId: first?.id ?? "" });
              }}
            >
              {feeMains.map((m) => (
                <option key={m.id} value={m.id}>
                  {categoryName(m, t)}
                </option>
              ))}
            </select>
          </div>
          <div className="field">
            <label>&nbsp;</label>
            <select
              value={form.feeCategoryId}
              onChange={(e) => setForm({ ...form, feeCategoryId: e.target.value })}
            >
              {feeSubs.map((s) => (
                <option key={s.id} value={s.id}>
                  {categoryName(s, t)}
                </option>
              ))}
            </select>
          </div>
        </div>
      )}
      <div className="field">
        <label>{t("field.tags")}</label>
        <TagInput value={form.tags} onChange={(tags) => setForm({ ...form, tags })} />
      </div>
      <div className="field">
        <label>{t("field.note")}</label>
        <textarea value={form.note} onChange={(e) => setForm({ ...form, note: e.target.value })} />
      </div>
      {error && <p className="err">{t(error)}</p>}
    </>
  );
}
