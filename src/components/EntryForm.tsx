import { useMemo, type RefObject } from "react";
import { useTranslation } from "react-i18next";
import type { AccountDto, CategoryDto, EntryWrite, KindDto } from "../lib/types";
import { parseCny } from "../lib/money";
import { accountName, categoryName, mains, subsOf } from "../lib/names";
import { fromUtcIso, localParts, toUtcIso } from "../lib/time";
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

export function emptyForm(
  kinds: KindDto[],
  accounts: AccountDto[],
  categories: CategoryDto[],
  defaultAccountId: string,
  defaultFeeCategoryId: string | null,
  kindId?: string,
): FormState {
  const now = localParts();
  const kind = kindId ?? kinds[0]?.id ?? "expense";
  const mainList = mains(categories);
  const main = mainList[0];
  const subs = main ? subsOf(categories, main.id) : [];
  return {
    kindId: kind,
    amount: "",
    destAmount: "",
    ...now,
    accountId: defaultAccountId || accounts[0]?.id || "",
    counterAccountId: accounts.find((a) => a.id !== defaultAccountId)?.id ?? accounts[0]?.id ?? "",
    mainId: main?.id ?? "",
    categoryId: subs[0]?.id ?? "",
    feeCategoryId: defaultFeeCategoryId ?? "",
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

export function toWrite(form: FormState, kind: KindDto | undefined): { write?: EntryWrite; error?: string } {
  const amountMinor = parseCny(form.amount);
  if (amountMinor == null || amountMinor <= 0) return { error: "error.amountInvalid" };
  const occurredAt = toUtcIso(form);
  if (!kind?.counterpartyRequired) {
    return {
      write: {
        kindId: form.kindId,
        amountMinor,
        occurredAt,
        accountId: form.accountId,
        counterAccountId: null,
        counterAmountMinor: null,
        categoryId: form.categoryId,
        feeCategoryId: null,
        note: form.note.trim() || null,
        tagNames: form.tags,
      },
    };
  }
  const dest = parseCny(form.destAmount || form.amount);
  if (dest == null || dest <= 0) return { error: "error.counterAmountInvalid" };
  const fee = amountMinor - dest;
  return {
    write: {
      kindId: form.kindId,
      amountMinor,
      occurredAt,
      accountId: form.accountId,
      counterAccountId: form.counterAccountId,
      counterAmountMinor: dest,
      categoryId: form.categoryId,
      feeCategoryId: fee > 0 ? form.feeCategoryId || null : null,
      note: form.note.trim() || null,
      tagNames: form.tags,
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
}: {
  form: FormState;
  setForm: (next: FormState) => void;
  kinds: KindDto[];
  accounts: AccountDto[];
  categories: CategoryDto[];
  error: string | null;
  amountRef?: RefObject<HTMLInputElement | null>;
}) {
  const { t } = useTranslation();
  const kind = kinds.find((k) => k.id === form.kindId);
  const mainList = mains(categories);
  const subList = form.mainId ? subsOf(categories, form.mainId) : [];
  const feeMains = mainList;
  const srcAmt = parseCny(form.amount);
  const dstAmt = parseCny(form.destAmount || form.amount);
  const fee = srcAmt != null && dstAmt != null ? srcAmt - dstAmt : 0;

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
      <div className="field">
        <label>{kind?.counterpartyRequired ? t("field.sourceAmount") : t("field.amount")}</label>
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
                kind?.counterpartyRequired && (!form.destAmount || form.destAmount === form.amount)
                  ? amount
                  : form.destAmount,
            });
          }}
        />
      </div>
      {kind?.counterpartyRequired && (
        <>
          <div className="field">
            <label>{t("field.destAmount")}</label>
            <input
              className="amount-input mono"
              value={form.destAmount}
              onChange={(e) => setForm({ ...form, destAmount: e.target.value })}
            />
          </div>
          {fee > 0 && (
            <p className="muted out mono">
              {t("field.fee")}: {(fee / 100).toFixed(2)}
            </p>
          )}
        </>
      )}
      <div className="row">
        <div className="field" style={{ flex: 1 }}>
          <label>{t("field.occurredAt")}</label>
          <div className="row">
            <input
              type="number"
              value={form.year}
              onChange={(e) => setForm({ ...form, year: Number(e.target.value) })}
            />
            <input
              type="number"
              value={form.month}
              onChange={(e) => setForm({ ...form, month: Number(e.target.value) })}
            />
            <input
              type="number"
              value={form.day}
              onChange={(e) => setForm({ ...form, day: Number(e.target.value) })}
            />
            <input
              type="number"
              value={form.hour}
              onChange={(e) => setForm({ ...form, hour: Number(e.target.value) })}
            />
            <input
              type="number"
              value={form.minute}
              onChange={(e) => setForm({ ...form, minute: Number(e.target.value) })}
            />
          </div>
        </div>
      </div>
      <div className="field">
        <label>{t("field.account")}</label>
        <select
          value={form.accountId}
          onChange={(e) => setForm({ ...form, accountId: e.target.value })}
        >
          {accounts.map((a) => (
            <option key={a.id} value={a.id}>
              {accountName(a, t)}
            </option>
          ))}
        </select>
      </div>
      {kind?.counterpartyRequired && (
        <div className="field">
          <label>{t("field.destAccount")}</label>
          <select
            value={form.counterAccountId}
            onChange={(e) => setForm({ ...form, counterAccountId: e.target.value })}
          >
            {accounts.map((a) => (
              <option key={a.id} value={a.id}>
                {accountName(a, t)}
              </option>
            ))}
          </select>
        </div>
      )}
      <div className="row">
        <div className="field" style={{ flex: 1 }}>
          <label>{t("field.mainCategory")}</label>
          <select
            value={form.mainId}
            onChange={(e) => {
              const mainId = e.target.value;
              const first = subsOf(categories, mainId)[0];
              setForm({ ...form, mainId, categoryId: first?.id ?? "" });
            }}
          >
            {mainList.map((m) => (
              <option key={m.id} value={m.id}>
                {categoryName(m, t)}
              </option>
            ))}
          </select>
        </div>
        <div className="field" style={{ flex: 1 }}>
          <label>{t("field.subCategory")}</label>
          <select
            value={form.categoryId}
            onChange={(e) => setForm({ ...form, categoryId: e.target.value })}
          >
            {subList.map((s) => (
              <option key={s.id} value={s.id}>
                {categoryName(s, t)}
              </option>
            ))}
          </select>
        </div>
      </div>
      {kind?.counterpartyRequired && fee > 0 && (
        <div className="row">
          <div className="field" style={{ flex: 1 }}>
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
          <div className="field" style={{ flex: 1 }}>
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
