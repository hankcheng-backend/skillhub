import { useEffect, useMemo, useState } from "react";
import { api } from "../lib/tauri";
import { t } from "../lib/i18n";
import type { RemoteSkill, Skill, Source } from "../types";
import { SearchBar } from "./SearchBar";
import { SkillCard } from "./SkillCard";
import { SkillModal } from "./SkillModal";
import { RemoteSkillCard } from "./RemoteSkillCard";
import { RemoteSkillModal } from "./RemoteSkillModal";

interface SkillGridProps {
  skills: Skill[];
  onRefresh: () => void;
  activeSource?: string;
  remoteSkills?: RemoteSkill[];
  browsing?: boolean;
  onInstall?: (sourceId: string, folderName: string, targetAgent: string, force?: boolean) => Promise<void>;
  sources?: Source[];
}

interface MergedSkillEntry {
  skill: Skill;
  originAgents: string[];
  syncedMap: Record<string, string>;
  syncSourceSkillId: string;
}

const AGENT_PRIORITY = ["claude", "codex", "gemini"];

function agentOrder(agentId: string): number {
  const index = AGENT_PRIORITY.indexOf(agentId);
  return index >= 0 ? index : Number.MAX_SAFE_INTEGER;
}

function mergeSkillsByFolder(skills: Skill[]): MergedSkillEntry[] {
  const grouped = new Map<string, Skill[]>();
  for (const skill of skills) {
    const list = grouped.get(skill.folder_name) ?? [];
    list.push(skill);
    grouped.set(skill.folder_name, list);
  }

  return Array.from(grouped.values())
    .map(group => {
      const sorted = [...group].sort((a, b) => {
        const byAgent = agentOrder(a.origin_agent) - agentOrder(b.origin_agent);
        if (byAgent !== 0) return byAgent;
        return a.id.localeCompare(b.id);
      });

      const primary = sorted[0];
      const originAgents = Array.from(new Set(sorted.map(s => s.origin_agent)));
      const syncedMap: Record<string, string> = {};

      for (const sourceSkill of sorted) {
        for (const syncedAgent of sourceSkill.synced_to) {
          if (originAgents.includes(syncedAgent)) continue;
          if (!syncedMap[syncedAgent]) {
            syncedMap[syncedAgent] = sourceSkill.id;
          }
        }
      }

      return {
        skill: {
          ...primary,
          synced_to: Object.keys(syncedMap),
        },
        originAgents,
        syncedMap,
        syncSourceSkillId: primary.id,
      };
    })
    .sort((a, b) => {
      const aName = (a.skill.name || a.skill.folder_name).toLowerCase();
      const bName = (b.skill.name || b.skill.folder_name).toLowerCase();
      return aName.localeCompare(bName);
    });
}

function LoadingSkeletons() {
  return (
    <div className="skill-grid">
      {Array.from({ length: 6 }).map((_, i) => (
        <div key={i} className="skeleton-card" style={{ animationDelay: `${i * 0.08}s` }}>
          <div className="skeleton-line" />
          <div className="skeleton-line" />
          <div className="skeleton-line" />
        </div>
      ))}
    </div>
  );
}

