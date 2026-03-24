import { useEffect, useRef, useState, type MouseEvent } from "react";
import Markdown from "react-markdown";
import { api } from "../lib/tauri";
import { t } from "../lib/i18n";
import type { RemoteSkill } from "../types";

interface RemoteSkillModalProps {
  skill: RemoteSkill;
  enabledAgents: string[];
  installed: boolean;
  onInstall: (sourceId: string, folderName: string, targetAgent: string, force?: boolean) => Promise<void>;
  onClose: () => void;
}

function stripFrontmatter(content: string): string {
  const match = content.match(/^---\r?\n[\s\S]*?\r?\n---\r?\n?/);
  return match ? content.slice(match[0].length) : content;
}

export function RemoteSkillModal({
  skill,
  enabledAgents,
  installed,
  onInstall,
  onClose,
}: RemoteSkillModalProps) {
  const overlayRef = useRef<HTMLDivElement>(null);
  const [expanded, setExpanded] = useState(false);
  const [fullText, setFullText] = useState<string | null>(null);
  const [loadingText, setLoadingText] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [selectedAgent, setSelectedAgent] = useState(enabledAgents[0] || "");
  const [overwriteConfirm, setOverwriteConfirm] = useState(false);
  const [toast, setToast] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 3000);
  };

  const handleClose = () => {
    overlayRef.current?.classList.remove("open");
    setTimeout(onClose, 500);
  };

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (overwriteConfirm) {
          setOverwriteConfirm(false);
        } else if (expanded) {
          setExpanded(false);
        } else {
          handleClose();
        }
      }
    };
    document.addEventListener("keydown", handleKey);
    return () => document.removeEventListener("keydown", handleKey);
  }, [onClose, overwriteConfirm, expanded]);

  useEffect(() => {
    const frame = requestAnimationFrame(() => {
      overlayRef.current?.classList.add("open");
    });
    return () => cancelAnimationFrame(frame);
  }, []);

  const handleOverlayClick = (e: MouseEvent) => {
    if (e.target === overlayRef.current) {
      if (overwriteConfirm) {
        setOverwriteConfirm(false);
      } else {
        handleClose();
      }
    }
  };

  const handleToggleExpand = async () => {
    if (expanded) {
      setExpanded(false);
      return;
    }
    if (fullText !== null) {
      setExpanded(true);
      return;
    }
    setLoadingText(true);
    try {
      const content = await api.getRemoteSkillContent(skill.source_id, skill.folder_name);
      setFullText(content || t("loadFailed"));
      setExpanded(true);
    } catch {
      setFullText(t("loadFailed"));
      setExpanded(true);
    } finally {
      setLoadingText(false);
    }
  };

  const handleInstall = async (force?: boolean) => {
    if (!selectedAgent || installing) return;
    setInstalling(true);
    try {
      await onInstall(skill.source_id, skill.folder_name, selectedAgent, force);
      showToast(t("installSuccess"));
    } catch (e: unknown) {
      const err = e as { kind?: string; message?: string };
      const message = err.message || t("installFailed");
      const isOverwriteConflict =
        err.kind === "Conflict" &&
        /already exists/i.test(message);
      if (isOverwriteConflict) {
        setOverwriteConfirm(true);
      } else {
        showToast(message);
      }
    } finally {
      setInstalling(false);
    }
  };

  const handleOverwriteConfirmAction = async () => {
    setOverwriteConfirm(false);
    await handleInstall(true);
  };

  const markdownBody = fullText ? stripFrontmatter(fullText) : "";

  const updatedParts: string[] = [];
  if (skill.updated_at) {
    const d = new Date(skill.updated_at);
    updatedParts.push(`${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`);
  }
  if (skill.updated_by) updatedParts.push(skill.updated_by);
  const updatedLabel = updatedParts.length > 0 ? `${t("lastUpdated")} ${updatedParts.join(" · ")}` : null;

  return (
    <div className="modal-overlay" ref={overlayRef} onClick={handleOverlayClick}>
      {toast && <div className="toast">{toast}</div>}
      <div className={`modal-card${expanded ? " modal-card--expanded" : ""}`}>
        <div className="modal-header">
          <div>
            <div className="modal-title">
              {skill.name || skill.folder_name}
              {updatedLabel && (
                <span className="modal-updated">{updatedLabel}</span>
              )}
            </div>
            <div className="modal-folder">📁 {skill.folder_name}</div>
          </div>
          <button className="modal-close" onClick={handleClose}>✕</button>
        </div>

        <div className="modal-desc">
          {skill.description || t("noDescription")}
        </div>

        <div className="modal-meta-row">
          <span style={{ fontSize: 12, color: "var(--text-muted)" }}>
            {t("source")} {skill.source_name}
          </span>
          {installed && (
            <span className="remote-card-badge">{t("installed")}</span>
          )}
        </div>

        <button
          className="btn-view-full"
          onClick={handleToggleExpand}
          disabled={loadingText}
        >
          {loadingText ? t("loading") : expanded ? t("collapseFullText") : t("viewFullText")}
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2.5"
            strokeLinecap="round"
            strokeLinejoin="round"
            className={`btn-view-full-icon${expanded ? " btn-view-full-icon--open" : ""}`}
          >
            <polyline points="6 9 12 15 18 9" />
          </svg>
        </button>

        {expanded && fullText !== null && (
          <div className="modal-reader">
            <div className="modal-reader-content markdown-body">
              <Markdown>{markdownBody}</Markdown>
            </div>
          </div>
        )}

        <div className="modal-footer">
          <select
            value={selectedAgent}
            onChange={e => setSelectedAgent(e.target.value)}
            className="remote-card-select"
          >
            {enabledAgents.map(a => (
              <option key={a} value={a}>{a}</option>
            ))}
          </select>
          <button
            className="btn-primary"
            onClick={() => { void handleInstall(); }}
            disabled={installing || !selectedAgent}
          >
            {installing ? t("installing") : installed ? t("reinstall") : t("install")}
          </button>
        </div>
      </div>

      {overwriteConfirm && (
        <div className="confirm-dialog" onClick={e => e.stopPropagation()}>
          <div className="confirm-dialog-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
              <line x1="12" y1="9" x2="12" y2="13" />
              <line x1="12" y1="17" x2="12.01" y2="17" />
            </svg>
          </div>
          <div className="confirm-dialog-title">{t("overwriteConfirm")}</div>
          <div className="confirm-dialog-desc">
            <strong>{skill.name || skill.folder_name}</strong>
          </div>
          <div className="confirm-dialog-actions">
            <button className="btn-secondary" onClick={() => setOverwriteConfirm(false)}>
              {t("cancel")}
            </button>
            <button className="btn-primary" onClick={() => { void handleOverwriteConfirmAction(); }}>
              {t("install")}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
