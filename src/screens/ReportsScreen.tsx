import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import type { AccountDto, CategoryDto, KindDto, LedgerJump, ReportDto, SettingsDto } from "../lib/types";
import { getReport, updateSettings } from "../lib/api";
import { formatMinor } from "../lib/money";
import { accountName, categoryName, liftHex } from "../lib/names";
import { localDateString, shiftDate, shiftMonth, shiftYear } from "../lib/time";
import TrendChart from "../charts/TrendChart";
import PieChart from "../charts/PieChart";
import BarChart from "../charts/BarChart";

export default function ReportsScreen({
  settings,
  accounts,
  categories,
  kinds,
  locale,
  theme,
  onJump,
  onChanged,
}: {
  settings: SettingsDto;
  accounts: AccountDto[];
  categories: CategoryDto[];
  kinds: KindDto[];
  locale: string;
  theme: "light" | "dark";
  onJump: (jump: LedgerJump) => void;
  onChanged: () => Promise<void>;
}) {
  const { t } = useTranslation();
  const [mode, setMode] = useState(settings.reportMode ?? "month");
  const [side, setSide] = useState(settings.reportSide ?? "expense");
  const [anchor, setAnchor] = useState(localDateString());
  const [customFrom, setCustomFrom] = useState(settings.reportCustomFrom ?? localDateString());
  const [customTo, setCustomTo] = useState(settings.reportCustomTo ?? localDateString());
  const [compBy, setCompBy] = useState<"main" | "sub">("main");
  const [report, setReport] = useState<ReportDto | null>(null);
  const [error, setError] = useState<string | null>(null);

  const color = side === "income" ? "var(--accent-in)" : "var(--accent-out)";
  const dark = theme === "dark";

  async function persist(next: Partial<SettingsDto>) {
    await updateSettings({
      ...settings,
      reportMode: mode,
      reportSide: side,
      reportCustomFrom: customFrom,
      reportCustomTo: customTo,
      ...next,
    });
    await onChanged();
  }

  useEffect(() => {
    void getReport({
      mode,
      side,
      anchorDate: anchor,
      customFrom: mode === "custom" ? customFrom : null,
      customTo: mode === "custom" ? customTo : null,
    })
      .then((r) => {
        setReport(r);
        setError(null);
      })
      .catch((ex) => {
        setError(typeof ex === "object" && ex && "code" in ex ? String((ex as { code: string }).code) : "error.db");
      });
  }, [mode, side, anchor, customFrom, customTo]);

  function shift(dir: number) {
    if (mode === "week") setAnchor(shiftDate(anchor, dir * 7));
    if (mode === "month") setAnchor(shiftMonth(anchor, dir));
    if (mode === "year") setAnchor(shiftYear(anchor, dir));
  }

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "[" ) shift(-1);
      if (e.key === "]") shift(1);
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  const composition = report
    ? compBy === "main"
      ? report.compositionMain
      : report.compositionSub
    : [];

  const slices = useMemo(() => {
    return composition.map((row) => {
      const cat = categories.find((c) => c.id === row.id);
      const acc = accounts.find((a) => a.id === row.id);
      const label = cat ? categoryName(cat, t) : acc ? accountName(acc, t) : row.id;
      return {
        id: row.id,
        label,
        amountMinor: row.amountMinor,
        color: liftHex(row.colorHex ?? "#52525b", dark),
      };
    });
  }, [composition, categories, accounts, t, dark]);

  return (
    <div>
      <div className="toolbar">
        <div className="seg">
          {(["week", "month", "year", "custom"] as const).map((m) => (
            <button
              key={m}
              type="button"
              className={mode === m ? "active" : ""}
              onClick={() => {
                setMode(m);
                void persist({ reportMode: m });
              }}
            >
              {t(`report.${m}`)}
            </button>
          ))}
        </div>
        {mode !== "custom" && (
          <>
            <button type="button" className="btn" onClick={() => shift(-1)}>
              [
            </button>
            <button type="button" className="btn" onClick={() => shift(1)}>
              ]
            </button>
          </>
        )}
        {mode === "custom" && (
          <>
            <input type="date" value={customFrom} onChange={(e) => setCustomFrom(e.target.value)} />
            <input type="date" value={customTo} onChange={(e) => setCustomTo(e.target.value)} />
          </>
        )}
        <div className="seg">
          <button
            type="button"
            className={side === "expense" ? "active" : ""}
            onClick={() => {
              setSide("expense");
              void persist({ reportSide: "expense" });
            }}
          >
            {t("report.side.expense")}
          </button>
          <button
            type="button"
            className={side === "income" ? "active" : ""}
            onClick={() => {
              setSide("income");
              void persist({ reportSide: "income" });
            }}
          >
            {t("report.side.income")}
          </button>
        </div>
        {report && (
          <span className="muted mono">
            {report.rangeFrom} – {report.rangeTo}
            {report.isoWeek ? ` · ${t("report.isoWeek", { week: report.isoWeek })}` : ""}
          </span>
        )}
      </div>
      {error && <p className="err">{t(error)}</p>}
      {report && report.sideTotal === 0 && report.incomeTotal === 0 && report.expenseTotal === 0 && (
        <p className="muted">{t("report.empty")}</p>
      )}
      {report && (
        <>
          <div className="figures">
            <div className="figure">
              <div className="lbl">{t("report.sideTotal")}</div>
              <div className={`val mono ${side === "income" ? "in" : "out"}`}>
                {formatMinor(report.sideTotal, locale)}
              </div>
            </div>
            <div className="figure">
              <div className="lbl">{t("report.average")}</div>
              <div className="val mono">{formatMinor(report.average, locale)}</div>
            </div>
            {report.delta != null && (
              <div className="figure">
                <div className="lbl">{t("report.vsPrevious")}</div>
                <div className="val mono">{formatMinor(report.delta, locale, true)}</div>
              </div>
            )}
            <div className="figure net">
              <div className="lbl">{t("report.net")}</div>
              <div className="val mono">{formatMinor(report.net, locale, true)}</div>
            </div>
          </div>
          <p className="muted">
            {t("report.secondary", {
              repayment: formatMinor(report.secondaryRepayment, locale),
              volume: formatMinor(report.secondaryTransferVolume, locale),
              fees: formatMinor(report.secondaryTransferFees, locale),
            })}
          </p>
          <div className="section">
            <h2>{t("report.trend")}</h2>
            <TrendChart points={report.trend} color={color} locale={locale} mode={mode} />
          </div>
          <div className="section">
            <div className="toolbar">
              <h2 style={{ margin: 0 }}>{t("report.composition")}</h2>
              <div className="seg">
                <button type="button" className={compBy === "main" ? "active" : ""} onClick={() => setCompBy("main")}>
                  {t("report.byMain")}
                </button>
                <button type="button" className={compBy === "sub" ? "active" : ""} onClick={() => setCompBy("sub")}>
                  {t("report.bySub")}
                </button>
              </div>
            </div>
            <div className="split" style={{ gridTemplateColumns: "280px 1fr" }}>
              <PieChart
                slices={slices}
                locale={locale}
                centerLabel={formatMinor(report.sideTotal, locale)}
                otherLabel={t("report.otherSlice")}
              />
              <table className="table">
                <tbody>
                  {composition.map((row) => {
                    const cat = categories.find((c) => c.id === row.id);
                    return (
                      <tr key={row.id}>
                        <td>
                          <span className="swatch" style={{ background: liftHex(row.colorHex ?? "#52525b", dark) }} />
                          {cat ? categoryName(cat, t) : row.id}
                        </td>
                        <td className="num">{formatMinor(row.amountMinor, locale)}</td>
                        <td className="num">{(row.percentBp / 100).toFixed(2)}%</td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </div>
          <div className="section">
            <h2>{t("report.byAccount")}</h2>
            <table className="table">
              <tbody>
                {report.byAccount.map((row) => {
                  const acc = accounts.find((a) => a.id === row.id);
                  return (
                    <tr key={row.id}>
                      <td>{acc ? accountName(acc, t) : row.id}</td>
                      <td className="num">{formatMinor(row.amountMinor, locale)}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
          {report.comparison && (
            <div className="section">
              <h2>{t("report.comparison")}</h2>
              <BarChart bars={report.comparison} color={color} locale={locale} mode={mode} />
            </div>
          )}
          <div className="section">
            <h2>{t("report.ranking")}</h2>
            <table className="table">
              <tbody>
                {report.ranking.map((row) => (
                  <tr
                    key={row.entryId}
                    onClick={() =>
                      onJump({
                        entryId: row.entryId,
                        fromDate: report.rangeFrom,
                        toDate: report.rangeTo,
                        kindIds: jumpKindIds(side, kinds),
                      })
                    }
                  >
                    <td className="mono">{row.occurredAt.slice(0, 16)}</td>
                    <td>{t(kinds.find((k) => k.id === row.kindId)?.labelKey ?? row.kindId)}</td>
                    <td>{categoryName(categories.find((c) => c.id === row.categoryId), t)}</td>
                    <td>{row.note?.slice(0, 32)}</td>
                    <td className="num">{formatMinor(row.amountMinor, locale)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            <button
              type="button"
              className="btn"
              onClick={() =>
                onJump({
                  entryId: report.ranking[0]?.entryId ?? "",
                  fromDate: report.rangeFrom,
                  toDate: report.rangeTo,
                  kindIds: jumpKindIds(side, kinds),
                })
              }
            >
              {t("action.more")}
            </button>
          </div>
        </>
      )}
    </div>
  );
}

function jumpKindIds(side: string, kinds: KindDto[]): string[] {
  if (side === "income") {
    return kinds.filter((k) => k.reportBucket === "income").map((k) => k.id);
  }
  return kinds
    .filter((k) => k.reportBucket === "expense" || k.feeReportBucket === "expense")
    .map((k) => k.id);
}
