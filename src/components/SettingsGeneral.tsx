import { useEffect, useState } from "react";
import { api } from "../lib/tauri";
import { t, getCurrentLang, setLang } from "../lib/i18n";
import type { Agent, AgentDirStatus } from "../types";

interface SettingsGeneralProps {
  onBack: () => void;
}

function Toggle({ checked, onChange }: { checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <label className="toggle-switch">
      <input type="checkbox" checked={checked} onChange={e => onChange(e.target.checked)} />
      <span className="toggle-track" />
    </label>
  );
}

export function SettingsGeneral({ onBack }: SettingsGeneralProps) {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [tab, setTab] = useState<"general" | "versions" | "mcp">("general");
  const [autostart, setAutostart] = useState(false);
  const [toast, setToast] = useState<string | null>(null);

  useEffect(() => {
    api.getAgents().then(setAgents);
    api.getAutostart().then(setAutostart).catch(() => setAutostart(false));
  }, []);

  const showToast = (msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 3000);
  };

  const [dirAlert, setDirAlert] = useState<{ agentId: string; status: AgentDirStatus } | null>(null);

  const toggleAgent = async (agent: Agent) => {
    // When enabling, check if config dir exists first
    if (!agent.enabled) {
      const status = await api.checkAgentDir(agent.id);
      if (status.status !== "Ok") {
        setDirAlert({ agentId: agent.id, status });
        return;
      }
    }
    await api.updateAgent(agent.id, !agent.enabled, agent.skill_dir ?? undefined);
    const updated = await api.getAgents();
    setAgents(updated);
  };

  const pickDir = async (agent: Agent) => {
    const picked = await api.pickAgentDir();
    if (picked) {
      await api.updateAgent(agent.id, agent.enabled, picked);
      const updated = await api.getAgents();
      setAgents(updated);
    }
  };

  const getResolvedPath = (agent: Agent): string => {
    if (agent.skill_dir) {
      // Backward compatibility: old builds might have saved ".../skills".
      return agent.skill_dir.replace(/[\\/]+skills$/i, "");
    }
    // Default: ~/.{agent_id}
    return `~/.${agent.id}`;
  };

  const toggleAutostart = async (enabled: boolean) => {
    try {
      await api.setAutostart(enabled);
      setAutostart(enabled);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      showToast(msg);
    }
  };

  const installCmds: Record<string, string> = {
    claude: "curl -fsSL https://claude.ai/install.sh | bash",
    codex: "npm install -g @openai/codex",
    gemini: "npm install -g @google/gemini-cli",
  };

  return (
    <div className="settings-container">
      {toast && <div className="toast">{toast}</div>}

      <div className="settings-back-row">
        <button className="btn-ghost" onClick={onBack}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" style={{ width: 14, height: 14 }}>
            <polyline points="15 18 9 12 15 6" />
          </svg>
          {t("back")}
        </button>
        <h2 className="settings-title">{t("settings")}</h2>
      </div>

      <div className="settings-tabs">
        {(["general", "versions", "mcp"] as const).map(tabId => (
          <button
            key={tabId}
            onClick={() => setTab(tabId)}
            className={`settings-tab${tab === tabId ? " active" : ""}`}
          >
            {tabId === "general" ? t("general") : tabId === "versions" ? t("versions") : t("mcp")}
          </button>
        ))}
      </div>

      {tab === "general" && (
        <div>
          <h3 className="settings-section-title">{t("agentManagement")}</h3>
          <div className="settings-section-card">
            {agents.map(agent => (
              <div key={agent.id} className="settings-agent-row">
                <div className="settings-agent-row-top">
                  <span className="settings-row-label settings-version-id">{agent.id}</span>
                  <Toggle checked={agent.enabled} onChange={() => toggleAgent(agent)} />
                </div>
                <div className="settings-agent-row-path">
                  <span className="settings-agent-path-text" title={getResolvedPath(agent)}>
                    {getResolvedPath(agent)}
                  </span>
                  <button
                    className="settings-agent-path-edit"
                    onClick={() => pickDir(agent)}
                    title={t("locateDir")}
                  >
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={{ width: 12, height: 12 }}>
                      <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
                      <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
                    </svg>
                  </button>
                </div>
              </div>
            ))}
          </div>

          {/* Agent directory alert dialog */}
          {dirAlert && (
            <div className="dialog-overlay" onClick={() => setDirAlert(null)}>
              <div className="dir-alert-dialog" onClick={e => e.stopPropagation()}>
                {dirAlert.status.status === "NotInstalled" ? (
                  <>
                    <div className="dir-alert-icon dir-alert-icon--warn">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={{ width: 24, height: 24 }}>
                        <circle cx="12" cy="12" r="10" />
                        <line x1="12" y1="8" x2="12" y2="12" />
                        <line x1="12" y1="16" x2="12.01" y2="16" />
                      </svg>
                    </div>
                    <p className="dir-alert-msg">
                      {t("agentNotInstalled").replace("{agent}", dirAlert.agentId)}
                    </p>
                    <code className="code-block">{dirAlert.status.install_cmd}</code>
                  </>
                ) : dirAlert.status.status === "DirMissing" ? (
                  <>
                    <div className="dir-alert-icon dir-alert-icon--info">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={{ width: 24, height: 24 }}>
                        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
                        <line x1="12" y1="11" x2="12" y2="17" />
                        <line x1="12" y1="11" x2="12.01" y2="11" />
                      </svg>
                    </div>
                    <p className="dir-alert-msg">
                      {t("agentDirMissing").replace("{agent}", dirAlert.agentId)}
                    </p>
                    <button
                      className="btn-primary"
                      onClick={async () => {
                        const picked = await api.pickAgentDir();
                        if (picked) {
                          await api.updateAgent(dirAlert.agentId, true, picked);
                          const updated = await api.getAgents();
                          setAgents(updated);
                          setDirAlert(null);
                        }
                      }}
                    >
                      {t("locateDir")}
                    </button>
                  </>
                ) : null}
                <button className="btn-ghost dir-alert-close" onClick={() => setDirAlert(null)}>
                  {t("cancel")}
                </button>
              </div>
            </div>
          )}

          <h3 className="settings-section-title spaced">{t("autostart")}</h3>
          <div className="settings-section-card">
            <div className="settings-row">
              <div>
                <div className="settings-row-label">{t("autostart")}</div>
                <div className="settings-row-sub">{t("autostartDesc")}</div>
              </div>
              <Toggle checked={autostart} onChange={toggleAutostart} />
            </div>
          </div>

          <h3 className="settings-section-title spaced">{t("language")}</h3>
          <div className="settings-section-card">
            <div className="settings-row">
              <span className="settings-row-label">{t("language")}</span>
              <select
                value={getCurrentLang()}
                onChange={e => setLang(e.target.value as "zh-TW" | "en")}
                className="settings-select"
              >
                <option value="zh-TW">中文（繁體）</option>
                <option value="en">English</option>
              </select>
            </div>
          </div>
        </div>
      )}

      {tab === "versions" && <VersionsTab installCmds={installCmds} showToast={showToast} />}
      {tab === "mcp" && <McpTab showToast={showToast} />}
    </div>
  );
}

