import { useState, useEffect, useCallback } from "react";
import type { Source } from "../types";
import { t } from "../lib/i18n";
import { ConfirmDeleteSourceDialog } from "./ConfirmDeleteSourceDialog";

interface SidebarProps {
  sources: Source[];
  activeSource: string;
  onSelectSource: (id: string) => void;
  onAddSource: () => void;
  expiredSourceIds: Set<string>;
  onTokenUpdate: (sourceId: string) => void;
  onDeleteSource: (sourceId: string) => Promise<void>;
}

export function Sidebar({ sources, activeSource, onSelectSource, onAddSource, expiredSourceIds, onTokenUpdate, onDeleteSource }: SidebarProps) {
  const [contextMenu, setContextMenu] = useState<{
    sourceId: string;
    sourceName: string;
    x: number;
    y: number;
  } | null>(null);

  const [confirmingDeleteSource, setConfirmingDeleteSource] = useState<{
    sourceId: string;
    sourceName: string;
  } | null>(null);

  const handleContextMenu = (e: React.MouseEvent, sourceId: string, sourceName: string) => {
    e.preventDefault();
    e.stopPropagation();
    setContextMenu({ sourceId, sourceName, x: e.clientX, y: e.clientY });
  };

  const closeContextMenu = useCallback(() => setContextMenu(null), []);

  useEffect(() => {
    if (!contextMenu) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") closeContextMenu();
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [contextMenu, closeContextMenu]);

  return (
    <aside className="app-sidebar">
      <button className="sidebar-add-btn" onClick={onAddSource}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round">
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
        {t("addSource")}
      </button>

      <div className="sidebar-divider" />

      <div className="sidebar-section-label">{t("sources")}</div>

      <div
        className={`sidebar-item${activeSource === "local" ? " active" : ""}`}
        role="button"
        tabIndex={0}
        onClick={() => onSelectSource("local")}
        onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onSelectSource("local"); } }}
      >
        <span className="sidebar-item-icon">💻</span>
        {t("localSkills")}
      </div>

      {sources.map(source => {
        const isExpired = expiredSourceIds.has(source.id);
        return (
          <div
            key={source.id}
            className={`sidebar-item${activeSource === source.id ? " active" : ""}`}
            role="button"
            tabIndex={0}
            onClick={() => isExpired ? onTokenUpdate(source.id) : onSelectSource(source.id)}
            onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); isExpired ? onTokenUpdate(source.id) : onSelectSource(source.id); } }}
            onContextMenu={(e) => handleContextMenu(e, source.id, source.name)}
          >
            <span className="sidebar-item-icon">{source.type === "gitlab" ? "🦊" : "📁"}</span>
            {source.name}
            {isExpired && <span className="pat-expired-badge">{t("patExpiredBadge")}</span>}
          </div>
        );
      })}

      {contextMenu && (
        <>
          <div className="sidebar-context-menu-backdrop" onClick={closeContextMenu} />
          <div className="sidebar-context-menu" style={{ top: contextMenu.y, left: contextMenu.x }}>
            <button
              className="sidebar-context-menu-item"
              onClick={() => {
                setConfirmingDeleteSource({ sourceId: contextMenu.sourceId, sourceName: contextMenu.sourceName });
                setContextMenu(null);
              }}
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M3 6h18" />
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
                <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
              </svg>
              {t("deleteSource")}
            </button>
          </div>
        </>
      )}

      {confirmingDeleteSource && (
        <ConfirmDeleteSourceDialog
          sourceName={confirmingDeleteSource.sourceName}
          onConfirm={async () => {
            await onDeleteSource(confirmingDeleteSource.sourceId);
            setConfirmingDeleteSource(null);
          }}
          onCancel={() => setConfirmingDeleteSource(null)}
        />
      )}
    </aside>
  );
}
