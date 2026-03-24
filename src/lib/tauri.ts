import { invoke } from "@tauri-apps/api/core";
import type { Agent, AgentDirStatus, AgentVersion, LatestVersions, RemoteSkill, Skill, Source } from "../types";

export const api = {
  listSkills: () => invoke<Skill[]>("list_skills"),
  scanSkills: () => invoke<Skill[]>("scan_skills"),
  deleteSkill: (skillId: string) => invoke("delete_skill", { skillId }),
  getSkillContent: (skillId: string) => invoke<string>("get_skill_content", { skillId }),
  updateSkillMeta: (skillId: string, tags?: string, notes?: string) =>
    invoke("update_skill_meta", { skillId, tags, notes }),
  syncSkill: (skillId: string, targetAgent: string) =>
    invoke("sync_skill", { skillId, targetAgent }),
  unsyncSkill: (skillId: string, agent: string) =>
    invoke("unsync_skill", { skillId, agent }),
  getAgents: () => invoke<Agent[]>("get_agents"),
  updateAgent: (agentId: string, enabled: boolean, skillDir?: string) =>
    invoke("update_agent", { agentId, enabled, skillDir }),
  getAgentVersions: () => invoke<AgentVersion[]>("get_agent_versions"),
  getLatestVersions: () => invoke<LatestVersions>("get_latest_versions"),
  listSources: () => invoke<Source[]>("list_sources"),
  addSource: (name: string, sourceType: string, url?: string, folderId?: string, token?: string) =>
    invoke<Source>("add_source", { name, sourceType, url, folderId, token }),
  removeSource: (sourceId: string) => invoke("remove_source", { sourceId }),
  browseSource: (sourceId: string) =>
    invoke<RemoteSkill[]>("browse_source", { sourceId }),
  installSkill: (sourceId: string, folderName: string, targetAgent: string, force?: boolean) =>
    invoke("install_skill", { sourceId, folderName, targetAgent, force }),
  uploadSkill: (sourceId: string, skillId: string, force?: boolean) =>
    invoke<void>("upload_skill", { sourceId, skillId, force }),
  getRemoteSkillContent: (sourceId: string, folderName: string) =>
    invoke<string | null>("get_remote_skill_content", { sourceId, folderName }),
  getConfig: (key: string) =>
    invoke<string | null>("get_config", { key }),
  setConfig: (key: string, value: string) =>
    invoke<void>("set_config", { key, value }),
  getAutostart: () => invoke<boolean>("get_autostart"),
  setAutostart: (enabled: boolean) =>
    invoke<void>("set_autostart", { enabled }),
  checkAgentDir: (agentId: string) =>
    invoke<AgentDirStatus>("check_agent_dir", { agentId }),
  pickAgentDir: () =>
    invoke<string | null>("pick_agent_dir"),
};
