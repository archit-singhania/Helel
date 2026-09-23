import { useCallback, useEffect, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { workspaceApi } from "./api";
import { ActivityBar, type ActivityView } from "./components/ActivityBar";
import { SidePanel } from "./components/SidePanel";
import { Workbench } from "./components/Workbench";
import { CommandPalette, type CommandItem } from "./components/CommandPalette";
import type { EditorTab, Problem, SearchMatch, TreeEntry } from "./ide";
import { isDirty, markSaved, updateTab, upsertTab } from "./ide";
import { appendTerminalHistory, type GitSummary, type ProcessExit, type ProcessOutput } from "./system";
import { describeTool, requiresApproval, type AgentSession, type CodeIndex } from "./intelligence";
import { DEFAULT_SETTINGS, loadSettings, projectName, saveSettings, type Settings, withRecentProject } from "./settings";

type ProjectSnapshot = { tree: TreeEntry[]; index: CodeIndex; agents: AgentSession[] };
let openingProject: { path: string; promise: Promise<ProjectSnapshot> } | undefined;

function openProjectOnce(path: string): Promise<ProjectSnapshot> {
  if (openingProject?.path === path) return openingProject.promise;
  const promise = (async () => {
    const tree = await workspaceApi.open(path);
    const [index, agents] = await Promise.all([workspaceApi.loadIndex(), workspaceApi.listAgents()]);
    return { tree, index, agents };
  })();
  openingProject = { path, promise };
  void promise.finally(() => { if (openingProject?.promise === promise) openingProject = undefined; }).catch(() => undefined);
  return promise;
}

export function App() {
  const [settings, setSettings] = useState<Settings>(() => loadSettings());
  const [view, setView] = useState<ActivityView>("explorer");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [tree, setTree] = useState<TreeEntry[]>([]);
  const [tabs, setTabs] = useState<EditorTab[]>([]);
  const [activePath, setActivePath] = useState<string>();
  const [problems, setProblems] = useState<Problem[]>([]);
  const [searchResults, setSearchResults] = useState<SearchMatch[]>([]);
  const [message, setMessage] = useState("Ready");
  const [terminalOutput, setTerminalOutput] = useState<string[]>(["Helel local terminal — commands run without a shell."]);
  const [git, setGit] = useState<GitSummary>();
  const [activeProcess, setActiveProcess] = useState<number>();
  const [index, setIndex] = useState<CodeIndex>();
  const [agents, setAgents] = useState<AgentSession[]>([]);
  const [modelRunning, setModelRunning] = useState(false);
  const [mcpRunning, setMcpRunning] = useState(false);
  const [mcpSummary, setMcpSummary] = useState("");
  const [currentProject, setCurrentProject] = useState<string>();
  const initialProject = useRef(settings.recentProjects[0]);
  const startupAttempted = useRef(false);
  const projectOpening = useRef(false);

  useEffect(() => saveSettings(settings), [settings]);
  useEffect(() => { const media = window.matchMedia("(prefers-color-scheme: light)"); const apply = () => { const theme = settings.theme === "system" ? (media.matches ? "light" : "dark") : settings.theme; document.documentElement.dataset.theme = theme; document.documentElement.style.colorScheme = theme; }; apply(); media.addEventListener("change", apply); return () => media.removeEventListener("change", apply); }, [settings.theme]);

  const loadProject = useCallback(async (path: string) => {
    if (projectOpening.current) { setMessage("A project is already opening; wait for it to finish"); return; }
    if (path === currentProject) return;
    if (activeProcess) { setMessage("Stop the active terminal before switching projects"); return; }
    if (tabs.some(isDirty) && !window.confirm("Discard unsaved edits and switch projects?")) return;
    projectOpening.current = true;
    setCurrentProject(undefined);
    setTree([]); setTabs([]); setActivePath(undefined); setProblems([]);
    setSearchResults([]); setGit(undefined); setIndex(undefined); setAgents([]);
    try {
      setMessage(`Opening ${projectName(path)}…`);
      const snapshot = await openProjectOnce(path);
      setTree(snapshot.tree); setIndex(snapshot.index); setAgents(snapshot.agents);
      setSettings((current) => withRecentProject(current, path));
      setCurrentProject(path);
      setMessage(`Opened ${projectName(path)} · ${snapshot.index.files.length} indexed files`);
    } catch (error) { setMessage(String(error)); }
    finally { projectOpening.current = false; }
  }, [activeProcess, currentProject, tabs]);
  useEffect(() => {
    if (startupAttempted.current) return;
    startupAttempted.current = true;
    if (initialProject.current) void loadProject(initialProject.current);
  }, [loadProject]);
  useEffect(() => {
    if (!currentProject) return;
    let active = true;
    const watch = async () => {
      while (active) {
        try {
          const events = await workspaceApi.changes();
          if (!active) return;
          if (events.length) {
            const nextTree = await workspaceApi.refresh();
            if (!active) return;
            setTree(nextTree);
            const paths = events.flatMap((event) => event.paths);
            setTabs((openTabs) => {
              const conflicted = openTabs.filter((tab) => isDirty(tab) && paths.some((path) => path.endsWith(tab.path)));
              if (conflicted.length) setMessage(`External change conflicts with unsaved ${conflicted[0].path}`);
              return openTabs;
            });
          }
        } catch (error) { if (active) setMessage(`Workspace watcher: ${String(error)}`); }
        await new Promise((resolve) => window.setTimeout(resolve, 250));
      }
    };
    void watch();
    return () => { active = false; };
  }, [currentProject]);
  useEffect(() => { const cleanups = Promise.all([listen<ProcessOutput>("process-output", ({ payload }) => setTerminalOutput((lines) => appendTerminalHistory(lines, `${payload.stream === "stderr" ? "! " : ""}${payload.line}`))), listen<ProcessExit>("process-exit", ({ payload }) => { setTerminalOutput((lines) => appendTerminalHistory(lines, `[exit ${payload.exitCode ?? "signal"}]`)); setActiveProcess((current) => current === payload.id ? undefined : current); void workspaceApi.refresh().then(setTree); void refreshGit(); })]); return () => { void cleanups.then((items) => items.forEach((unlisten) => unlisten())); }; }, []);
  useEffect(() => { if (!activeProcess) return; let active = true; const poll = async () => { while (active) { try { const update = await workspaceApi.readTerminal(activeProcess); if (update.output) setTerminalOutput((lines) => appendTerminalHistory(lines, update.output)); if (!update.running) { setActiveProcess(undefined); setTerminalOutput((lines) => appendTerminalHistory(lines, "[process exited]")); void refreshGit(); break; } } catch { break; } await new Promise((resolve) => window.setTimeout(resolve, 120)); } }; void poll(); return () => { active = false; }; }, [activeProcess]);
  async function chooseProject() { const selected = await open({ directory: true, multiple: false, title: "Open a project in Helel" }); if (typeof selected === "string") await loadProject(selected); }
  async function chooseModelArtifact(key: "modelConfig" | "modelTokenizer" | "modelWeights" | "voiceModel") { const selected = await open({ directory: false, multiple: false, title: key === "voiceModel" ? "Select local whisper.cpp GGML model" : "Select local model artifact" }); if (typeof selected === "string") setSettings((current) => ({ ...current, [key]: selected })); }
  async function useLocalSmokeModel() { try { const defaults = await workspaceApi.localModelDefaults(); setSettings((current) => ({ ...current, modelProgram: defaults.program, modelArgs: "", modelConfig: defaults.config, modelTokenizer: defaults.tokenizer, modelWeights: defaults.weights })); setMessage("Selected the local smoke model"); } catch (error) { setMessage(String(error)); } }
  async function useLocalVoice() { try { const defaults = await workspaceApi.localVoiceDefaults(); setSettings((current) => ({ ...current, voiceProgram: defaults.program, voiceModel: defaults.model })); setMessage("Selected the installed offline voice runtime"); } catch (error) { setMessage(String(error)); } }
  async function toggleModel() { try { if (modelRunning) { await workspaceApi.stopModel(); setModelRunning(false); setMessage("Local model stopped"); } else { if (!settings.modelConfig || !settings.modelTokenizer || !settings.modelWeights) { setMessage("Select config, tokenizer, and weights first"); return; } await workspaceApi.startModel(settings.modelProgram, settings.modelArgs.trim() ? settings.modelArgs.trim().split(/\s+/) : [], settings.modelConfig, settings.modelTokenizer, settings.modelWeights, true); setModelRunning(true); setMessage("Local model is healthy"); } } catch (error) { setModelRunning(false); setMessage(String(error)); } }
  async function toggleMcp() { try { if (mcpRunning) { await workspaceApi.stopMcp(settings.mcpName); setMcpRunning(false); setMcpSummary(""); setMessage("MCP server stopped"); return; } if (!settings.mcpName.trim() || !settings.mcpProgram.trim()) { setMessage("Enter an MCP name and executable"); return; } const server = { name: settings.mcpName.trim(), program: settings.mcpProgram.trim(), args: settings.mcpArgs.trim() ? settings.mcpArgs.trim().split(/\s+/) : [], enabled: true }; await workspaceApi.saveMcp([server], true); await workspaceApi.startMcp(server.name, true); const tools = await workspaceApi.callMcp(server.name, "tools/list", {}, true); setMcpRunning(true); setMcpSummary(`${Array.isArray(tools.tools) ? tools.tools.length : 0} tools discovered`); setMessage(`MCP ${server.name} initialized`); } catch (error) { setMcpRunning(false); setMessage(String(error)); } }
  async function openFile(path: string, line?: number) { try { const existing = tabs.find((tab) => tab.path === path); if (!existing) { const content = await workspaceApi.read(path); setTabs((current) => upsertTab(current, { path, content, savedContent: content })); } setActivePath(path); setMessage(line ? `${path}:${line}` : path); } catch (error) { setMessage(String(error)); } }
  async function saveOne(path: string) { const tab = tabs.find((item) => item.path === path); if (!tab) return; try { await workspaceApi.save(path, tab.content); setTabs((current) => markSaved(current, path, tab.content)); setMessage(`Saved ${path}`); } catch (error) { setMessage(String(error)); } }
  async function saveAll() { try { for (const tab of tabs.filter(isDirty)) { await workspaceApi.save(tab.path, tab.content); setTabs((current) => markSaved(current, tab.path, tab.content)); } setMessage("Saved all files"); } catch (error) { setMessage(`Save failed: ${String(error)}`); } }
  function closeTab(path: string) { const tab = tabs.find((item) => item.path === path); if (tab && isDirty(tab) && !window.confirm(`Discard unsaved changes to ${path}?`)) return; const index = tabs.findIndex((item) => item.path === path); const next = tabs.filter((item) => item.path !== path); setTabs(next); if (activePath === path) setActivePath(next[Math.max(0, index - 1)]?.path); setProblems((items) => items.filter((problem) => problem.path !== path)); }
  async function mutate(kind: "file" | "folder" | "rename" | "delete", selected?: string) { try { let next: TreeEntry[] | undefined; if (kind === "file" || kind === "folder") { const path = window.prompt(`New ${kind} path`); if (path) next = await workspaceApi.create(path, kind === "folder"); } else if (kind === "rename" && selected) { const to = window.prompt("Rename to", selected); if (to && to !== selected) { next = await workspaceApi.rename(selected, to); setTabs((items) => items.filter((tab) => tab.path !== selected)); } } else if (kind === "delete" && selected && window.confirm(`Delete ${selected}? Non-empty folders are protected.`)) { next = await workspaceApi.delete(selected); closeTab(selected); } if (next) { setTree(next); setMessage(`${kind} completed`); } } catch (error) { setMessage(String(error)); } }
  async function search(query: string) { try { setSearchResults(await workspaceApi.search(query)); } catch (error) { setMessage(String(error)); } }
  async function replace(query: string, replacement: string) { if (tabs.some(isDirty)) { setMessage("Save or discard open changes before replacing across files"); return; } try { const count = await workspaceApi.replace(query, replacement); const refreshed = await Promise.all(tabs.map(async (tab) => { const content = await workspaceApi.read(tab.path); return { ...tab, content, savedContent: content }; })); setTabs(refreshed); setSearchResults(await workspaceApi.search(query)); setMessage(`Replaced ${count} matches`); } catch (error) { setMessage(String(error)); } }
  async function refreshGit() { try { setGit(await workspaceApi.git()); } catch (error) { setGit(undefined); setMessage(String(error)); } }
  async function stageAll() { if (!git?.changes.length) return; const paths = git.changes.map((change) => change.slice(3).split(" -> ").at(-1) ?? "").filter(Boolean); if (!window.confirm(`Stage ${paths.length} changed paths?`)) return; try { setGit(await workspaceApi.gitStage(paths, true)); setMessage(`Staged ${paths.length} paths`); } catch (error) { setMessage(String(error)); } }
  async function commitChanges() { const message = window.prompt("Commit message"); if (!message?.trim()) return; try { const output = await workspaceApi.gitCommit(message, true); setMessage(output.trim().split("\n")[0] ?? "Commit created"); await refreshGit(); } catch (error) { setMessage(String(error)); } }
  async function runCommand(command: string, args: string[]) { try { if (activeProcess) { await workspaceApi.writeTerminal(activeProcess, `${[command, ...args].join(" ")}\n`); return; } const risk = await workspaceApi.classify(command, args); const approved = risk === "safe" || window.confirm(`This is a ${risk} command:\n\n${command} ${args.join(" ")}\n\nRun it inside the workspace PTY?`); if (!approved) { setTerminalOutput((lines) => appendTerminalHistory(lines, `$ ${command} ${args.join(" ")}`, "Cancelled.")); return; } setTerminalOutput((lines) => appendTerminalHistory(lines, `$ ${command} ${args.join(" ")}`)); setActiveProcess(await workspaceApi.startTerminal(command, args, approved)); } catch (error) { setTerminalOutput((lines) => appendTerminalHistory(lines, String(error))); } }
  async function cancelCommand() { if (!activeProcess) return; try { await workspaceApi.stopTerminal(activeProcess); setActiveProcess(undefined); setTerminalOutput((lines) => appendTerminalHistory(lines, "Cancellation requested.")); } catch (error) { setTerminalOutput((lines) => appendTerminalHistory(lines, String(error))); } }
  async function applyPatch(patch: string, reverse: boolean) { if (!patch.trim()) return; if (tabs.some(isDirty)) { setMessage("Save or discard unsaved editor changes before applying a patch"); return; } if (!window.confirm(`${reverse ? "Reverse" : "Apply"} this patch inside the current workspace?`)) return; try { setTree(await workspaceApi.applyPatch(patch, reverse, true)); setMessage(reverse ? "Patch reversed" : "Patch applied"); void refreshGit(); } catch (error) { setMessage(String(error)); } }
  async function rollbackPatch() { if (!window.confirm("Restore every file changed by the most recent task checkpoint?")) return; try { setTree(await workspaceApi.rollbackPatch()); setMessage("Restored the last task checkpoint"); void refreshGit(); } catch (error) { setMessage(String(error)); } }
  async function buildIndex() { try { const built = await workspaceApi.buildIndex(); setIndex(built); setMessage(`Indexed ${built.files.length} files and ${built.symbols.length} symbols`); } catch (error) { setMessage(String(error)); } }
  async function startAgent(objective: string) { try { const session = await workspaceApi.startAgent(objective); setAgents((items) => [session, ...items]); setMessage(`Started local session #${session.id}`); } catch (error) { setMessage(String(error)); } }
  async function advanceAgent(session: AgentSession) { const tool = session.pendingTool; if (tool?.kind === "applyPatch" && tabs.some(isDirty)) { setMessage("Save or discard unsaved editor changes before the agent applies a patch"); return; } const needsApproval = requiresApproval(session); const approved = !needsApproval || window.confirm(`Session #${session.id} requests approval.\n\n${describeTool(tool)}\n\nApprove this one action?`); if (!approved) return; try { const updated = tool ? await workspaceApi.advanceAgent(session.id, approved) : await workspaceApi.runAgentModelStep(session.id); setAgents((items) => items.map((item) => item.id === updated.id ? updated : item)); setMessage(`Session #${updated.id}: ${updated.phase}`); } catch (error) { setMessage(String(error)); } }
  async function cancelAgent(id: number) { try { const updated = await workspaceApi.cancelAgent(id); setAgents((items) => items.map((item) => item.id === id ? updated : item)); } catch (error) { setMessage(String(error)); } }
  async function retryAgent(id: number) { try { const updated = await workspaceApi.retryAgent(id); setAgents((items) => items.map((item) => item.id === id ? updated : item)); setMessage(`Restarted session #${id} with fresh budgets`); } catch (error) { setMessage(String(error)); } }
  async function exportAudit() { try { const path = await workspaceApi.exportAudit(); setMessage(`Exported verified audit log to ${path}`); } catch (error) { setMessage(String(error)); } }
  async function pauseAgent(session: AgentSession) { try { const updated = await workspaceApi.pauseAgent(session.id, session.phase !== "paused"); setAgents((items) => items.map((item) => item.id === session.id ? updated : item)); } catch (error) { setMessage(String(error)); } }
  async function speak(text: string) { try { await workspaceApi.speak(text, true); setMessage("Speaking locally"); } catch (error) { setMessage(String(error)); } }

  const commands: CommandItem[] = [
    { id: "open", label: "Open project", detail: "Choose a local workspace", shortcut: "⌘O", icon: "folder", run: () => void chooseProject() },
    { id: "explorer", label: "Show Explorer", detail: "Browse project files", icon: "files", run: () => setView("explorer") },
    { id: "search", label: "Search project", detail: "Find and replace across files", shortcut: "⌘⇧F", icon: "search", run: () => setView("search") },
    { id: "source", label: "Source control", detail: "Review Git changes and patches", icon: "branch", run: () => { setView("source"); void refreshGit(); } },
    { id: "agent", label: "Local agent", detail: "Create and inspect autonomous tasks", icon: "agent", run: () => setView("agent") },
    { id: "save-all", label: "Save all files", detail: `${tabs.filter(isDirty).length} unsaved file${tabs.filter(isDirty).length === 1 ? "" : "s"}`, shortcut: "⌘⇧S", icon: "files", run: () => void saveAll() },
    { id: "settings", label: "Open settings", detail: "Theme, model, and MCP runtime", shortcut: "⌘,", icon: "settings", run: () => setSettingsOpen(true) },
  ];

  return <main className="app-shell" aria-label="Helel workspace" onKeyDown={(event) => { if (event.key === "Escape" && settingsOpen) { setSettingsOpen(false); } else if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") { event.preventDefault(); setPaletteOpen(true); } else if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "o") { event.preventDefault(); void chooseProject(); } else if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key.toLowerCase() === "f") { event.preventDefault(); setView("search"); } else if ((event.metaKey || event.ctrlKey) && event.key === ",") { event.preventDefault(); setSettingsOpen(true); } else if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "s") { event.preventDefault(); if (event.shiftKey) void saveAll(); else if (activePath) void saveOne(activePath); } }}>
    <header className="titlebar"><div className="traffic-space" aria-hidden="true" /><span className="wordmark"><span className="brand-mark"><span>H</span></span><span>Helel<small>LOCAL STUDIO</small></span></span><button className="project-switcher" onClick={() => setPaletteOpen(true)}><span className="project-status" />{currentProject ? projectName(currentProject) : "No project"}<kbd>⌘ K</kbd></button><span className="runtime-pill"><span className={modelRunning ? "live" : ""} /> {modelRunning ? "Model ready" : "Offline"}</span><button className="icon-button title-action" onClick={() => setSettingsOpen(true)} aria-label="Open settings"><Icon name="settings" /></button></header>
    <div className="workspace"><ActivityBar active={view} onChange={(next) => { setView(next); if (next === "source") void refreshGit(); }} /><SidePanel view={view} width={settings.sidebarWidth} tree={tree} recentProjects={settings.recentProjects} currentProject={currentProject} searchResults={searchResults} git={git} index={index} agents={agents} voiceProgram={settings.voiceProgram} voiceModel={settings.voiceModel} onSearch={search} onReplace={replace} onRefreshGit={refreshGit} onStageAll={stageAll} onCommit={commitChanges} onApplyPatch={applyPatch} onRollback={rollbackPatch} onBuildIndex={buildIndex} onStartAgent={startAgent} onAdvanceAgent={advanceAgent} onPauseAgent={pauseAgent} onCancelAgent={cancelAgent} onRetryAgent={retryAgent} onExportAudit={exportAudit} onOpenProject={chooseProject} onSelectProject={loadProject} onOpenFile={openFile} onMutate={mutate} /><Workbench project={currentProject} agents={agents} tabs={tabs} activePath={activePath} problems={problems} terminalOutput={terminalOutput} activeProcess={activeProcess} sidebarWidth={settings.sidebarWidth} bottomHeight={settings.bottomPanelHeight} agentWidth={settings.agentWidth} onResize={(update) => setSettings((current) => ({ ...current, ...update }))} onOpenProject={chooseProject} onActivate={setActivePath} onClose={closeTab} onChange={(path, content) => setTabs((current) => updateTab(current, path, content))} onSave={saveOne} onSaveAll={saveAll} onProblems={(path, next) => setProblems((items) => [...items.filter((problem) => problem.path !== path), ...next])} onOpenProblem={openFile} onRunCommand={runCommand} onCancelCommand={cancelCommand} onSpeak={speak} /></div>
    {activeProcess && <button className="process-stop" type="button" onClick={cancelCommand}>Stop process #{activeProcess}</button>}
    <footer className="statusbar" aria-label="Status bar"><span><Icon name="branch" /> {git?.branch ?? "local"}</span><span className="status-grow">{message}</span><span>{activeProcess ? `process #${activeProcess} running` : `${problems.length} problems`}</span><span>v0.1 · local</span></footer>
    {paletteOpen && <CommandPalette commands={commands} onClose={() => setPaletteOpen(false)} />}
    {settingsOpen && <div className="dialog-backdrop" role="presentation" onMouseDown={() => setSettingsOpen(false)}><section className="settings-dialog" role="dialog" aria-modal="true" aria-labelledby="settings-title" onMouseDown={(event) => event.stopPropagation()}><div className="dialog-heading"><div><p className="overline">Preferences</p><h2 id="settings-title">Appearance</h2></div><button className="icon-button" onClick={() => setSettingsOpen(false)} aria-label="Close settings"><Icon name="close" /></button></div><fieldset><legend>Theme</legend><div className="segmented">{(["dark", "light", "system"] as const).map((theme) => <button key={theme} className={settings.theme === theme ? "selected" : ""} onClick={() => setSettings((current) => ({ ...current, theme }))}>{theme}</button>)}</div></fieldset><fieldset><legend>Local model runtime</legend><label>Executable<input value={settings.modelProgram} onChange={(event) => setSettings((current) => ({ ...current, modelProgram: event.target.value }))} /></label><label>Arguments<input value={settings.modelArgs} onChange={(event) => setSettings((current) => ({ ...current, modelArgs: event.target.value }))} placeholder="optional" /></label><button className="secondary-button" onClick={useLocalSmokeModel}>Use generated smoke model</button><button className="secondary-button" onClick={() => chooseModelArtifact("modelConfig")}>Config: {settings.modelConfig ? projectName(settings.modelConfig) : "select"}</button><button className="secondary-button" onClick={() => chooseModelArtifact("modelTokenizer")}>Tokenizer: {settings.modelTokenizer ? projectName(settings.modelTokenizer) : "select"}</button><button className="secondary-button" onClick={() => chooseModelArtifact("modelWeights")}>Weights: {settings.modelWeights ? projectName(settings.modelWeights) : "select"}</button><button className="primary-button" onClick={toggleModel}>{modelRunning ? "Stop local model" : "Start local model"}</button></fieldset><fieldset><legend>Offline voice</legend><label>whisper.cpp executable<input value={settings.voiceProgram} onChange={(event) => setSettings((current) => ({ ...current, voiceProgram: event.target.value }))} /></label><button className="secondary-button" onClick={useLocalVoice}>Use installed offline voice</button><button className="secondary-button" onClick={() => chooseModelArtifact("voiceModel")}>Speech model: {settings.voiceModel ? projectName(settings.voiceModel) : "select local GGML model"}</button><small>Recordings are capped at 30 seconds and deleted after local transcription.</small></fieldset><fieldset><legend>Local MCP server</legend><label>Name<input value={settings.mcpName} onChange={(event) => setSettings((current) => ({ ...current, mcpName: event.target.value }))} /></label><label>Executable<input value={settings.mcpProgram} onChange={(event) => setSettings((current) => ({ ...current, mcpProgram: event.target.value }))} /></label><label>Arguments<input value={settings.mcpArgs} onChange={(event) => setSettings((current) => ({ ...current, mcpArgs: event.target.value }))} /></label><button className="primary-button" onClick={toggleMcp}>{mcpRunning ? "Stop MCP server" : "Save, start, and discover"}</button>{mcpSummary && <small>{mcpSummary}</small>}</fieldset><button className="secondary-button" onClick={() => setSettings({ ...DEFAULT_SETTINGS, recentProjects: settings.recentProjects })}>Reset panel layout</button></section></div>}
  </main>;
}

export function Icon({ name }: { name: "files" | "search" | "branch" | "agent" | "settings" | "folder" | "close" | "terminal" | "mic" | "speaker" }) { const paths = { files: <><path d="M5 3h9l5 5v13H5z"/><path d="M14 3v5h5"/><path d="M9 13h6M9 17h6"/></>, search: <><circle cx="11" cy="11" r="7"/><path d="m16 16 5 5"/></>, branch: <><circle cx="6" cy="5" r="2"/><circle cx="6" cy="19" r="2"/><circle cx="18" cy="8" r="2"/><path d="M6 7v10M8 7c3 0 3 1 3 1h5M11 8v4c0 3-2 3-3 3H6"/></>, agent: <><rect x="4" y="6" width="16" height="13" rx="3"/><path d="M12 3v3M8 12h.01M16 12h.01M8 16h8"/></>, settings: <><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1-2.8 2.8-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.6v.2h-4V21a1.7 1.7 0 0 0-1-1.6 1.7 1.7 0 0 0-1.9.3l-.1.1L4.2 17l.1-.1a1.7 1.7 0 0 0 .3-1.9A1.7 1.7 0 0 0 3 14H2.8v-4H3a1.7 1.7 0 0 0 1.6-1 1.7 1.7 0 0 0-.3-1.9L4.2 7 7 4.2l.1.1a1.7 1.7 0 0 0 1.9.3A1.7 1.7 0 0 0 10 3v-.2h4V3a1.7 1.7 0 0 0 1 1.6 1.7 1.7 0 0 0 1.9-.3l.1-.1L19.8 7l-.1.1a1.7 1.7 0 0 0-.3 1.9 1.7 1.7 0 0 0 1.6 1h.2v4H21a1.7 1.7 0 0 0-1.6 1z"/></>, folder: <path d="M3 6h7l2 2h9v11H3z"/>, close: <path d="m6 6 12 12M18 6 6 18"/>, terminal: <path d="m5 7 4 4-4 4M11 17h8"/>, mic: <><rect x="9" y="3" width="6" height="12" rx="3"/><path d="M5.5 11a6.5 6.5 0 0 0 13 0M12 17.5V21M9 21h6"/></>, speaker: <><path d="M5 10h4l5-4v12l-5-4H5zM17 9a4 4 0 0 1 0 6M19 6a8 8 0 0 1 0 12"/></> }; return <svg viewBox="0 0 24 24" aria-hidden="true">{paths[name]}</svg>; }
