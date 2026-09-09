import { useRef, useState, type FormEvent, type RefObject } from "react";
import { useTranslation } from "react-i18next";
import type { AccountDto, CategoryDto, KindDto, SettingsDto } from "../lib/types";
import { createEntry } from "../lib/api";
import { localParts } from "../lib/time";
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
  const [form, setForm] = useState<FormState>(() =>
    emptyForm(kinds, accounts, categories, settings.defaultAccountId, settings.defaultFeeCategoryId),
  );
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
      const now = localParts();
      setForm({
        ...form,
        amount: "",
        destAmount: "",
        tags: [],
        note: "",
        ...now,
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
    <form className="surface" style={{ maxWidth: 560, padding: 16 }} onSubmit={(e) => void onSubmit(e)}>
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
      <button type="submit" className="btn primary">
        {t("action.save")}
      </button>
    </form>
  );
}
