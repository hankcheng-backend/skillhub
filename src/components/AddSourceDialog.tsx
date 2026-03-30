import { useState } from "react";
import { api } from "../lib/tauri";
import { formatAddSourceError } from "../lib/error";
import { t } from "../lib/i18n";

interface AddSourceDialogProps {
  onClose: () => void;
  onAdded: () => void;
}

export function AddSourceDialog({ onClose, onAdded }: AddSourceDialogProps) {
  const [type, setType] = useState<"gitlab" | "gdrive">("gitlab");
  const [name, setName] = useState("");
  const [url, setUrl] = useState("");
  const [token, setToken] = useState("");
  const [error, setError] = useState<string | null>(null);

  const nameValue = name.trim();
  const urlValue = url.trim();
  const tokenValue = token.trim();
  const canSubmit =
    nameValue.length > 0 &&
    urlValue.length > 0 &&
    (type !== "gitlab" || tokenValue.length > 0);

  const handleSubmit = async () => {
    if (!canSubmit) {
      setError(t("fillAllFields"));
      return;
    }

    try {
      setError(null);
      await api.addSource(
        nameValue,
        type,
        type === "gitlab" ? urlValue : undefined,
        type === "gdrive" ? urlValue : undefined,
        type === "gitlab" ? tokenValue : undefined,
      );
      onAdded();
      onClose();
    } catch (e: unknown) {
      setError(formatAddSourceError(e));
    }
  };

  return (
    <div className="dialog-overlay">
      <div className="dialog-box">
        <h3 className="dialog-title">{t("addSourceTitle")}</h3>

        <div className="form-group">
          <label className="form-label">{t("type")}</label>
          <div className="radio-group">
            <label className="radio-label">
              <input type="radio" checked={type === "gitlab"} onChange={() => setType("gitlab")} /> GitLab
            </label>
            <label className="radio-label-disabled">
              <input type="radio" disabled checked={false} onChange={() => {}} /> Google Drive
              <span className="coming-soon-badge">{t("comingSoon")}</span>
            </label>
          </div>
        </div>

        <div className="form-group">
          <label className="form-label">{t("name")}</label>
          <input
            value={name}
            onChange={e => setName(e.target.value)}
            placeholder={t("placeholderSourceName")}
            className="form-input"
          />
        </div>

        <div className="form-group">
          <label className="form-label">
            {type === "gitlab" ? t("repositoryUrl") : t("folderId")}
          </label>
          <input
            value={url}
            onChange={e => setUrl(e.target.value)}
            placeholder={type === "gitlab" ? t("placeholderRepoUrl") : t("placeholderFolderId")}
            className="form-input"
          />
        </div>

        {type === "gitlab" && (
          <div className="form-group">
            <label className="form-label">{t("personalAccessToken")}</label>
            <input
              type="password"
              value={token}
              onChange={e => setToken(e.target.value)}
              placeholder={t("placeholderToken")}
              className="form-input"
            />
          </div>
        )}

        {error && <div className="mcp-error">{error}</div>}

        <div className="dialog-footer">
          <button onClick={onClose} className="btn-secondary">{t("cancel")}</button>
          <button onClick={handleSubmit} className="btn-primary" disabled={!canSubmit}>{t("add")}</button>
        </div>
      </div>
    </div>
  );
}