const AGENT_DISPLAY: Record<string, { label: string; icon: string; color: string }> = {
  claude: { label: "Claude Code", icon: "🟠", color: "#DA7756" },
  codex: { label: "Codex", icon: "🟢", color: "#34A853" },
  gemini: { label: "Gemini CLI", icon: "🔵", color: "#4A90D9" },
};

const AGENTS_ORDER = ["claude", "codex", "gemini"];

interface VersionCardData {
  id: string;
  installed: boolean;
  currentVersion: string | null;
  latestVersion: string | null;
  localLoading: boolean;
  latestLoading: boolean;
}

function VersionCard({
  data,
  installCmd,
  onShowCommand,
}: {
  data: VersionCardData;
  installCmd: string;
  onShowCommand: (agentId: string, command: string, mode: "install" | "update") => void;
}) {
  const display = AGENT_DISPLAY[data.id] || { label: data.id, icon: "⬜", color: "#999" };
  const isFullyLoading = data.localLoading;

  const isLatest =
    data.installed &&
    data.currentVersion &&
    data.latestVersion &&
    data.currentVersion === data.latestVersion;

  const isOutdated =
    data.installed &&
    data.currentVersion &&
    data.latestVersion &&
    data.currentVersion !== data.latestVersion;

  return (
    <div className="version-card">
      <div className="version-card-header">
        <span className="version-card-dot" style={{ background: display.color }} />
        <span className="version-card-name">{display.label}</span>
      </div>

      {isFullyLoading ? (
        <div className="version-card-body">
          <div className="skeleton-line" style={{ width: '70%', height: 10, marginBottom: 8 }} />
          <div className="skeleton-line" style={{ width: '50%', height: 10, marginBottom: 0 }} />
        </div>
      ) : (
        <div className="version-card-body">
          {data.installed ? (
            <>
              <div className="version-card-row">
                <span className="version-card-label">{t("versions")}</span>
                <span className="version-card-value">{data.currentVersion}</span>
              </div>
              <div className="version-card-row">
                <span className="version-card-label">{t("latest")}</span>
                {data.latestLoading ? (
                  <span className="skeleton-line" style={{ width: 50, height: 10, margin: 0, display: 'inline-block' }} />
                ) : (
                  <span className="version-card-value">{data.latestVersion || "—"}</span>
                )}
              </div>
              <div className="version-card-status-row">
                {data.latestLoading ? null : isLatest ? (
                  <span className="version-badge version-badge--latest">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" style={{ width: 11, height: 11 }}>
                      <polyline points="20 6 9 17 4 12" />
                    </svg>
                    {t("upToDate")}
                  </span>
                ) : isOutdated ? (
                  <button
                    type="button"
                    className="version-badge version-badge--outdated version-badge--action"
                    onClick={() => onShowCommand(data.id, installCmd, "update")}
                  >
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" style={{ width: 11, height: 11 }}>
                      <polyline points="18 15 12 9 6 15" />
                    </svg>
                    {t("updateAvailable")}
                  </button>
                ) : null}
              </div>
            </>
          ) : (
            <>
              <div className="version-card-not-installed">
                <button
                  type="button"
                  className="version-not-installed-action"
                  onClick={() => onShowCommand(data.id, installCmd, "install")}
                >
                  {t("notInstalled")}
                </button>
              </div>
            </>
          )}
        </div>
      )}
    </div>
  );
}

