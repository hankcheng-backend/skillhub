import { useState } from "react";
import { t } from "../lib/i18n";
import { api } from "../lib/tauri";
import { extractErrorMessage } from "../lib/error";

interface UpdateTokenDialogProps {
  sourceId: string;
  sourceName: string;
  onClose: () => void;
  onSuccess: () => void;
}

export function UpdateTokenDialog({ sourceId, sourceName, onClose, onSuccess }: UpdateTokenDialogProps) {
  const [token, setToken] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSave = async () => {
    if (!token.trim()) return;
    setIsSaving(true);
    setError(null);
    try {
      await api.updateSourceToken(sourceId, token.trim());
      onSuccess();
    } catch (e: unknown) {
      setError(extractErrorMessage(e));
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog-box" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>{t("patExpiredTitle")}</h2>
        </div>
        <div className="dialog-body">
          <p style={{ color: "var(--text-muted)", marginBottom: "1rem" }}>
            {t("patExpiredDesc")}
          </p>
          <p style={{ color: "var(--text-muted)", marginBottom: "0.5rem", fontSize: "0.85rem" }}>
            {sourceName}
          </p>
          <input
            type="password"
            className="form-input"
            value={token}
            onChange={(e) => setToken(e.target.value)}
            placeholder="glpat-xxxxxxxxxxxxxxxxxxxx"
            autoFocus
          />
          {error && <p style={{ color: "var(--danger)", marginTop: "0.5rem", fontSize: "0.85rem" }}>{error}</p>}
        </div>
        <div className="dialog-actions">
          <button className="btn-secondary" onClick={onClose} disabled={isSaving}>
            {t("cancel")}
          </button>
          <button className="btn-primary" onClick={() => { void handleSave(); }} disabled={isSaving || !token.trim()}>
            {isSaving ? "..." : t("patExpiredSave")}
          </button>
        </div>
      </div>
    </div>
  );
}
