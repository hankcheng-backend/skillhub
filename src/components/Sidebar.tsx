import type { Source } from "../types";
import { t } from "../lib/i18n";

interface SidebarProps {
  sources: Source[];
  activeSource: string;
  onSelectSource: (id: string) => void;
  onAddSource: () => void;
}

export function Sidebar({ sources, activeSource, onSelectSource, onAddSource }: SidebarProps) {
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

      {sources.map(source => (
        <div
          key={source.id}
          className={`sidebar-item${activeSource === source.id ? " active" : ""}`}
          role="button"
          tabIndex={0}
          onClick={() => onSelectSource(source.id)}
          onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onSelectSource(source.id); } }}
        >
          <span className="sidebar-item-icon">{source.type === "gitlab" ? "🦊" : "📁"}</span>
          {source.name}
        </div>
      ))}
    </aside>
  );
}
