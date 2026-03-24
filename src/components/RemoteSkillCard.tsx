import { t } from "../lib/i18n";
import type { RemoteSkill } from "../types";

function formatDate(iso: string): string {
  const d = new Date(iso);
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

interface RemoteSkillCardProps {
  skill: RemoteSkill;
  installed: boolean;
  onClick: () => void;
}

export function RemoteSkillCard({ skill, installed, onClick }: RemoteSkillCardProps) {
  const updatedParts: string[] = [];
  if (skill.updated_at) updatedParts.push(formatDate(skill.updated_at));
  if (skill.updated_by) updatedParts.push(skill.updated_by);
  const updatedText = updatedParts.length > 0 ? updatedParts.join(" · ") : null;

  return (
    <div className="remote-card" onClick={onClick} style={{ cursor: "pointer" }}>
      <div className="remote-card-name">
        {skill.name || skill.folder_name}
        {installed && <span className="remote-card-badge">{t("installed")}</span>}
      </div>
      {skill.description && (
        <div className="remote-card-desc">{skill.description}</div>
      )}
      <div className="remote-card-meta">
        {skill.folder_name} &middot; {skill.source_name}
        {updatedText && (
          <>
            {" "}&middot;{" "}{t("lastUpdated")} {updatedText}
          </>
        )}
      </div>
    </div>
  );
}
