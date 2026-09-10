#![allow(clippy::needless_pass_by_value)] // Tauri command extraction requires owned arguments.

use helel_core::workspace::{SearchMatch, TreeEntry, Workspace};
use std::sync::Mutex;
use tauri::State;

#[derive(Default)]
struct WorkspaceState(Mutex<Option<Workspace>>);

fn with_workspace<T>(
    state: &State<'_, WorkspaceState>,
    operation: impl FnOnce(&Workspace) -> std::io::Result<T>,
) -> Result<T, String> {
    let guard = state
        .0
        .lock()
        .map_err(|_| "workspace lock is unavailable".to_owned())?;
    let workspace = guard
        .as_ref()
        .ok_or_else(|| "no workspace is open".to_owned())?;
    operation(workspace).map_err(|error| error.to_string())
}

#[tauri::command]
fn open_workspace(
    root: String,
    state: State<'_, WorkspaceState>,
) -> Result<Vec<TreeEntry>, String> {
    let workspace = Workspace::open(root).map_err(|error| error.to_string())?;
    let tree = workspace.tree().map_err(|error| error.to_string())?;
    *state
        .0
        .lock()
        .map_err(|_| "workspace lock is unavailable".to_owned())? = Some(workspace);
    Ok(tree)
}

#[tauri::command]
fn refresh_tree(state: State<'_, WorkspaceState>) -> Result<Vec<TreeEntry>, String> {
    with_workspace(&state, Workspace::tree)
}
#[tauri::command]
fn read_file(path: String, state: State<'_, WorkspaceState>) -> Result<String, String> {
    with_workspace(&state, |workspace| workspace.read_text(&path))
}
#[tauri::command]
fn save_file(
    path: String,
    content: String,
    state: State<'_, WorkspaceState>,
) -> Result<(), String> {
    with_workspace(&state, |workspace| workspace.write_text(&path, &content))
}
#[tauri::command]
fn create_entry(
    path: String,
    directory: bool,
    state: State<'_, WorkspaceState>,
) -> Result<Vec<TreeEntry>, String> {
    with_workspace(&state, |workspace| {
        if directory {
            workspace.create_directory(&path)?;
        } else {
            workspace.create_file(&path)?;
        }
        workspace.tree()
    })
}
#[tauri::command]
fn rename_entry(
    from: String,
    to: String,
    state: State<'_, WorkspaceState>,
) -> Result<Vec<TreeEntry>, String> {
    with_workspace(&state, |workspace| {
        workspace.rename(&from, &to)?;
        workspace.tree()
    })
}
#[tauri::command]
fn delete_entry(path: String, state: State<'_, WorkspaceState>) -> Result<Vec<TreeEntry>, String> {
    with_workspace(&state, |workspace| {
        workspace.delete(&path)?;
        workspace.tree()
    })
}
#[tauri::command]
fn search_workspace(
    query: String,
    state: State<'_, WorkspaceState>,
) -> Result<Vec<SearchMatch>, String> {
    with_workspace(&state, |workspace| workspace.search(&query, 200))
}

#[tauri::command]
fn replace_workspace(
    query: String,
    replacement: String,
    state: State<'_, WorkspaceState>,
) -> Result<usize, String> {
    with_workspace(&state, |workspace| {
        workspace.replace_all(&query, &replacement)
    })
}

/// Starts the native Helel application event loop.
///
/// # Panics
///
/// Panics when Tauri cannot initialize or run the application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(WorkspaceState::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            open_workspace,
            refresh_tree,
            read_file,
            save_file,
            create_entry,
            rename_entry,
            delete_entry,
            search_workspace,
            replace_workspace
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Helel desktop application");
}
