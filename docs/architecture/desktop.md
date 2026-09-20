# Desktop architecture

## Ownership

The desktop subsystem owns windows, visual layout, interaction state, accessibility, and rendering data returned by core. It does not own filesystem access, shell execution, Git operations, indexing, agent policy, or model inference.

It depends on typed Tauri commands and events. No backend subsystem may depend on React. Its public interface is the user interface plus generated TypeScript bindings planned for later phases. State logic and boundary adapters are tested separately; smoke tests cover shell landmarks and application builds.

## Phase 1 implementation

The shell has five visual regions: title bar, activity bar and sidebar, editor workspace, agent panel, and bottom panel. Pointer-accessible separators resize the sidebar, agent panel, and bottom panel within bounded dimensions. Narrow windows hide the agent panel while preserving the editor.

Theme, panel dimensions, and up to eight recent project paths are stored locally under the versioned key `helel.settings.v1`. Invalid persisted values fall back to safe defaults. The native directory dialog has permission to select a folder, but Phase 1 does not read its contents.

The application shell includes a keyboard-first command center for navigation and common workspace actions. The welcome surface renders its ambient background with a small local WebGL shader, caps pixel density, draws at no more than 30 frames per second, resizes through `ResizeObserver`, pauses while hidden, and respects the operating system reduced-motion preference. It loads no remote assets and provides a static visual fallback when WebGL is unavailable.

The Agent view can capture up to 30 seconds of microphone audio, resample it to a mono 16 kHz WAV in memory, and send it to the native local-speech boundary. A transcript fills the normal editable objective field. The active task can speak its latest observation through the operating system voice. Missing speech tools do not affect typed agent operation.

## Phase 2 editor

Monaco and its TypeScript, JSON, HTML, CSS, and base editor workers are bundled locally. Editor buffers remain in React state with separate current and saved content for dirty-state and diff calculation. Typed Tauri invocations handle workspace registration, tree refresh, text reads/writes, entry operations, and text search/replace. Monaco markers and deterministic merge-conflict checks feed the Problems panel.

## Local agent workspace

The agent panel shows the active objective, plan, pending typed action, observations, and phase. Controls support per-action approval, continue, pause, resume, cancel, and task rollback. Settings can select and health-check local model artifacts. The terminal uses a native PTY, and the workspace tree refreshes from native filesystem events with dirty-buffer conflict notices.

Saved workspaces load their persisted JSON and SQLite indexes at startup. React development remounts and repeated open actions share one in-flight workspace request. Native watcher batches update only changed or removed paths, including deleted directory prefixes; an explicit **Rebuild index** remains the full-rescan boundary. The rendered terminal history keeps the newest 2,000 entries so long-running commands cannot grow the React tree without bound.
