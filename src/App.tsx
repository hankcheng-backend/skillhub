import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api } from "./lib/tauri";
import type { RemoteSkill, Skill, Source } from "./types";
import { Layout } from "./components/Layout";
import { Sidebar } from "./components/Sidebar";
import { SkillGrid } from "./components/SkillGrid";
import { SettingsGeneral } from "./components/SettingsGeneral";
import { AddSourceDialog } from "./components/AddSourceDialog";
import { UpdateTokenDialog } from "./components/UpdateTokenDialog";
import "./styles/globals.css";

const SOURCE_AUTO_REFRESH_SECONDS = 15;
const SOURCE_AUTO_REFRESH_MS = SOURCE_AUTO_REFRESH_SECONDS * 1000;

function App() {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [sources, setSources] = useState<Source[]>([]);
  const [activeSource, setActiveSource] = useState("local");
  const [showSettings, setShowSettings] = useState(false);
  const [showAddSource, setShowAddSource] = useState(false);
  const [remoteSkills, setRemoteSkills] = useState<RemoteSkill[]>([]);
  const [browsing, setBrowsing] = useState(false);
  const [expiredSourceIds, setExpiredSourceIds] = useState<Set<string>>(new Set());
  const [tokenUpdateSourceId, setTokenUpdateSourceId] = useState<string | null>(null);

  const loadSkills = async () => {
    const s = await api.listSkills();
    setSkills(s);
  };

  const loadSources = async () => {
    const s = await api.listSources();
    setSources(s);
  };

  useEffect(() => {
    loadSkills();
    loadSources();
    const unlisten = listen("skills-changed", () => loadSkills());

    return () => { unlisten.then(fn => fn()); };
  }, []);

  useEffect(() => {
    if (activeSource === "local") {
      setRemoteSkills([]);
      setBrowsing(false);
      return;
    }

    let cancelled = false;
    let inFlight = false;
    let hasLoadedOnce = false;

    const refreshRemoteSkills = async () => {
      if (cancelled || inFlight) return;
      inFlight = true;
      const isInitialLoad = !hasLoadedOnce;

      if (isInitialLoad) {
        setBrowsing(true);
      }

      try {
        const latest = await api.browseSource(activeSource);
        if (!cancelled) {
          setRemoteSkills(latest);
          hasLoadedOnce = true;
        }
      } catch (e: unknown) {
        const err = e as { kind?: string; message?: string };
        if (err.kind === "TokenExpired") {
          setExpiredSourceIds(prev => new Set([...prev, activeSource]));
        }
        if (!cancelled && isInitialLoad) {
          setRemoteSkills([]);
        }
      } finally {
        if (!cancelled && isInitialLoad) {
          setBrowsing(false);
        }
        inFlight = false;
      }
    };

    void refreshRemoteSkills();
    const timer = setInterval(() => {
      void refreshRemoteSkills();
    }, SOURCE_AUTO_REFRESH_MS);

    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, [activeSource]);

  const handleInstall = async (sourceId: string, folderName: string, targetAgent: string, force?: boolean) => {
    await api.installSkill(sourceId, folderName, targetAgent, force);
  };

  const handleSelectSource = (sourceId: string) => {
    setActiveSource(sourceId);
    if (showSettings) {
      setShowSettings(false);
      void loadSkills();
    }
  };

  const handleTokenUpdateSuccess = async (sourceId: string) => {
    setExpiredSourceIds(prev => {
      const next = new Set(prev);
      next.delete(sourceId);
      return next;
    });
    setTokenUpdateSourceId(null);
    setActiveSource(sourceId);
    setBrowsing(true);
    try {
      const latest = await api.browseSource(sourceId);
      setRemoteSkills(latest);
    } catch {
      setRemoteSkills([]);
    } finally {
      setBrowsing(false);
    }
  };

  return (
    <Layout
      sidebar={
        <Sidebar
          sources={sources}
          activeSource={activeSource}
          onSelectSource={handleSelectSource}
          onAddSource={() => setShowAddSource(true)}
          expiredSourceIds={expiredSourceIds}
          onTokenUpdate={(sourceId) => setTokenUpdateSourceId(sourceId)}
        />
      }
      onSettings={() => setShowSettings(!showSettings)}
    >
      {showSettings ? (
        <SettingsGeneral onBack={() => { setShowSettings(false); void loadSkills(); }} />
      ) : (
        <SkillGrid
          skills={skills}
          onRefresh={loadSkills}
          activeSource={activeSource}
          remoteSkills={remoteSkills}
          browsing={browsing}
          onInstall={handleInstall}
          sources={sources}
        />
      )}
      {showAddSource && <AddSourceDialog onClose={() => setShowAddSource(false)} onAdded={loadSources} />}
      {tokenUpdateSourceId && (
        <UpdateTokenDialog
          sourceId={tokenUpdateSourceId}
          sourceName={sources.find(s => s.id === tokenUpdateSourceId)?.name ?? ""}
          onClose={() => setTokenUpdateSourceId(null)}
          onSuccess={() => { void handleTokenUpdateSuccess(tokenUpdateSourceId); }}
        />
      )}
    </Layout>
  );
}

export default App;