export function SkillGrid({ skills, onRefresh, activeSource = "local", remoteSkills = [], browsing = false, onInstall, sources = [] }: SkillGridProps) {
  const [search, setSearch] = useState("");
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);
  const [selectedRemoteSkill, setSelectedRemoteSkill] = useState<RemoteSkill | null>(null);
  const [enabledAgents, setEnabledAgents] = useState<string[]>([]);

  useEffect(() => {
    api.getAgents().then(agents => {
      setEnabledAgents(agents.filter(a => a.enabled).map(a => a.id));
    });
  }, []);

  const isRemote = activeSource !== "local";
  const q = search.toLowerCase();

  const filteredLocal = skills.filter(s =>
    (s.name || "").toLowerCase().includes(q) ||
    (s.description || "").toLowerCase().includes(q) ||
    s.folder_name.toLowerCase().includes(q) ||
    (s.tags || "").toLowerCase().includes(q)
  );
  const mergedLocal = mergeSkillsByFolder(filteredLocal);

  const selectedEntry = useMemo(() => {
    if (!selectedFolder) return null;
    return mergedLocal.find(entry => entry.skill.folder_name === selectedFolder) ?? null;
  }, [selectedFolder, mergedLocal]);

  const filteredRemote = remoteSkills.filter(s =>
    (s.name || "").toLowerCase().includes(q) ||
    (s.description || "").toLowerCase().includes(q) ||
    s.folder_name.toLowerCase().includes(q)
  );

  const installedFolders = useMemo(
    () => new Set(skills.map(s => s.folder_name)),
    [skills]
  );

  const handleDelete = async (folderName: string) => {
    const skillIds = skills
      .filter(skill => skill.folder_name === folderName)
      .map(skill => skill.id);

    if (skillIds.length === 0) {
      throw new Error(`No skills found for folder '${folderName}'`);
    }

    const errors: string[] = [];
    for (const skillId of skillIds) {
      try {
        await api.deleteSkill(skillId);
      } catch (e: unknown) {
        const msg = e instanceof Error ? e.message : String(e);
        if (!msg.toLowerCase().includes("not found")) {
          errors.push(`${skillId}: ${msg}`);
        }
      }
    }

    await onRefresh();

    if (errors.length > 0) {
      throw new Error(`Delete failed:\n${errors.join("\n")}`);
    }
  };

  return (
    <>
      <SearchBar value={search} onChange={setSearch} />
      {browsing ? (
        <LoadingSkeletons />
      ) : (
        <div className="skill-grid">
          {isRemote ? (
            <>
              {filteredRemote.map(skill => (
                <RemoteSkillCard
                  key={skill.folder_name}
                  skill={skill}
                  installed={installedFolders.has(skill.folder_name)}
                  onClick={() => setSelectedRemoteSkill(skill)}
                />
              ))}
              {filteredRemote.length === 0 && (
                <div className="skill-grid-empty">
                  <div className="skill-grid-empty-icon">🔍</div>
                  <div className="skill-grid-empty-title">
                    {search ? t("noResultsFound") : t("noSkillsAvailable")}
                  </div>
                  <div className="skill-grid-empty-desc">
                    {search ? t("tryDifferentSearch") : t("noSkillsInSource")}
                  </div>
                </div>
              )}
            </>
          ) : (
            <>
              {mergedLocal.map(entry => (
                <SkillCard
                  key={entry.skill.folder_name}
                  skill={entry.skill}
                  enabledAgents={enabledAgents}
                  originAgents={entry.originAgents}
                  syncedMap={entry.syncedMap}
                  syncSourceSkillId={entry.syncSourceSkillId}
                  onOpen={() => { setSelectedFolder(entry.skill.folder_name); }}
                  onSync={onRefresh}
                />
              ))}
              {mergedLocal.length === 0 && (
                <div className="skill-grid-empty">
                  <div className="skill-grid-empty-icon">{search ? "🔍" : "✨"}</div>
                  <div className="skill-grid-empty-title">
                    {search ? t("noSkillsMatchSearch") : t("noSkillsYet")}
                  </div>
                  <div className="skill-grid-empty-desc">
                    {search ? t("tryDifferentSearch") : t("enableAgentsHint")}
                  </div>
                </div>
              )}
            </>
          )}
        </div>
      )}

      {selectedEntry && (
        <SkillModal
          skill={selectedEntry.skill}
          enabledAgents={enabledAgents}
          originAgents={selectedEntry.originAgents}
          syncedMap={selectedEntry.syncedMap}
          syncSourceSkillId={selectedEntry.syncSourceSkillId}
          onSync={onRefresh}
          onDelete={() => handleDelete(selectedEntry.skill.folder_name)}
          onClose={() => { setSelectedFolder(null); }}
          sources={sources}
        />
      )}

      {selectedRemoteSkill && onInstall && (
        <RemoteSkillModal
          skill={selectedRemoteSkill}
          enabledAgents={enabledAgents}
          installed={installedFolders.has(selectedRemoteSkill.folder_name)}
          onInstall={async (sourceId, folderName, targetAgent, force) => {
            await onInstall(sourceId, folderName, targetAgent, force);
            onRefresh();
          }}
          onClose={() => setSelectedRemoteSkill(null)}
        />
      )}
    </>
  );
}
