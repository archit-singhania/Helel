export interface TreeEntry { name: string; path: string; isDirectory: boolean; children: TreeEntry[]; }
export interface SearchMatch { path: string; line: number; preview: string; }
export interface EditorTab { path: string; content: string; savedContent: string; }
export interface Problem { path: string; line: number; column: number; severity: "error" | "warning"; message: string; }

export function languageForPath(path: string) {
  const extension = path.split(".").at(-1)?.toLowerCase();
  return ({ ts: "typescript", tsx: "typescript", js: "javascript", jsx: "javascript", json: "json", jsonl: "json", css: "css", scss: "scss", less: "less", html: "html", md: "markdown", py: "python", rs: "rust", java: "java", kt: "kotlin", kts: "kotlin", go: "go", c: "c", h: "c", cc: "cpp", cpp: "cpp", hpp: "cpp", cs: "csharp", swift: "swift", dart: "dart", php: "php", rb: "ruby", ex: "elixir", exs: "elixir", lua: "lua", r: "r", pl: "perl", scala: "scala", fs: "fsharp", fsx: "fsharp", hs: "haskell", m: "objective-c", mm: "objective-cpp", proto: "protobuf", vue: "vue", svelte: "svelte", zig: "zig", sql: "sql", yaml: "yaml", yml: "yaml", toml: "toml", xml: "xml", sh: "shell", bash: "shell", zsh: "shell", fish: "shell" } as Record<string, string>)[extension ?? ""] ?? "plaintext";
}

export function upsertTab(tabs: EditorTab[], tab: EditorTab) { return tabs.some((item) => item.path === tab.path) ? tabs : [...tabs, tab]; }
export function updateTab(tabs: EditorTab[], path: string, content: string) { return tabs.map((tab) => tab.path === path ? { ...tab, content } : tab); }
export function markSaved(tabs: EditorTab[], path: string, savedContent: string) { return tabs.map((tab) => tab.path === path ? { ...tab, savedContent } : tab); }
export function isDirty(tab: EditorTab) { return tab.content !== tab.savedContent; }
