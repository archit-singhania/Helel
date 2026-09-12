export interface ProjectProfile { languages: string[]; frameworks: string[]; manifests: string[]; }
export interface CodeIndex { version: number; generatedAtMs: number; profile: ProjectProfile; files: { path: string; language: string; bytes: number; modifiedMs: number }[]; symbols: SymbolMatch[]; references: ReferenceMatch[]; }
export interface SymbolMatch { name: string; kind: string; path: string; line: number; column: number; signature: string; }
export interface ReferenceMatch { name: string; path: string; line: number; column: number; }
export interface ContextHit { path: string; line: number; score: number; preview: string; }

export type AgentPhase = "planning" | "gathering" | "executing" | "verifying" | "awaitingApproval" | "paused" | "completed" | "failed" | "cancelled";
export type ToolRequest = { kind: "searchCode"; query: string; limit: number } | { kind: "readFile"; path: string } | { kind: "inspectGit" } | { kind: "runCommand"; command: string; args: string[] } | { kind: "applyPatch"; patch: string; reverse: boolean };
export interface AgentSession { id: number; objective: string; phase: AgentPhase; plan: string[]; pendingTool?: ToolRequest; observations: { step: number; summary: string; success: boolean }[]; step: number; }

export function requiresApproval(session: AgentSession) { return session.phase === "awaitingApproval" || session.pendingTool?.kind === "runCommand" || session.pendingTool?.kind === "applyPatch"; }
