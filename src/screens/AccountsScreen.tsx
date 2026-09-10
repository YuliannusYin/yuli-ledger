import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type { AccountDto, SettingsDto } from "../lib/types";
import {
  accountUsage,
  createAccount,
  deleteAccount,
  reorderAccounts,
  updateAccount,
  updateSettings,
} from "../lib/api";
import { formatMinor, parseCny } from "../lib/money";
import { accountName } from "../lib/names";
import { fromUtcIso, localParts, toUtcIso } from "../lib/time";
import ConfirmDialog from "../components/ConfirmDialog";
import DragList from "../components/DragList";

const KINDS = ["cash", "bank", "ewallet", "credit", "other"];

export default function AccountsScreen({
  accounts,
  settings,
  locale,
  onChanged,
}: {
  accounts: AccountDto[];
  settings: SettingsDto;
  locale: string;
  onChanged: () => Promise<void>;
}) {
  const { t } = useTranslation();
  const [selected, setSelected] = useState<string | null>(accounts[0]?.id ?? null);
  const acc = accounts.find((a) => a.id === selected);
  const [name, setName] = useState("");
  const [kind, setKind] = useState("other");
  const [opening, setOpening] = useState("0.00");
  const [openingDebt, setOpeningDebt] = useState("0.00");
  const [openingAt, setOpeningAt] = useState(toUtcIso({ ...localParts(), hour: 0, minute: 0 }));
  const [note, setNote] = useState("");
  const [usage, setUsage] = useState(0);
  const [confirm, setConfirm] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!acc) return;
    setName(accountName(acc, t));
    setKind(acc.accountKind);
    setOpening((acc.openingBalanceMinor / 100).toFixed(2));
    setOpeningDebt((acc.openingDebtMinor / 100).toFixed(2));
    setOpeningAt(acc.openingAt);
    setNote(acc.note ?? "");
    void accountUsage(acc.id).then(setUsage);
    setError(null);
  }, [acc, t]);

  async function save() {
    if (!acc) return;
    const minor = parseCny(opening);
    const debtMinor = parseCny(openingDebt);
    if (minor == null || debtMinor == null) {
      setError("error.amountInvalid");
      return;
    }
    const parts = fromUtcIso(openingAt);
    try {
      await updateAccount(acc.id, {
        name,
        accountKind: kind,
        openingBalanceMinor: minor,
        openingDebtMinor: debtMinor,
        openingAt: toUtcIso(parts),
        note: note.trim() || null,
      });
      await onChanged();
    } catch (ex) {
      setError(typeof ex === "object" && ex && "code" in ex ? String((ex as { code: string }).code) : "error.db");
    }
  }

  async function create() {
    const created = await createAccount({
      name: t("accounts.newName"),
      accountKind: "other",
      openingBalanceMinor: 0,
      openingDebtMinor: 0,
      openingAt: toUtcIso({ ...localParts(), hour: 0, minute: 0 }),
      note: null,
    });
    await onChanged();
    setSelected(created.id);
  }

  const canDelete =
    Boolean(acc) && usage === 0 && accounts.length > 1 && settings.defaultAccountId !== acc?.id;

  return (
    <div>
      <div className="toolbar">
        <h1 style={{ margin: 0 }}>{t("nav.accounts")}</h1>
        <button type="button" className="btn" onClick={() => void create()}>
          {t("action.addAccount")}
        </button>
      </div>
      <div className="split">
        <DragList
          items={accounts}
          selectedId={selected}
          onSelect={setSelected}
          onReorder={(ids) => void reorderAccounts(ids).then(onChanged)}
          render={(a) => (
            <>
              <span>{accountName(a, t)}</span>
              <span className="mono muted">
                {formatMinor(a.balanceMinor, locale, true)}
                {" · "}
                {t("accounts.debt")} {formatMinor(a.debtMinor, locale, true)}
              </span>
            </>
          )}
        />
        {acc && (
          <div className="surface" style={{ padding: 16 }}>
            <div className="field">
              <label>{t("field.name")}</label>
              <input value={name} onChange={(e) => setName(e.target.value)} />
            </div>
            <div className="field">
              <label>{t("field.accountKind")}</label>
              <select value={kind} onChange={(e) => setKind(e.target.value)}>
                {KINDS.map((k) => (
                  <option key={k} value={k}>
                    {t(`accountKind.${k}`)}
                  </option>
                ))}
              </select>
            </div>
            <div className="field">
              <label>{t("field.openingBalance")}</label>
              <input className="mono" value={opening} onChange={(e) => setOpening(e.target.value)} />
            </div>
            <div className="field">
              <label>{t("field.openingDebt")}</label>
              <input className="mono" value={openingDebt} onChange={(e) => setOpeningDebt(e.target.value)} />
            </div>
            <div className="field">
              <label>{t("field.openingAt")}</label>
              <input
                type="datetime-local"
                value={datetimeLocal(openingAt)}
                onChange={(e) => setOpeningAt(fromDatetimeLocal(e.target.value))}
              />
            </div>
            <div className="field">
              <label>{t("field.accountNote")}</label>
              <textarea value={note} onChange={(e) => setNote(e.target.value)} />
            </div>
            <p className="muted">
              {t("accounts.balance")}: <span className="mono">{formatMinor(acc.balanceMinor, locale, true)}</span>
              {" · "}
              {t("accounts.debt")}: <span className="mono">{formatMinor(acc.debtMinor, locale, true)}</span>
              {settings.defaultAccountId === acc.id ? ` · ${t("accounts.default")}` : ""}
            </p>
            {error && <p className="err">{t(error)}</p>}
            <div className="row">
              <button type="button" className="btn primary" onClick={() => void save()}>
                {t("action.save")}
              </button>
              {settings.defaultAccountId !== acc.id && (
                <button
                  type="button"
                  className="btn"
                  onClick={() =>
                    void updateSettings({ ...settings, defaultAccountId: acc.id }).then(onChanged)
                  }
                >
                  {t("action.setDefault")}
                </button>
              )}
              <button
                type="button"
                className="btn danger"
                disabled={!canDelete}
                onClick={() => setConfirm(true)}
              >
                {t("action.delete")}
              </button>
            </div>
            {!canDelete && (
              <p className="reason">
                {usage > 0
                  ? t("error.accountInUse", { count: usage })
                  : settings.defaultAccountId === acc.id
                    ? t("error.defaultAccount")
                    : t("error.lastAccount")}
              </p>
            )}
          </div>
        )}
      </div>
      {confirm && acc && (
        <ConfirmDialog
          message={t("confirm.deleteAccount")}
          onCancel={() => setConfirm(false)}
          onConfirm={() =>
            void deleteAccount(acc.id).then(async () => {
              setConfirm(false);
              setSelected(accounts.find((a) => a.id !== acc.id)?.id ?? null);
              await onChanged();
            })
          }
        />
      )}
    </div>
  );
}

function datetimeLocal(iso: string): string {
  const p = fromUtcIso(iso);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${p.year}-${pad(p.month)}-${pad(p.day)}T${pad(p.hour)}:${pad(p.minute)}`;
}

function fromDatetimeLocal(value: string): string {
  const [date, time] = value.split("T");
  const [y, m, d] = date.split("-").map(Number);
  const [h, min] = (time || "00:00").split(":").map(Number);
  return toUtcIso({ year: y, month: m, day: d, hour: h, minute: min });
}
