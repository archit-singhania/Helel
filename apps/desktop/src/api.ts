import { invoke } from "@tauri-apps/api/core";
import type { SearchMatch, TreeEntry } from "./ide";
import type { AuditEntry, GitSummary, ProcessStarted, Risk } from "./system";
import type { AgentSession, CodeIndex, ContextHit, ReferenceMatch, SymbolMatch } from "./intelligence";

export const workspaceApi = {
  open: (root: string) => invoke<TreeEntry[]>("open_workspace", { root }),
  refresh: () => invoke<TreeEntry[]>("refresh_tree"),
  read: (path: string) => invoke<string>("read_file", { path }),
  save: (path: string, content: string) => invoke<void>("save_file", { path, content }),
  create: (path: string, directory: boolean) => invoke<TreeEntry[]>("create_entry", { path, directory }),
  rename: (from: string, to: string) => invoke<TreeEntry[]>("rename_entry", { from, to }),
  delete: (path: string) => invoke<TreeEntry[]>("delete_entry", { path }),
  search: (query: string) => invoke<SearchMatch[]>("search_workspace", { query }),
  replace: (query: string, replacement: string) => invoke<number>("replace_workspace", { query, replacement }),
  classify: (command: string, args: string[]) => invoke<Risk>("classify_command", { command, args }),
  run: (command: string, args: string[], approved: boolean) => invoke<ProcessStarted>("run_command", { command, args, approved }),
  cancel: (id: number) => invoke<void>("cancel_command", { id }),
  git: () => invoke<GitSummary>("git_status"),
  applyPatch: (patch: string, reverse: boolean, approved: boolean) => invoke<TreeEntry[]>("apply_workspace_patch", { patch, reverse, approved }),
  audit: () => invoke<AuditEntry[]>("audit_log"),
  buildIndex: () => invoke<CodeIndex>("build_code_index"),
  context: (query: string, limit = 20) => invoke<ContextHit[]>("code_context", { query, limit }),
  definitions: (name: string, limit = 100) => invoke<SymbolMatch[]>("symbol_definitions", { name, limit }),
  references: (name: string, limit = 500) => invoke<ReferenceMatch[]>("symbol_references", { name, limit }),
  startAgent: (objective: string) => invoke<AgentSession>("start_agent_session", { objective }),
  advanceAgent: (id: number, approved: boolean) => invoke<AgentSession>("advance_agent_session", { id, approved }),
  listAgents: () => invoke<AgentSession[]>("list_agent_sessions"),
  cancelAgent: (id: number) => invoke<AgentSession>("cancel_agent_session", { id }),
};
