import { useRef, useState, type FormEvent, type RefObject } from "react";
import { useTranslation } from "react-i18next";
import type { AccountDto, CategoryDto, KindDto, SettingsDto } from "../lib/types";
import { createEntry, updateSettings } from "../lib/api";
import { toUtcIso } from "../lib/time";
import EntryForm, { emptyForm, toWrite, type FormState } from "../components/EntryForm";

export default function RecordScreen({
  kinds,
  accounts,
  categories,
  settings,
  amountFocusRef,
  onSaved,
}: {
  kinds: KindDto[];
  accounts: AccountDto[];
  categories: CategoryDto[];
  settings: SettingsDto;
  amountFocusRef: RefObject<HTMLInputElement | null>;
  onSaved: () => Promise<void>;
}) {
  const { t } = useTranslation();
  const [form, setForm] = useState<FormState>(() => emptyForm(kinds, accounts, categories, settings));
  const [error, setError] = useState<string | null>(null);
  const saving = useRef(false);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    const kind = kinds.find((k) => k.id === form.kindId);
    const { write, error: err } = toWrite(form, kind);
    if (!write) {
      setError(err ?? "error.amountInvalid");
      return;
    }
    if (saving.current) return;
    saving.current = true;
    try {
      await createEntry(write);
      await updateSettings({
        ...settings,
        lastKindId: form.kindId,
        lastAccountId: form.accountId || null,
        lastCounterAccountId: form.counterAccountId || null,
        lastCategoryId: form.categoryId || null,
        lastFeeCategoryId: form.feeCategoryId || null,
        lastOccurredAt: toUtcIso(form),
      });
      setForm({
        ...form,
        amount: "",
        destAmount: "",
        tags: [],
        note: "",
      });
      setError(null);
      await onSaved();
      amountFocusRef.current?.focus();
    } catch (ex) {
      const code = typeof ex === "object" && ex && "code" in ex ? String((ex as { code: string }).code) : "error.db";
      setError(code);
    } finally {
      saving.current = false;
    }
  }

  return (
    <form className="surface" style={{ maxWidth: 720, padding: 16 }} onSubmit={(e) => void onSubmit(e)}>
      <h1>{t("nav.record")}</h1>
      <EntryForm
        form={form}
        setForm={setForm}
        kinds={kinds}
        accounts={accounts}
        categories={categories}
        error={error}
        amountRef={amountFocusRef}
      />
      <button type="submit" className="btn primary btn-block">
        {t("action.save")}
      </button>
    </form>
  );
}
