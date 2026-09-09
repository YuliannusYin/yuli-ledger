import { useTranslation } from "react-i18next";

export default function ConfirmDialog({
  message,
  onConfirm,
  onCancel,
}: {
  message: string;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="modal-back">
      <div className="modal surface">
        <p>{message}</p>
        <div className="row" style={{ marginTop: 12 }}>
          <button type="button" className="btn" onClick={onCancel}>
            {t("action.cancel")}
          </button>
          <button type="button" className="btn danger" onClick={onConfirm}>
            {t("action.confirm")}
          </button>
        </div>
      </div>
    </div>
  );
}
