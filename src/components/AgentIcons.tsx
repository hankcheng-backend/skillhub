import { api } from "../lib/tauri";
import { t } from "../lib/i18n";
import type { Skill } from "../types";
import React from "react";

const CLAUDE_SVG = (
  <svg viewBox="0 0 1200 1200" fill="white" xmlns="http://www.w3.org/2000/svg">
    <path d="M233.96 800.21L468.64 668.54l3.95-11.44-3.95-6.36h-11.44l-39.22-2.42-134.09-3.62-116.3-4.83L54.93 633.83l-26.58-5.96L0 592.75l2.74-17.48 23.84-16.03 34.15 2.98 75.46 5.15 113.24 7.81 82.15 4.83 121.69 12.65h19.33l2.74-7.81-6.6-4.83-5.16-4.83L346.39 495.79 219.54 411.87l-66.44-48.32-35.92-24.48-18.12-22.95-7.81-50.09 32.62-35.92 43.81 2.98 11.19 2.98 44.38 34.15 94.79 73.37 123.79 91.17 18.12 15.06 7.25-5.15.89-3.62-8.13-13.62-67.46-121.69-71.84-123.79-31.97-51.3-8.46-30.77c-2.98-12.64-5.15-23.27-5.15-36.24l37.13-50.42 20.54-6.6 49.53 6.6 20.86 18.12 30.77 70.39 49.85 110.82 77.32 150.68 22.63 44.7 12.08 41.4 4.51 12.64h7.81v-7.25l6.36-84.89 11.76-104.21 11.44-134.09 3.95-37.77 18.68-45.26 37.13-24.48 28.99 13.85 23.84 34.15-3.3 22.07-14.17 92.13-27.79 144.32-18.12 96.64h10.55l12.08-12.08 48.89-64.91 82.15-102.68 36.24-40.75 42.28-45.02 27.14-21.42h51.3l37.77 56.13-16.91 57.99-52.83 67.01-43.81 56.78-62.82 84.56-39.22 67.65 3.62 5.4 9.35-0.89 141.91-30.2 76.67-13.85 91.49-15.7 41.4 19.33 4.51 19.65-16.27 40.19-97.85 24.16-114.77 22.95-170.9 40.43-2.09 1.53 2.42 2.98 76.99 7.25 32.94 1.77 80.62 0 150.12 11.19 39.22 25.93 23.52 31.73-3.95 24.16-60.4 30.77-81.5-19.33-190.23-45.26-65.23-16.27-9.02 0v5.4l54.36 53.15 99.62 89.96 124.75 115.97 6.36 28.67-16.03 22.63-16.91-2.42-109.61-82.47-42.28-37.13-95.76-80.62h-6.36v8.46l22.07 32.3 116.54 175.17 6.04 53.72-8.46 17.48-30.2 10.55-33.18-6.04-68.21-95.76-70.39-107.84-56.78-96.64-6.93 3.95-33.5 360.89-15.7 18.44-36.24 13.85-30.2-22.95-16.03-37.13 16.03-73.37 19.33-95.76 15.7-76.11 14.17-94.55 8.46-31.41-.56-2.09-6.93.89-71.27 97.85-108.4 146.5-85.77 91.81-20.54 8.13-35.6-18.44 3.3-32.94 20.14-29.35 118.71-150.91 71.6-93.58 46.23-51.97-.32-7.81h-2.74L205.29 929.4l-56.13 7.25-24.16-22.63 2.98-37.13 11.44-12.08 94.79-65.23z" />
  </svg>
);

