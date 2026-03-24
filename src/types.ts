export interface Agent {
  id: string;
  enabled: boolean;
  skill_dir: string | null;
}

export interface Skill {
  id: string;
  folder_name: string;
  origin_agent: string;
  name: string | null;
  description: string | null;
  tags: string | null;
  notes: string | null;
  discovered_at: number | null;
  updated_at: number | null;
  synced_to: string[];
}

export interface Source {
  id: string;
  name: string;
  type: "gitlab" | "gdrive";
  url: string | null;
  folder_id: string | null;
  added_at: number | null;
}

export interface AgentVersion {
  id: string;
  installed: boolean;
  current_version: string | null;
}

export interface RemoteSkill {
  folder_name: string;
  name: string | null;
  description: string | null;
  source_id: string;
  source_name: string;
  updated_at: string | null;
  updated_by: string | null;
}

export interface LatestVersions {
  claude: string | null;
  codex: string | null;
  gemini: string | null;
}

export type AgentDirStatus =
  | { status: "Ok"; path: string }
  | { status: "NotInstalled"; install_cmd: string }
  | { status: "DirMissing"; path: string };

export interface AppError {
  kind: string;
  message: string;
}
