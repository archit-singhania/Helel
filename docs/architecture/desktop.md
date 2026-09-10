# Desktop architecture

## Ownership

The desktop subsystem owns windows, visual layout, interaction state, accessibility, and rendering data returned by core. It does not own filesystem access, shell execution, Git operations, indexing, agent policy, or model inference.

It depends on typed Tauri commands and events. No backend subsystem may depend on React. Its public interface is the user interface plus generated TypeScript bindings planned for later phases. State logic and boundary adapters will be tested separately; smoke tests will cover application startup.
