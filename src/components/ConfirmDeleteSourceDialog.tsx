import { useState } from "react";
import { t } from "../lib/i18n";

interface ConfirmDeleteSourceDialogProps {
  sourceName: string;
  onConfirm: () => Promise<void>;
  onCancel: () => void;
}

export function ConfirmDeleteSourceDialog({ sourceName, onConfirm, onCancel }: ConfirmDeleteSourceDialogProps) {
  const [deleting, setDeleting] = useState(false);

  const handleConfirm = async () => {
    setDeleting(true);
    try {
      await onConfirm();
    } finally {
      setDeleting(false);
    }
  };

  return (
    <div className="dialog-overlay" onClick={onCancel}>
      <div
        className="confirm-dialog"
        style={{ position: "relative", top: "auto", left: "auto", transform: "none", animation: "dialogIn 0.3s cubic-bezier(0.22, 1, 0.36, 1)" }}
        onClick={e => e.stopPropagation()}
      >
        <div className="confirm-dialog-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M3 6h18" />
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
            <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
          </svg>
        </div>
        <div className="confirm-dialog-title">{t("deleteSource")}</div>
        <div className="confirm-dialog-desc">
          {t("deleteSourceConfirmMsg")} <strong>{sourceName}</strong>{t("deleteSourceConfirmSuffix")}
        </div>
        <div className="confirm-dialog-actions">
          <button
            className="btn-secondary"
            onClick={onCancel}
            disabled={deleting}
          >
            {t("cancel")}
          </button>
          <button
            className="btn-danger"
            onClick={() => { void handleConfirm(); }}
            disabled={deleting}
          >
            {deleting ? t("deleting") : t("delete")}
          </button>
        </div>
      </div>
    </div>
  );
}
