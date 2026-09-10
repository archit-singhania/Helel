import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { ActivityBar, type ActivityView } from "./components/ActivityBar";
import { SidePanel } from "./components/SidePanel";
import { Workbench } from "./components/Workbench";
import { DEFAULT_SETTINGS, loadSettings, projectName, saveSettings, type Settings, withRecentProject } from "./settings";

export function App() {
  const [settings, setSettings] = useState<Settings>(() => loadSettings());
  const [view, setView] = useState<ActivityView>("explorer");
  const [settingsOpen, setSettingsOpen] = useState(false);

  useEffect(() => saveSettings(settings), [settings]);
  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: light)");
    const applyTheme = () => {
      const resolved = settings.theme === "system" ? (media.matches ? "light" : "dark") : settings.theme;
      document.documentElement.dataset.theme = resolved;
    };
    applyTheme();
    media.addEventListener("change", applyTheme);
    return () => media.removeEventListener("change", applyTheme);
  }, [settings.theme]);

  async function chooseProject() {
    const selected = await open({ directory: true, multiple: false, title: "Open a project in Helel" });
    if (typeof selected === "string") setSettings((current) => withRecentProject(current, selected));
  }

  const currentProject = settings.recentProjects[0];
  return (
    <main className="app-shell" aria-label="Helel workspace">
      <header className="titlebar">
        <div className="traffic-space" aria-hidden="true" />
        <span className="wordmark"><span className="brand-mark">H</span> Helel</span>
        <span className="project-title">{currentProject ? projectName(currentProject) : "Welcome"}</span>
        <button className="icon-button title-action" onClick={() => setSettingsOpen(true)} aria-label="Open settings"><Icon name="settings" /></button>
      </header>
      <div className="workspace">
        <ActivityBar active={view} onChange={setView} />
        <SidePanel view={view} width={settings.sidebarWidth} recentProjects={settings.recentProjects} currentProject={currentProject} onOpenProject={chooseProject} onSelectProject={(path) => setSettings((current) => withRecentProject(current, path))} />
        <Workbench project={currentProject} sidebarWidth={settings.sidebarWidth} bottomHeight={settings.bottomPanelHeight} agentWidth={settings.agentWidth} onResize={(update) => setSettings((current) => ({ ...current, ...update }))} onOpenProject={chooseProject} />
      </div>
      <footer className="statusbar" aria-label="Status bar"><span><Icon name="branch" /> main</span><span className="status-grow">Local mode</span><span>No network AI</span><span>Phase 1</span></footer>
      {settingsOpen && <div className="dialog-backdrop" role="presentation" onMouseDown={() => setSettingsOpen(false)}><section className="settings-dialog" role="dialog" aria-modal="true" aria-labelledby="settings-title" onMouseDown={(event) => event.stopPropagation()}><div className="dialog-heading"><div><p className="overline">Preferences</p><h2 id="settings-title">Appearance</h2></div><button className="icon-button" onClick={() => setSettingsOpen(false)} aria-label="Close settings"><Icon name="close" /></button></div><fieldset><legend>Theme</legend><div className="segmented">{(["dark", "light", "system"] as const).map((theme) => <button key={theme} className={settings.theme === theme ? "selected" : ""} onClick={() => setSettings((current) => ({ ...current, theme }))}>{theme}</button>)}</div></fieldset><button className="secondary-button" onClick={() => setSettings({ ...DEFAULT_SETTINGS, recentProjects: settings.recentProjects })}>Reset panel layout</button></section></div>}
    </main>
  );
}

export function Icon({ name }: { name: "files" | "search" | "branch" | "agent" | "settings" | "folder" | "close" | "terminal" }) {
  const paths = {
    files: <><path d="M5 3h9l5 5v13H5z"/><path d="M14 3v5h5"/><path d="M9 13h6M9 17h6"/></>, search: <><circle cx="11" cy="11" r="7"/><path d="m16 16 5 5"/></>, branch: <><circle cx="6" cy="5" r="2"/><circle cx="6" cy="19" r="2"/><circle cx="18" cy="8" r="2"/><path d="M6 7v10M8 7c3 0 3 1 3 1h5M11 8v4c0 3-2 3-3 3H6"/></>, agent: <><rect x="4" y="6" width="16" height="13" rx="3"/><path d="M12 3v3M8 12h.01M16 12h.01M8 16h8"/></>, settings: <><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1-2.8 2.8-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.6v.2h-4V21a1.7 1.7 0 0 0-1-1.6 1.7 1.7 0 0 0-1.9.3l-.1.1L4.2 17l.1-.1a1.7 1.7 0 0 0 .3-1.9A1.7 1.7 0 0 0 3 14H2.8v-4H3a1.7 1.7 0 0 0 1.6-1 1.7 1.7 0 0 0-.3-1.9L4.2 7 7 4.2l.1.1a1.7 1.7 0 0 0 1.9.3A1.7 1.7 0 0 0 10 3v-.2h4V3a1.7 1.7 0 0 0 1 1.6 1.7 1.7 0 0 0 1.9-.3l.1-.1L19.8 7l-.1.1a1.7 1.7 0 0 0-.3 1.9 1.7 1.7 0 0 0 1.6 1h.2v4H21a1.7 1.7 0 0 0-1.6 1z"/></>, folder: <path d="M3 6h7l2 2h9v11H3z"/>, close: <path d="m6 6 12 12M18 6 6 18"/>, terminal: <path d="m5 7 4 4-4 4M11 17h8"/>,
  };
  return <svg viewBox="0 0 24 24" aria-hidden="true">{paths[name]}</svg>;
}
