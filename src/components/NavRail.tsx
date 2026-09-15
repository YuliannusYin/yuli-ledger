import { useTranslation } from "react-i18next";
import { SCREENS, type ScreenId } from "../lib/types";

const ICONS: Record<ScreenId, string> = {
  record: "R",
  pending: "I",
  ledger: "L",
  reports: "P",
  accounts: "A",
  categories: "C",
  settings: "S",
};

export default function NavRail({
  screen,
  collapsed,
  pendingCount,
  onScreen,
  onToggle,
}: {
  screen: ScreenId;
  collapsed: boolean;
  pendingCount: number;
  onScreen: (id: ScreenId) => void;
  onToggle: () => void;
}) {
  const { t } = useTranslation();
  return (
    <nav className={`rail ${collapsed ? "collapsed" : ""}`}>
      {SCREENS.map((id) => (
        <button
          key={id}
          type="button"
          className={`rail-item ${screen === id ? "active" : ""}`}
          onClick={() => onScreen(id)}
          title={t(`nav.${id}`)}
        >
          <span className="rail-icon">{ICONS[id]}</span>
          {!collapsed && t(`nav.${id}`)}
          {id === "pending" && pendingCount > 0 && (
            <span className="rail-badge">{pendingCount > 99 ? "99+" : pendingCount}</span>
          )}
        </button>
      ))}
      <button
        type="button"
        className="rail-collapse"
        onClick={onToggle}
        title={collapsed ? t("nav.expand") : t("nav.collapse")}
      >
        {collapsed ? "»" : "«"}
      </button>
    </nav>
  );
}
