import { useEffect, useMemo, useRef, useState } from "react";
import { Icon } from "../App";

export interface CommandItem {
  id: string;
  label: string;
  detail: string;
  shortcut?: string;
  icon: "files" | "search" | "branch" | "agent" | "settings" | "folder";
  run: () => void;
}

export function CommandPalette({ commands, onClose }: { commands: CommandItem[]; onClose: () => void }) {
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const results = useMemo(() => {
    const words = query.toLowerCase().trim().split(/\s+/).filter(Boolean);
    return commands.filter((command) => words.every((word) => `${command.label} ${command.detail}`.toLowerCase().includes(word)));
  }, [commands, query]);

  useEffect(() => { inputRef.current?.focus(); }, []);
  useEffect(() => { setSelected(0); }, [query]);
  const choose = (command?: CommandItem) => { if (command) { onClose(); command.run(); } };

  return <div className="palette-backdrop" role="presentation" onMouseDown={onClose}>
    <section className="command-palette" role="dialog" aria-modal="true" aria-label="Command center" onMouseDown={(event) => event.stopPropagation()} onKeyDown={(event) => {
      if (event.key === "Escape") onClose();
      if (event.key === "ArrowDown") { event.preventDefault(); setSelected((value) => Math.min(value + 1, results.length - 1)); }
      if (event.key === "ArrowUp") { event.preventDefault(); setSelected((value) => Math.max(value - 1, 0)); }
      if (event.key === "Enter") { event.preventDefault(); choose(results[selected]); }
    }}>
      <div className="palette-search"><Icon name="search" /><input ref={inputRef} value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search actions…" aria-label="Search commands" /><kbd>esc</kbd></div>
      <div className="palette-label">Quick actions</div>
      <div className="palette-results" role="listbox">
        {results.map((command, index) => <button key={command.id} className={index === selected ? "selected" : ""} role="option" aria-selected={index === selected} onMouseEnter={() => setSelected(index)} onClick={() => choose(command)}><span className="command-icon"><Icon name={command.icon} /></span><span><strong>{command.label}</strong><small>{command.detail}</small></span>{command.shortcut && <kbd>{command.shortcut}</kbd>}</button>)}
        {results.length === 0 && <p className="palette-empty">No matching actions</p>}
      </div>
      <footer><span><b>↑↓</b> Navigate</span><span><b>↵</b> Select</span><span>Everything stays on this Mac</span></footer>
    </section>
  </div>;
}