const OPENAI_SVG = (
  <svg viewBox="0 0 24 24" fill="white" xmlns="http://www.w3.org/2000/svg">
    <path d="M22.282 9.821a5.985 5.985 0 0 0-.516-4.91 6.046 6.046 0 0 0-6.51-2.9A6.065 6.065 0 0 0 4.981 4.18a5.985 5.985 0 0 0-3.998 2.9 6.046 6.046 0 0 0 .743 7.097 5.98 5.98 0 0 0 .51 4.911 6.051 6.051 0 0 0 6.515 2.9A5.985 5.985 0 0 0 13.26 24a6.056 6.056 0 0 0 5.772-4.206 5.99 5.99 0 0 0 3.997-2.9 6.056 6.056 0 0 0-.747-7.073zM13.26 22.43a4.476 4.476 0 0 1-2.876-1.04l.141-.081 4.779-2.758a.795.795 0 0 0 .392-.681v-6.737l2.02 1.168a.071.071 0 0 1 .038.052v5.583a4.504 4.504 0 0 1-4.494 4.494zM3.6 18.304a4.47 4.47 0 0 1-.535-3.014l.142.085 4.783 2.759a.771.771 0 0 0 .78 0l5.843-3.369v2.332a.08.08 0 0 1-.033.062L9.74 19.95a4.5 4.5 0 0 1-6.14-1.646zM2.34 7.896a4.485 4.485 0 0 1 2.366-1.973V11.6a.766.766 0 0 0 .388.676l5.815 3.355-2.02 1.168a.076.076 0 0 1-.071 0l-4.83-2.786A4.504 4.504 0 0 1 2.34 7.896zm16.597 3.855l-5.833-3.387L15.119 7.2a.076.076 0 0 1 .071 0l4.83 2.791a4.494 4.494 0 0 1-.676 8.105v-5.678a.79.79 0 0 0-.407-.667zm2.01-3.023l-.141-.085-4.774-2.782a.776.776 0 0 0-.785 0L9.409 9.23V6.897a.066.066 0 0 1 .028-.061l4.83-2.787a4.5 4.5 0 0 1 6.68 4.66zm-12.64 4.135l-2.02-1.164a.08.08 0 0 1-.038-.057V6.075a4.5 4.5 0 0 1 7.375-3.453l-.142.08L8.704 5.46a.795.795 0 0 0-.393.681zm1.097-2.365l2.602-1.5 2.607 1.5v2.999l-2.597 1.5-2.607-1.5z" />
  </svg>
);

const GEMINI_SVG = (
  <svg viewBox="0 0 24 24" fill="white" xmlns="http://www.w3.org/2000/svg">
    <path d="M12 2C12 2 12.8 7.2 14.8 9.2C16.8 11.2 22 12 22 12C22 12 16.8 12.8 14.8 14.8C12.8 16.8 12 22 12 22C12 22 11.2 16.8 9.2 14.8C7.2 12.8 2 12 2 12C2 12 7.2 11.2 9.2 9.2C11.2 7.2 12 2 12 2Z" />
  </svg>
);

const AGENT_CONFIG: Record<string, { color: string; icon: React.ReactNode }> = {
  claude: { color: "#DA7756", icon: CLAUDE_SVG },
  codex:  { color: "#34A853", icon: OPENAI_SVG },
  gemini: { color: "#4A90D9", icon: GEMINI_SVG },
};

const AGENTS = ["claude", "codex", "gemini"];

interface AgentIconsProps {
  skill: Skill;
  enabledAgents: string[];
  originAgents?: string[];
  syncedMap?: Record<string, string>;
  syncSourceSkillId?: string;
  onSync: () => void;
  onError?: (message: string) => void;
}

export function AgentIcons({
  skill,
  enabledAgents,
  originAgents,
  syncedMap,
  syncSourceSkillId,
  onSync,
  onError,
}: AgentIconsProps) {
  const effectiveOriginAgents = originAgents ?? [skill.origin_agent];
  const effectiveSyncedMap = syncedMap ?? Object.fromEntries(skill.synced_to.map(agent => [agent, skill.id]));
  const sourceSkillId = syncSourceSkillId ?? skill.id;

  const handleClick = async (agent: string) => {
    if (effectiveOriginAgents.includes(agent)) return;
    const linkedSkillId = effectiveSyncedMap[agent];
    try {
      if (linkedSkillId) {
        await api.unsyncSkill(linkedSkillId, agent);
      } else {
        await api.syncSkill(sourceSkillId, agent);
      }
      onSync();
    } catch (err: unknown) {
      const message =
        (err as { message?: string })?.message || t("syncFailed");
      if (onError) {
        onError(message);
      } else {
        console.error("Sync operation failed:", err);
      }
    }
  };

  return (
    <div className="agent-badge-row">
      {AGENTS.filter(a => enabledAgents.includes(a)).map(agent => {
        const cfg = AGENT_CONFIG[agent];
        if (!cfg) return null;
        const isOrigin = effectiveOriginAgents.includes(agent);
        const isSynced = Boolean(effectiveSyncedMap[agent]);
        const isActive = isOrigin || isSynced;
        const isClickable = !isOrigin;
        const title = isOrigin
          ? t("origin")
          : isSynced
            ? t("syncedClickToUnsync")
            : t("clickToSync");

        const classes = [
          "agent-badge",
          isOrigin ? "agent-badge--origin" : "",
          isActive ? "" : "agent-badge--inactive",
          isClickable ? "agent-badge--clickable" : "",
        ].filter(Boolean).join(" ");

        return (
          <span
            key={agent}
            className={classes}
            onClick={e => { e.stopPropagation(); handleClick(agent); }}
            title={title}
            style={{
              '--badge-color': cfg.color,
            } as React.CSSProperties}
          >
            {cfg.icon}
          </span>
        );
      })}
    </div>
  );
}
