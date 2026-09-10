export interface TreeEntry { name: string; path: string; isDirectory: boolean; children: TreeEntry[]; }
export interface SearchMatch { path: string; line: number; preview: string; }
export interface EditorTab { path: string; content: string; savedContent: string; }
export interface Problem { path: string; line: number; column: number; severity: "error" | "warning"; message: string; }

export function languageForPath(path: string) {
  const extension = path.split(".").at(-1)?.toLowerCase();
  return ({ ts: "typescript", tsx: "typescript", js: "javascript", jsx: "javascript", json: "json", css: "css", html: "html", md: "markdown", py: "python", rs: "rust", java: "java", sql: "sql", yaml: "yaml", yml: "yaml", xml: "xml", sh: "shell" } as Record<string, string>)[extension ?? ""] ?? "plaintext";
}

export function upsertTab(tabs: EditorTab[], tab: EditorTab) { return tabs.some((item) => item.path === tab.path) ? tabs : [...tabs, tab]; }
export function updateTab(tabs: EditorTab[], path: string, content: string) { return tabs.map((tab) => tab.path === path ? { ...tab, content } : tab); }
export function markSaved(tabs: EditorTab[], path: string) { return tabs.map((tab) => tab.path === path ? { ...tab, savedContent: tab.content } : tab); }
export function isDirty(tab: EditorTab) { return tab.content !== tab.savedContent; }
