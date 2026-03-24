import { useEffect, useRef, useState, type MouseEvent } from "react";
import Markdown from "react-markdown";
import { api } from "../lib/tauri";
import { t } from "../lib/i18n";
import type { Skill, Source } from "../types";
import { AgentIcons } from "./AgentIcons";

interface SkillModalProps {
  skill: Skill;
  enabledAgents: string[];
  originAgents?: string[];
  syncedMap?: Record<string, string>;
  syncSourceSkillId?: string;
  onSync: () => void;
  onDelete: () => Promise<void> | void;
  onClose: () => void;
  sources?: Source[];
}

function stripFrontmatter(content: string): string {
  const match = content.match(/^---\r?\n[\s\S]*?\r?\n---\r?\n?/);
  return match ? content.slice(match[0].length) : content;
}

export function SkillModal({
  skill,
  enabledAgents,
  originAgents,
  syncedMap,
  syncSourceSkillId,
  onSync,
  onDelete,
  onClose,
  sources = [],
}: SkillModalProps) {
  const overlayRef = useRef<HTMLDivElement>(null);
  const [deleting, setDeleting] = useState(false);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [fullText, setFullText] = useState<string | null>(null);
  const [loadingText, setLoadingText] = useState(false);
  const [showUploadDialog, setShowUploadDialog] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [uploadSourceId, setUploadSourceId] = useState("");
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
        if (confirmingDelete) {
          setConfirmingDelete(false);
        } else if (overwriteConfirm) {
          setOverwriteConfirm(false);
        } else if (showUploadDialog) {
          setShowUploadDialog(false);
        } else if (expanded) {
          setExpanded(false);
        } else {
          handleClose();
        }
      }
    };
    document.addEventListener("keydown", handleKey);
    return () => document.removeEventListener("keydown", handleKey);
  }, [onClose, confirmingDelete, overwriteConfirm, showUploadDialog, expanded]);

  useEffect(() => {
    const frame = requestAnimationFrame(() => {
      overlayRef.current?.classList.add("open");
    });
    return () => cancelAnimationFrame(frame);
  }, []);

  const handleOverlayClick = (e: MouseEvent) => {
    if (e.target === overlayRef.current) {
      if (confirmingDelete) {
        setConfirmingDelete(false);
      } else if (overwriteConfirm) {
        setOverwriteConfirm(false);
      } else if (showUploadDialog) {
        setShowUploadDialog(false);
      } else {
        handleClose();
      }
    }
  };

  const handleDelete = async () => {
    if (deleting) return;
    setDeleting(true);
    try {
      await onDelete();
      handleClose();
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : (e as { message?: string }).message || String(e);
      showToast(msg);
    } finally {
      setDeleting(false);
      setConfirmingDelete(false);
    }
  };

  const gitlabSources = sources.filter(s => s.type === "gitlab");

  const handleUpload = async () => {
    if (uploading) return;
    setUploading(true);
    try {
      await api.uploadSkill(uploadSourceId, skill.id);
      setShowUploadDialog(false);
      showToast(t("uploadSuccess"));
    } catch (e: unknown) {
      const err = e as { kind?: string; message?: string };
      if (err.kind === "Conflict") {
        setShowUploadDialog(false);
        setOverwriteConfirm(true);
      } else {
        showToast(err.message || t("uploadFailed"));
      }
    } finally {
      setUploading(false);
    }
  };

  const handleOverwriteConfirm = async () => {
    setOverwriteConfirm(false);
    setShowUploadDialog(false);
    setUploading(true);
    try {
      await api.uploadSkill(uploadSourceId, skill.id, true);
      showToast(t("uploadSuccess"));
    } catch (e2: unknown) {
      const msg = (e2 as { message?: string }).message || t("uploadFailed");
      showToast(msg);
    } finally {
      setUploading(false);
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
      const content = await api.getSkillContent(skill.id);
      setFullText(content);
      setExpanded(true);
    } catch {
      setFullText(t("loadFailed"));
      setExpanded(true);
    } finally {
      setLoadingText(false);
    }
  };

  const tags: string[] = (() => {
    try { return JSON.parse(skill.tags || "[]"); }
    catch { return []; }
  })();

  const markdownBody = fullText ? stripFrontmatter(fullText) : "";

  return (
    <div className="modal-overlay" ref={overlayRef} onClick={handleOverlayClick}>
      {toast && <div className="toast">{toast}</div>}
      <div className={`modal-card${expanded ? " modal-card--expanded" : ""}`}>
        <div className="modal-header">
          <div>
            <div className="modal-title">{skill.name || skill.folder_name}</div>
            <div className="modal-folder">📁 {skill.folder_name}</div>
          </div>
          <button className="modal-close" onClick={handleClose}>✕</button>
        </div>

        <div className="modal-desc">
          {skill.description || t("noDescription")}
        </div>

        <div className="modal-agents-row">
          <span className="modal-agents-label">{t("agents")}</span>
          <AgentIcons
            skill={skill}
            enabledAgents={enabledAgents}
            originAgents={originAgents}
            syncedMap={syncedMap}
            syncSourceSkillId={syncSourceSkillId}
            onSync={onSync}
            onError={showToast}
          />
        </div>

        {tags.length > 0 && (
          <div className="tags">
            {tags.map(tag => (
              <span key={tag} className="tag">#{tag}</span>
            ))}
          </div>
        )}

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
          <button
            className="btn-danger"
            onClick={() => setConfirmingDelete(true)}
            disabled={deleting}
          >
            {t("delete")}
          </button>
          {gitlabSources.length > 0 && (
            <button
              className="btn-primary"
              onClick={() => {
                if (!uploadSourceId) setUploadSourceId(gitlabSources[0].id);
                setShowUploadDialog(true);
              }}
            >
              {t("upload")}
            </button>
          )}
        </div>
      </div>

      {confirmingDelete && (
        <div className="confirm-dialog" onClick={e => e.stopPropagation()}>
          <div className="confirm-dialog-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M3 6h18" />
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
              <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
            </svg>
          </div>
          <div className="confirm-dialog-title">{t("deleteSkill")}</div>
          <div className="confirm-dialog-desc">
            {t("deleteConfirmMsg")} <strong>{skill.name || skill.folder_name}</strong>{t("deleteConfirmSuffix")}
          </div>
          <div className="confirm-dialog-actions">
            <button
              className="btn-secondary"
              onClick={() => setConfirmingDelete(false)}
              disabled={deleting}
            >
              {t("cancel")}
            </button>
            <button
              className="btn-danger"
              onClick={() => { void handleDelete(); }}
              disabled={deleting}
            >
              {deleting ? t("deleting") : t("delete")}
            </button>
          </div>
        </div>
      )}

      {showUploadDialog && (
        <div className="confirm-dialog" onClick={e => e.stopPropagation()}>
          <div className="confirm-dialog-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="17 8 12 3 7 8" />
              <line x1="12" y1="3" x2="12" y2="15" />
            </svg>
          </div>
          <div className="confirm-dialog-title">{t("uploadSkill")}</div>
          <div className="confirm-dialog-desc">
            <strong>{skill.name || skill.folder_name}</strong>
          </div>
          <div className="confirm-dialog-field">
            <label className="confirm-dialog-label">
              {t("selectTargetSource")}
            </label>
            {gitlabSources.length === 0 ? (
              <div className="confirm-dialog-empty">
                {t("noGitlabSources")}
              </div>
            ) : (
              <select
                className="confirm-dialog-select"
                value={uploadSourceId}
                onChange={e => setUploadSourceId(e.target.value)}
              >
                {gitlabSources.map(s => (
                  <option key={s.id} value={s.id}>{s.name}</option>
                ))}
              </select>
            )}
          </div>
          <div className="confirm-dialog-actions">
            <button
              className="btn-secondary"
              onClick={() => setShowUploadDialog(false)}
              disabled={uploading}
            >
              {t("cancel")}
            </button>
            <button
              className="btn-primary"
              onClick={() => { void handleUpload(); }}
              disabled={uploading || !uploadSourceId}
            >
              {uploading ? t("uploading") : t("upload")}
            </button>
          </div>
        </div>
      )}

      {overwriteConfirm && (
        <div className="confirm-dialog" onClick={e => e.stopPropagation()}>
          <div className="confirm-dialog-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
              <line x1="12" y1="9" x2="12" y2="13" />
              <line x1="12" y1="17" x2="12.01" y2="17" />
            </svg>
          </div>
          <div className="confirm-dialog-title">{t("uploadSkill")}</div>
          <div className="confirm-dialog-desc">{t("overwriteRemoteConfirm")}</div>
          <div className="confirm-dialog-actions">
            <button className="btn-secondary" onClick={() => setOverwriteConfirm(false)}>
              {t("cancel")}
            </button>
            <button className="btn-primary" onClick={() => { void handleOverwriteConfirm(); }}>
              {t("upload")}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
