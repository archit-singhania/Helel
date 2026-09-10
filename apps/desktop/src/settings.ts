export type Theme = "dark" | "light" | "system";
export interface Settings { theme: Theme; sidebarWidth: number; agentWidth: number; bottomPanelHeight: number; recentProjects: string[]; }
export const SETTINGS_KEY = "helel.settings.v1";
export const DEFAULT_SETTINGS: Settings = { theme: "dark", sidebarWidth: 248, agentWidth: 320, bottomPanelHeight: 190, recentProjects: [] };
const bounded = (value: unknown, fallback: number, min: number, max: number) => typeof value === "number" && Number.isFinite(value) ? Math.min(max, Math.max(min, value)) : fallback;
export function parseSettings(value: string | null): Settings {
  if (!value) return DEFAULT_SETTINGS;
  try {
    const candidate = JSON.parse(value) as Partial<Settings>;
    const theme = candidate.theme;
    return { theme: theme === "dark" || theme === "light" || theme === "system" ? theme : DEFAULT_SETTINGS.theme, sidebarWidth: bounded(candidate.sidebarWidth, DEFAULT_SETTINGS.sidebarWidth, 200, 420), agentWidth: bounded(candidate.agentWidth, DEFAULT_SETTINGS.agentWidth, 260, 520), bottomPanelHeight: bounded(candidate.bottomPanelHeight, DEFAULT_SETTINGS.bottomPanelHeight, 120, 420), recentProjects: Array.isArray(candidate.recentProjects) ? candidate.recentProjects.filter((path): path is string => typeof path === "string" && path.length > 0).slice(0, 8) : [] };
  } catch { return DEFAULT_SETTINGS; }
}
export function loadSettings(storage?: Pick<Storage, "getItem">) {
  const source = storage ?? (typeof window === "undefined" ? undefined : window.localStorage);
  return parseSettings(source?.getItem(SETTINGS_KEY) ?? null);
}
export function saveSettings(settings: Settings, storage: Pick<Storage, "setItem"> = localStorage) { storage.setItem(SETTINGS_KEY, JSON.stringify(settings)); }
export function withRecentProject(settings: Settings, path: string): Settings { return { ...settings, recentProjects: [path, ...settings.recentProjects.filter((item) => item !== path)].slice(0, 8) }; }
export function projectName(path: string) { return path.split(/[\\/]/).filter(Boolean).at(-1) ?? path; }
