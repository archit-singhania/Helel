import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { App } from "./App";
import { DEFAULT_SETTINGS, parseSettings, projectName, withRecentProject } from "./settings";

describe("desktop shell", () => {
  it("renders primary accessible landmarks", () => { const html = renderToStaticMarkup(<App />); expect(html).toContain("aria-label=\"Primary navigation\""); expect(html).toContain("aria-label=\"Editor\""); expect(html).toContain("aria-label=\"Helel agent\""); expect(html).toContain("aria-label=\"Terminal and problems\""); });
  it("extracts cross-platform project names", () => { expect(projectName("/work/helel")).toBe("helel"); expect(projectName("C:\\work\\helel")).toBe("helel"); });
});
describe("settings", () => {
  it("recovers from invalid data and clamps dimensions", () => { expect(parseSettings("broken")).toEqual(DEFAULT_SETTINGS); expect(parseSettings(JSON.stringify({ sidebarWidth: 999, agentWidth: 1, bottomPanelHeight: 200 }))).toMatchObject({ sidebarWidth: 420, agentWidth: 260, bottomPanelHeight: 200 }); });
  it("deduplicates and limits recent projects", () => { let settings = DEFAULT_SETTINGS; for (let index = 0; index < 10; index += 1) settings = withRecentProject(settings, `/project/${index}`); settings = withRecentProject(settings, "/project/5"); expect(settings.recentProjects).toHaveLength(8); expect(settings.recentProjects[0]).toBe("/project/5"); expect(new Set(settings.recentProjects).size).toBe(8); });
});
