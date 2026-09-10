import { Icon } from "../App";
import { projectName } from "../settings";
import type { ActivityView } from "./ActivityBar";
interface Props { view: ActivityView; width: number; recentProjects: string[]; currentProject?: string; onOpenProject: () => void; onSelectProject: (path: string) => void; }
export function SidePanel({ view, width, recentProjects, currentProject, onOpenProject, onSelectProject }: Props) {
  const title = { explorer: "Explorer", search: "Search", source: "Source control", agent: "Agent history" }[view];
  return <aside className="side-panel" aria-label={title} style={{ width }}><p className="panel-title">{title}</p>{view === "explorer" ? <><button className="open-button" onClick={onOpenProject}><Icon name="folder" /> Open project</button><div className="section-label">Recent</div>{recentProjects.length === 0 ? <p className="empty-copy">Your recent projects will appear here.</p> : <ul className="recent-list">{recentProjects.map((path) => <li key={path}><button className={path === currentProject ? "current" : ""} title={path} onClick={() => onSelectProject(path)}><Icon name="folder" /><span><strong>{projectName(path)}</strong><small>{path}</small></span></button></li>)}</ul>}</> : <EmptyPanel view={view} />}</aside>;
}
function EmptyPanel({ view }: { view: Exclude<ActivityView, "explorer"> }) { const copy = { search: "Repository search arrives with the IDE core in Phase 2.", source: "Git integration arrives with the local system engine in Phase 3.", agent: "Agent sessions arrive after secure tools and repository intelligence." }[view]; return <div className="empty-panel"><p>{copy}</p><span>Planned</span></div>; }
