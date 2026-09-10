# Desktop architecture

## Ownership

The desktop subsystem owns windows, visual layout, interaction state, accessibility, and rendering data returned by core. It does not own filesystem access, shell execution, Git operations, indexing, agent policy, or model inference.

It depends on typed Tauri commands and events. No backend subsystem may depend on React. Its public interface is the user interface plus generated TypeScript bindings planned for later phases. State logic and boundary adapters are tested separately; smoke tests cover shell landmarks and application builds.

## Phase 1 implementation

The shell has five visual regions: title bar, activity bar and sidebar, editor workspace, agent panel, and bottom panel. Pointer-accessible separators resize the sidebar, agent panel, and bottom panel within bounded dimensions. Narrow windows hide the agent panel while preserving the editor.

Theme, panel dimensions, and up to eight recent project paths are stored locally under the versioned key `helel.settings.v1`. Invalid persisted values fall back to safe defaults. The native directory dialog has permission to select a folder, but Phase 1 does not read its contents.

## Phase 2 editor

Monaco and its TypeScript, JSON, HTML, CSS, and base editor workers are bundled locally. Editor buffers remain in React state with separate current and saved content for dirty-state and diff calculation. Typed Tauri invocations handle workspace registration, tree refresh, text reads/writes, entry operations, and text search/replace. Monaco markers and deterministic merge-conflict checks feed the Problems panel.
