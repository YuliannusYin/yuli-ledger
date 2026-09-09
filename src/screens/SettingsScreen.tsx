import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { AccountDto, CategoryDto, SettingsDto } from "../lib/types";
import { updateSettings } from "../lib/api";
import { accountName, categoryName, mains, subsOf } from "../lib/names";

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
  const subs = categories.filter((c) => c.parentId);

  async function patch(partial: Partial<SettingsDto>) {
    await updateSettings({ ...settings, ...partial });
    await onChanged();
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
      <span hidden>{subs.length}</span>
    </div>
  );
}
