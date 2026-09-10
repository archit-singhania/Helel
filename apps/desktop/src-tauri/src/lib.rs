#![allow(clippy::needless_pass_by_value)] // Tauri command extraction requires owned arguments.

use helel_core::system::{self, GitSummary, Risk};
use helel_core::workspace::{SearchMatch, TreeEntry, Workspace};
use serde::Serialize;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, State};

#[derive(Default)]
struct WorkspaceState(Mutex<Option<Workspace>>);

#[derive(Default)]
struct AuditState(Mutex<Vec<AuditEntry>>);

#[derive(Default)]
struct ProcessState {
    next_id: AtomicU64,
    children: Arc<Mutex<HashMap<u64, Arc<Mutex<Child>>>>>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuditEntry {
    timestamp_ms: u128,
    action: String,
    risk: Risk,
    approved: bool,
    success: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessStarted {
    id: u64,
    risk: Risk,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessOutput {
    id: u64,
    stream: String,
    line: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessExit {
    id: u64,
    exit_code: Option<i32>,
}

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

#[tauri::command]
fn classify_command(command: String, args: Vec<String>) -> Risk {
    system::classify(&command, &args)
}

#[tauri::command]
fn run_command(
    command: String,
    args: Vec<String>,
    approved: bool,
    state: State<'_, WorkspaceState>,
    processes: State<'_, ProcessState>,
    audit: State<'_, AuditState>,
    app: AppHandle,
) -> Result<ProcessStarted, String> {
    let risk = system::classify(&command, &args);
    if risk != Risk::Safe && !approved {
        return Err(format!("{risk:?} command requires approval"));
    }
    let root = with_workspace(&state, |workspace| Ok(workspace.root().to_path_buf()))?;
    let mut process = Command::new(&command)
        .args(&args)
        .current_dir(&root)
        .env("HELEL_WORKSPACE", &root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    let stdout = process.stdout.take();
    let stderr = process.stderr.take();
    let id = processes.next_id.fetch_add(1, Ordering::Relaxed) + 1;
    let process = Arc::new(Mutex::new(process));
    processes
        .children
        .lock()
        .map_err(|_| "process lock is unavailable".to_owned())?
        .insert(id, Arc::clone(&process));
    stream_pipe(stdout, id, "stdout", app.clone());
    stream_pipe(stderr, id, "stderr", app.clone());
    let children = Arc::clone(&processes.children);
    std::thread::spawn(move || {
        loop {
            let status = process
                .lock()
                .ok()
                .and_then(|mut child| child.try_wait().ok())
                .flatten();
            if let Some(status) = status {
                if let Ok(mut items) = children.lock() {
                    items.remove(&id);
                }
                let _ = app.emit(
                    "process-exit",
                    ProcessExit {
                        id,
                        exit_code: status.code(),
                    },
                );
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(40));
        }
    });
    let entry = AuditEntry {
        timestamp_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        action: format!("{} {}", command, args.join(" ")),
        risk,
        approved,
        success: true,
    };
    if let Ok(mut entries) = audit.0.lock() {
        entries.push(entry);
    }
    Ok(ProcessStarted { id, risk })
}

fn stream_pipe<R: std::io::Read + Send + 'static>(
    pipe: Option<R>,
    id: u64,
    stream: &str,
    app: AppHandle,
) {
    let stream = stream.to_owned();
    std::thread::spawn(move || {
        if let Some(pipe) = pipe {
            for line in BufReader::new(pipe).lines().map_while(Result::ok) {
                let _ = app.emit(
                    "process-output",
                    ProcessOutput {
                        id,
                        stream: stream.clone(),
                        line,
                    },
                );
            }
        }
    });
}

#[tauri::command]
fn cancel_command(
    id: u64,
    processes: State<'_, ProcessState>,
    audit: State<'_, AuditState>,
) -> Result<(), String> {
    let child = processes
        .children
        .lock()
        .map_err(|_| "process lock is unavailable".to_owned())?
        .get(&id)
        .cloned()
        .ok_or_else(|| "process is no longer running".to_owned())?;
    child
        .lock()
        .map_err(|_| "process lock is unavailable".to_owned())?
        .kill()
        .map_err(|error| error.to_string())?;
    if let Ok(mut entries) = audit.0.lock() {
        entries.push(AuditEntry {
            timestamp_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            action: format!("cancel process {id}"),
            risk: Risk::Modify,
            approved: true,
            success: true,
        });
    }
    Ok(())
}

#[tauri::command]
fn git_status(state: State<'_, WorkspaceState>) -> Result<GitSummary, String> {
    with_workspace(&state, |workspace| system::git_summary(workspace.root()))
}

#[tauri::command]
fn apply_workspace_patch(
    patch: String,
    reverse: bool,
    approved: bool,
    state: State<'_, WorkspaceState>,
    audit: State<'_, AuditState>,
) -> Result<Vec<TreeEntry>, String> {
    if !approved {
        return Err("patch application requires approval".to_owned());
    }
    let result = with_workspace(&state, |workspace| {
        system::apply_patch(workspace.root(), &patch, reverse)?;
        workspace.tree()
    });
    if let Ok(mut entries) = audit.0.lock() {
        entries.push(AuditEntry {
            timestamp_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            action: if reverse {
                "reverse patch".into()
            } else {
                "apply patch".into()
            },
            risk: Risk::Modify,
            approved,
            success: result.is_ok(),
        });
    }
    result
}

#[tauri::command]
fn audit_log(audit: State<'_, AuditState>) -> Result<Vec<AuditEntry>, String> {
    audit
        .0
        .lock()
        .map(|items| items.clone())
        .map_err(|_| "audit lock is unavailable".to_owned())
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
        .manage(AuditState::default())
        .manage(ProcessState::default())
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
            replace_workspace,
            classify_command,
            run_command,
            cancel_command,
            git_status,
            apply_workspace_patch,
            audit_log
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Helel desktop application");
}