function VersionsTab({
  installCmds,
  showToast,
}: {
  installCmds: Record<string, string>;
  showToast: (msg: string) => void;
}) {
  const [cards, setCards] = useState<VersionCardData[]>(() =>
    AGENTS_ORDER.map(id => ({
      id,
      installed: false,
      currentVersion: null,
      latestVersion: null,
      localLoading: true,
      latestLoading: true,
    }))
  );
  const [refreshing, setRefreshing] = useState(false);
  const [commandDialog, setCommandDialog] = useState<{
    agentId: string;
    command: string;
    mode: "install" | "update";
  } | null>(null);

  const loadVersions = async () => {
    setRefreshing(true);
    setCards(prev => prev.map(c => ({ ...c, localLoading: true, latestLoading: true })));

    // Phase 1: local versions come back fast — show them immediately
    api.getAgentVersions()
      .then(versions => {
        setCards(prev => prev.map(c => {
          const v = versions.find(ver => ver.id === c.id);
          return {
            ...c,
            installed: v?.installed ?? false,
            currentVersion: v?.current_version ?? null,
            localLoading: false,
          };
        }));
      })
      .catch(() => {
        setCards(prev => prev.map(c => ({ ...c, localLoading: false })));
      });

    // Phase 2: latest versions from npm — fills in when ready
    api.getLatestVersions()
      .then(latest => {
        const latestMap: Record<string, string | null> = {
          claude: latest.claude,
          codex: latest.codex,
          gemini: latest.gemini,
        };
        setCards(prev => prev.map(c => ({
          ...c,
          latestVersion: latestMap[c.id] ?? null,
          latestLoading: false,
        })));
      })
      .catch(() => {
        setCards(prev => prev.map(c => ({ ...c, latestLoading: false })));
      })
      .finally(() => {
        setRefreshing(false);
      });
  };

  useEffect(() => {
    loadVersions();
  }, []);

  const handleShowCommand = (agentId: string, command: string, mode: "install" | "update") => {
    setCommandDialog({ agentId, command, mode });
  };

  const handleCopyCommand = async () => {
    if (!commandDialog) return;
    try {
      await navigator.clipboard.writeText(commandDialog.command);
      showToast(t("commandCopied"));
    } catch {
      showToast(t("copyFailed"));
    }
  };

  const isAnyLoading = cards.some(c => c.localLoading || c.latestLoading);
  const commandTitle = commandDialog?.mode === "update" ? t("updateCommandTitle") : t("installCommandTitle");
  const commandAgentLabel = commandDialog ? (AGENT_DISPLAY[commandDialog.agentId]?.label || commandDialog.agentId) : "";

  return (
    <div>
      <div className="versions-header">
        <h3 className="settings-section-title" style={{ margin: 0 }}>{t("agentVersions")}</h3>
        <button
          className="btn-ghost"
          onClick={loadVersions}
          disabled={isAnyLoading}
          title={t("refresh")}
        >
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2.5"
            strokeLinecap="round"
            strokeLinejoin="round"
            className={refreshing ? "spin" : ""}
            style={{ width: 15, height: 15 }}
          >
            <polyline points="23 4 23 10 17 10" />
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
          </svg>
        </button>
      </div>
      <div className="versions-grid">
        {cards.map(card => (
          <VersionCard
            key={card.id}
            data={card}
            installCmd={installCmds[card.id] || ""}
            onShowCommand={handleShowCommand}
          />
        ))}
      </div>

      {commandDialog && (
        <div className="dialog-overlay" onClick={() => setCommandDialog(null)}>
          <div className="version-command-dialog" onClick={e => e.stopPropagation()}>
            <div className="version-command-title">{commandTitle}</div>
            <div className="version-command-subtitle">
              {commandAgentLabel} · {t("runInTerminal")}
            </div>
            <code className="code-block">{commandDialog.command}</code>
            <div className="version-command-actions">
              <button className="btn-secondary" onClick={() => setCommandDialog(null)}>
                {t("cancel")}
              </button>
              <button className="btn-primary" onClick={() => { void handleCopyCommand(); }}>
                {t("copyCommand")}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function McpTab({ showToast }: { showToast: (msg: string) => void }) {
  const [port, setPort] = useState("9800");
  const [portError, setPortError] = useState<string | null>(null);
  const [restartRequired, setRestartRequired] = useState(false);
  const [mcpStatus, setMcpStatus] = useState<"running" | "stopped">("stopped");
  const [savedPort, setSavedPort] = useState("9800");

  useEffect(() => {
    api.getConfig("mcp_port").then(v => {
      const p = v ?? "9800";
      setPort(p);
      setSavedPort(p);
    });
  }, []);

  useEffect(() => {
    const checkHealth = async () => {
      try {
        const res = await fetch(`http://127.0.0.1:${savedPort}/health`);
        setMcpStatus(res.ok ? "running" : "stopped");
      } catch {
        setMcpStatus("stopped");
      }
    };
    checkHealth();
    const interval = setInterval(checkHealth, 5000);
    return () => clearInterval(interval);
  }, [savedPort]);

  const validatePort = (val: string): boolean => {
    const n = parseInt(val, 10);
    if (isNaN(n) || String(n) !== val || n < 1024 || n > 65535) {
      setPortError(t("mcpPortInvalid"));
      return false;
    }
    setPortError(null);
    return true;
  };

  const savePort = async () => {
    if (!validatePort(port)) return;
    try {
      await api.setConfig("mcp_port", port);
      setSavedPort(port);
      setRestartRequired(true);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      showToast(msg);
    }
  };

  return (
    <div>
      <h3 className="settings-section-title">{t("mcpStatus")}</h3>
      <div className="mcp-status-row" style={{ marginBottom: 20 }}>
        <span
          className={`mcp-status-dot${mcpStatus === "running" ? " mcp-status-dot--running" : " mcp-status-dot--stopped"}`}
        />
        <span className="mcp-status-text">
          {mcpStatus === "running" ? t("mcpRunning") : t("mcpStopped")}
        </span>
      </div>

      <h3 className="settings-section-title">{t("mcpPort")}</h3>
      <div className="settings-section-card">
        <div className="settings-row" style={{ flexDirection: 'column', alignItems: 'flex-start', gap: 8 }}>
          <div className="settings-row-sub" style={{ margin: 0 }}>{t("mcpPortDesc")}</div>
          <div>
            <input
              value={port}
              onChange={e => { setPort(e.target.value); setPortError(null); setRestartRequired(false); }}
              onBlur={savePort}
              onKeyDown={e => { if (e.key === "Enter") e.currentTarget.blur(); }}
              className={`mcp-port-input${portError ? " mcp-port-input--error" : ""}`}
            />
            {portError && <div className="mcp-error">{portError}</div>}
            {restartRequired && !portError && (
              <div className="mcp-hint">{t("mcpRestartRequired")}</div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
