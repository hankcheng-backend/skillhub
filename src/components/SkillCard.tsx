import type { Skill } from "../types";
import { t } from "../lib/i18n";
import { AgentIcons } from "./AgentIcons";

interface SkillCardProps {
  skill: Skill;
  enabledAgents: string[];
  originAgents?: string[];
  syncedMap?: Record<string, string>;
  syncSourceSkillId?: string;
  onOpen: () => void;
  onSync: () => void;
}

export function SkillCard({
  skill,
  enabledAgents,
  originAgents,
  syncedMap,
  syncSourceSkillId,
  onOpen,
  onSync,
}: SkillCardProps) {
  return (
    <div className="skill-card" onClick={onOpen}>
      <div className="skill-card-header">
        <div className="skill-card-name">{skill.name || skill.folder_name}</div>
      </div>
      {skill.description ? (
        <div className="skill-card-desc">{skill.description}</div>
      ) : (
        <div className="skill-card-desc" style={{ color: 'var(--text-muted)', fontStyle: 'italic' }}>
          {t("noDescription")}
        </div>
      )}
      <div className="skill-card-footer">
        <AgentIcons
          skill={skill}
          enabledAgents={enabledAgents}
          originAgents={originAgents}
          syncedMap={syncedMap}
          syncSourceSkillId={syncSourceSkillId}
          onSync={onSync}
        />
      </div>
    </div>
  );
}
