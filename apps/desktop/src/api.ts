import { invoke } from "@tauri-apps/api/core";
import type { SearchMatch, TreeEntry } from "./ide";
import type { AuditEntry, GitSummary, ProcessStarted, Risk } from "./system";
import type { AgentSession, CodeIndex, ContextHit, ReferenceMatch, SymbolMatch } from "./intelligence";

export const workspaceApi = {
  open: (root: string) => invoke<TreeEntry[]>("open_workspace", { root }),
  refresh: () => invoke<TreeEntry[]>("refresh_tree"),
  changes: () => invoke<WorkspaceEvent[]>("workspace_changes"),
  projectProfile: () => invoke<ProjectProfile>("project_profile"),
  localModelDefaults: () => invoke<{ program: string; config: string; tokenizer: string; weights: string }>("local_model_defaults"),
  startModel: (program: string, args: string[], config: string, tokenizer: string, weights: string, approved: boolean) => invoke<void>("start_model_runtime", { program, args, config, tokenizer, weights, approved }),
  generateLocal: (request: string) => invoke<Record<string, unknown>[]>("generate_local", { request }),
  stopModel: () => invoke<void>("stop_model_runtime"),
  read: (path: string) => invoke<string>("read_file", { path }),
  save: (path: string, content: string) => invoke<void>("save_file", { path, content }),
  create: (path: string, directory: boolean) => invoke<TreeEntry[]>("create_entry", { path, directory }),
  rename: (from: string, to: string) => invoke<TreeEntry[]>("rename_entry", { from, to }),
  delete: (path: string) => invoke<TreeEntry[]>("delete_entry", { path }),
  search: (query: string) => invoke<SearchMatch[]>("search_workspace", { query }),
  replace: (query: string, replacement: string) => invoke<number>("replace_workspace", { query, replacement }),
  classify: (command: string, args: string[]) => invoke<Risk>("classify_command", { command, args }),
  startTerminal: (command: string, args: string[], approved: boolean, cols = 100, rows = 30) => invoke<number>("start_terminal", { command, args, approved, cols, rows }),
  writeTerminal: (id: number, input: string) => invoke<void>("write_terminal", { id, input }),
  readTerminal: (id: number) => invoke<{ output: string; running: boolean }>("read_terminal", { id }),
  resizeTerminal: (id: number, cols: number, rows: number) => invoke<void>("resize_terminal", { id, cols, rows }),
  stopTerminal: (id: number) => invoke<void>("stop_terminal", { id }),
  listMcp: () => invoke<McpServer[]>("list_mcp_servers"),
  saveMcp: (servers: McpServer[], approved: boolean) => invoke<void>("save_mcp_servers", { servers, approved }),
  startMcp: (name: string, approved: boolean) => invoke<Record<string, unknown>>("start_mcp_server", { name, approved }),
  callMcp: (name: string, method: string, params: Record<string, unknown>, approved: boolean) => invoke<Record<string, unknown>>("call_mcp_tool", { name, method, params, approved }),
  stopMcp: (name: string) => invoke<void>("stop_mcp_server", { name }),
  run: (command: string, args: string[], approved: boolean) => invoke<ProcessStarted>("run_command", { command, args, approved }),
  cancel: (id: number) => invoke<void>("cancel_command", { id }),
  git: () => invoke<GitSummary>("git_status"),
  gitStage: (paths: string[], approved: boolean) => invoke<GitSummary>("git_stage", { paths, approved }),
  gitCommit: (message: string, approved: boolean) => invoke<string>("git_commit", { message, approved }),
  applyPatch: (patch: string, reverse: boolean, approved: boolean) => invoke<TreeEntry[]>("apply_workspace_patch", { patch, reverse, approved }),
  rollbackPatch: () => invoke<TreeEntry[]>("rollback_last_patch"),
  audit: () => invoke<AuditEntry[]>("audit_log"),
  buildIndex: () => invoke<CodeIndex>("build_code_index"),
  context: (query: string, limit = 20) => invoke<ContextHit[]>("code_context", { query, limit }),
  definitions: (name: string, limit = 100) => invoke<SymbolMatch[]>("symbol_definitions", { name, limit }),
  references: (name: string, limit = 500) => invoke<ReferenceMatch[]>("symbol_references", { name, limit }),
  startAgent: (objective: string) => invoke<AgentSession>("start_agent_session", { objective }),
  advanceAgent: (id: number, approved: boolean) => invoke<AgentSession>("advance_agent_session", { id, approved }),
  submitAgentProposal: (id: number, proposal: string, nonce: string) => invoke<AgentSession>("submit_agent_proposal", { id, proposal, nonce }),
  runAgentModelStep: (id: number) => invoke<AgentSession>("run_agent_model_step", { id }),
  pauseAgent: (id: number, paused: boolean) => invoke<AgentSession>("set_agent_paused", { id, paused }),
  completeAgent: (id: number, summary: string) => invoke<AgentSession>("complete_agent_session", { id, summary }),
  listAgents: () => invoke<AgentSession[]>("list_agent_sessions"),
  cancelAgent: (id: number) => invoke<AgentSession>("cancel_agent_session", { id }),
};

export type ProjectProfile = {
  kind: "rust" | "react" | "angular" | "node" | "python" | "maven" | "gradle" | "flutter" | "unknown";
  commands: Array<{ label: string; program: string; args: string[]; workingDirectory: string; timeoutSeconds: number; requiresApproval: boolean }>;
};
export type WorkspaceEvent = { kind: string; paths: string[] };
export type McpServer = { name: string; program: string; args: string[]; enabled: boolean };
