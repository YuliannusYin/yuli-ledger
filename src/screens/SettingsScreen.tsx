import { useState } from "react";
import { useTranslation } from "react-i18next";
import { save } from "@tauri-apps/plugin-dialog";
import type { AccountDto, CategoryDto, SettingsDto } from "../lib/types";
import { exportBackupJson, exportEntriesCsv, updateSettings } from "../lib/api";
import { accountName, categoryName, mains, subsOf } from "../lib/names";
import en from "../i18n/en";

export default function SettingsScreen({
  settings,
  accounts,
  categories,
  dbPath,
  onChanged,
}: {
  settings: SettingsDto;
  accounts: AccountDto[];
  categories: CategoryDto[];
  dbPath: string;
  onChanged: () => Promise<void>;
}) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);
  const [csvFrom, setCsvFrom] = useState("");
  const [csvTo, setCsvTo] = useState("");
  const [exportMsg, setExportMsg] = useState<string | null>(null);
  const [exportErr, setExportErr] = useState<string | null>(null);

  async function patch(partial: Partial<SettingsDto>) {
    await updateSettings({ ...settings, ...partial });
    await onChanged();
  }

  function labels(): Record<string, string> {
    return Object.fromEntries(Object.keys(en).map((k) => [k, t(k)]));
  }

  async function exportCsv() {
    setExportMsg(null);
    setExportErr(null);
    const path = await save({
      defaultPath: "yuli-ledger-entries.csv",
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    if (!path) return;
    try {
      await exportEntriesCsv({
        path,
        fromDate: csvFrom.trim() || null,
        toDate: csvTo.trim() || null,
        labels: labels(),
      });
      setExportMsg("settings.export.ok");
    } catch (ex) {
      setExportErr(
        typeof ex === "object" && ex && "code" in ex ? String((ex as { code: string }).code) : "error.io",
      );
    }
  }

  async function exportJson() {
    setExportMsg(null);
    setExportErr(null);
    const path = await save({
      defaultPath: "yuli-ledger-backup.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!path) return;
    try {
      await exportBackupJson(path);
      setExportMsg("settings.export.ok");
    } catch (ex) {
      setExportErr(
        typeof ex === "object" && ex && "code" in ex ? String((ex as { code: string }).code) : "error.io",
      );
    }
  }

  return (
    <div className="surface" style={{ maxWidth: 560, padding: 16 }}>
      <h1>{t("nav.settings")}</h1>
      <div className="field">
        <label>{t("settings.language")}</label>
        <select
          value={settings.uiLanguage ?? ""}
          onChange={(e) => void patch({ uiLanguage: e.target.value || null })}
        >
          <option value="">{t("settings.language.system")}</option>
          <option value="en">English</option>
          <option value="zh-Hans">简体中文</option>
        </select>
      </div>
      <div className="field">
        <label>{t("settings.colorScheme")}</label>
        <select
          value={settings.colorScheme ?? "system"}
          onChange={(e) => void patch({ colorScheme: e.target.value })}
        >
          <option value="system">{t("settings.colorScheme.system")}</option>
          <option value="light">{t("settings.colorScheme.light")}</option>
          <option value="dark">{t("settings.colorScheme.dark")}</option>
        </select>
      </div>
      <div className="field">
        <label>{t("settings.defaultAccount")}</label>
        <select
          value={settings.defaultAccountId}
          onChange={(e) => void patch({ defaultAccountId: e.target.value })}
        >
          {accounts.map((a) => (
            <option key={a.id} value={a.id}>
              {accountName(a, t)}
            </option>
          ))}
        </select>
      </div>
      <div className="field">
        <label>{t("settings.defaultFeeCategory")}</label>
        <select
          value={settings.defaultFeeCategoryId ?? ""}
          onChange={(e) => void patch({ defaultFeeCategoryId: e.target.value || null })}
        >
          <option value="">{t("settings.none")}</option>
          {mains(categories).map((m) =>
            subsOf(categories, m.id).map((s) => (
              <option key={s.id} value={s.id}>
                {categoryName(m, t)} / {categoryName(s, t)}
              </option>
            )),
          )}
        </select>
      </div>
      <div className="field">
        <label>{t("settings.dbPath")}</label>
        <div className="row">
          <input readOnly value={dbPath} className="mono" />
          <button
            type="button"
            className="btn"
            onClick={() => {
              void navigator.clipboard.writeText(dbPath);
              setCopied(true);
              setTimeout(() => setCopied(false), 1200);
            }}
          >
            {copied ? t("action.copied") : t("action.copy")}
          </button>
        </div>
      </div>
      <h2>{t("settings.export")}</h2>
      <div className="field">
        <label>{t("settings.export.csvRange")}</label>
        <div className="row">
          <input type="date" value={csvFrom} onChange={(e) => setCsvFrom(e.target.value)} />
          <input type="date" value={csvTo} onChange={(e) => setCsvTo(e.target.value)} />
        </div>
        <p className="muted">{t("settings.export.csvHint")}</p>
      </div>
      <div className="row">
        <button type="button" className="btn" onClick={() => void exportCsv()}>
          {t("settings.export.csv")}
        </button>
        <button type="button" className="btn" onClick={() => void exportJson()}>
          {t("settings.export.json")}
        </button>
      </div>
      {exportMsg && <p className="muted">{t(exportMsg)}</p>}
      {exportErr && <p className="err">{t(exportErr)}</p>}
    </div>
  );
}
