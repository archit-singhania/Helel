export type Risk = "safe" | "modify" | "dangerous";
export interface ProcessStarted { id: number; risk: Risk; }
export interface ProcessOutput { id: number; stream: "stdout" | "stderr"; line: string; }
export interface ProcessExit { id: number; exitCode: number | null; }
export interface GitSummary { branch: string; changes: string[]; diff: string; stagedDiff: string; }
export interface AuditEntry { timestampMs: number; action: string; risk: Risk; approved: boolean; success: boolean; }

const TERMINAL_HISTORY_LIMIT = 2_000;

export function appendTerminalHistory(current: string[], ...entries: string[]) {
  const result: string[] = [];
  let remaining = 262_144;
  const all = [...current, ...entries];
  for (let index = all.length - 1; index >= 0 && remaining > 0 && result.length < TERMINAL_HISTORY_LIMIT; index -= 1) {
    const entry = all[index].slice(-remaining);
    result.push(entry);
    remaining -= entry.length + 1;
  }
  return result.reverse();
}

export function parseCommand(input: string): { command: string; args: string[] } | undefined {
  const tokens: string[] = [];
  let token = "";
  let quote: "'" | '"' | undefined;
  let escaping = false;
  for (const character of input.trim()) {
    if (escaping) { token += character; escaping = false; continue; }
    if (character === "\\" && quote !== "'") { escaping = true; continue; }
    if ((character === "'" || character === '"')) { if (quote === character) quote = undefined; else if (!quote) quote = character; else token += character; continue; }
    if (/\s/.test(character) && !quote) { if (token) { tokens.push(token); token = ""; } } else token += character;
  }
  if (quote || escaping) return undefined;
  if (token) tokens.push(token);
  const [command, ...args] = tokens;
  return command ? { command, args } : undefined;
}
