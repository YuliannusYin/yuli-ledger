import { getCurrentWindow } from "@tauri-apps/api/window";
import { useTranslation } from "react-i18next";

export default function TitleBar() {
  const { t } = useTranslation();
  const win = getCurrentWindow();
  return (
    <div className="titlebar" data-tauri-drag-region>
      <div className="titlebar-title" data-tauri-drag-region>
        {t("app.name")}
      </div>
      <div className="titlebar-btns">
        <button type="button" aria-label={t("title.minimize")} onClick={() => void win.minimize()}>
          –
        </button>
        <button
          type="button"
          aria-label={t("title.maximize")}
          onClick={() => void win.toggleMaximize()}
        >
          □
        </button>
        <button
          type="button"
          className="close"
          aria-label={t("title.close")}
          onClick={() => void win.close()}
        >
          ×
        </button>
      </div>
    </div>
  );
}
