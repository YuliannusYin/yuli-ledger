import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import TitleBar from "./components/TitleBar";
import NavRail from "./components/NavRail";
import RecordScreen from "./screens/RecordScreen";
import LedgerScreen from "./screens/LedgerScreen";
import ReportsScreen from "./screens/ReportsScreen";
import AccountsScreen from "./screens/AccountsScreen";
import CategoriesScreen from "./screens/CategoriesScreen";
import SettingsScreen from "./screens/SettingsScreen";
import { getBootstrap } from "./lib/api";
import { SCREENS, type BootstrapDto, type LedgerJump, type ScreenId } from "./lib/types";
import i18n from "./i18n";

function resolvedTheme(scheme: string | null): "light" | "dark" {
  if (scheme === "light" || scheme === "dark") return scheme;
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

export default function App() {
  const { t } = useTranslation();
  const [data, setData] = useState<BootstrapDto | null>(null);
  const [screen, setScreen] = useState<ScreenId>("record");
  const [collapsed, setCollapsed] = useState(false);
  const [jump, setJump] = useState<LedgerJump | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const amountRef = useRef<HTMLInputElement>(null);

  const reload = useCallback(async () => {
    const boot = await getBootstrap();
    setData(boot);
    const lang = boot.resolvedLanguage === "zh-Hans" ? "zh-Hans" : "en";
    if (i18n.language !== lang) await i18n.changeLanguage(lang);
    document.documentElement.lang = lang === "zh-Hans" ? "zh-Hans" : "en";
    document.documentElement.dataset.theme = resolvedTheme(boot.settings.colorScheme);
  }, []);

  useEffect(() => {
    void reload().catch(() => setLoadError("error.db"));
  }, [reload]);

  useEffect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onMq = () => {
      if (data?.settings.colorScheme === "system" || !data?.settings.colorScheme) {
        document.documentElement.dataset.theme = resolvedTheme("system");
      }
    };
    mq.addEventListener("change", onMq);
    return () => mq.removeEventListener("change", onMq);
  }, [data?.settings.colorScheme]);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.ctrlKey && e.key.toLowerCase() === "n") {
        e.preventDefault();
        setScreen("record");
        setTimeout(() => amountRef.current?.focus(), 0);
      }
      if (e.ctrlKey && e.key >= "1" && e.key <= "6") {
        e.preventDefault();
        setScreen(SCREENS[Number(e.key) - 1]);
      }
      if (e.ctrlKey && e.key === "Enter" && screen === "record") {
        const form = document.querySelector("form");
        form?.requestSubmit();
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [screen]);

  if (loadError) {
    return (
      <div className="shell">
        <TitleBar />
        <p className="err" style={{ padding: 16 }}>
          {t(loadError)}
        </p>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="shell">
        <TitleBar />
        <p className="muted" style={{ padding: 16 }}>
          {t("loading")}
        </p>
      </div>
    );
  }

  const locale = data.resolvedLanguage === "zh-Hans" ? "zh-Hans" : "en";
  const theme = resolvedTheme(data.settings.colorScheme);

  return (
    <div className="shell">
      <TitleBar />
      <div className="body">
        <NavRail
          screen={screen}
          collapsed={collapsed}
          onScreen={setScreen}
          onToggle={() => setCollapsed((v) => !v)}
        />
        <main className="main">
          {screen === "record" && (
            <RecordScreen
              kinds={data.kinds}
              accounts={data.accounts}
              categories={data.categories}
              settings={data.settings}
              amountFocusRef={amountRef}
              onSaved={reload}
            />
          )}
          {screen === "ledger" && (
            <LedgerScreen
              kinds={data.kinds}
              accounts={data.accounts}
              categories={data.categories}
              tags={data.tags}
              locale={locale}
              jump={jump}
              onJumpConsumed={() => setJump(null)}
              onChanged={reload}
            />
          )}
          {screen === "reports" && (
            <ReportsScreen
              settings={data.settings}
              accounts={data.accounts}
              categories={data.categories}
              kinds={data.kinds}
              locale={locale}
              theme={theme}
              onJump={(j) => {
                setJump(j);
                setScreen("ledger");
              }}
              onChanged={reload}
            />
          )}
          {screen === "accounts" && (
            <AccountsScreen
              accounts={data.accounts}
              settings={data.settings}
              locale={locale}
              onChanged={reload}
            />
          )}
          {screen === "categories" && (
            <CategoriesScreen categories={data.categories} locale={locale} onChanged={reload} />
          )}
          {screen === "settings" && (
            <SettingsScreen
              settings={data.settings}
              accounts={data.accounts}
              categories={data.categories}
              dbPath={data.dbPath}
              onChanged={reload}
            />
          )}
        </main>
      </div>
    </div>
  );
}
