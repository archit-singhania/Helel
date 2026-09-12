//! Classified, shell-free process and Git operations.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Risk {
    Safe,
    Modify,
    Dangerous,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessResult {
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub risk: Risk,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitSummary {
    pub branch: String,
    pub changes: Vec<String>,
    pub diff: String,
    pub staged_diff: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointEntry {
    pub path: String,
    pub existed: bool,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCheckpoint {
    pub entries: Vec<CheckpointEntry>,
}

/// Creates a task-scoped checkpoint. Paths must be relative and remain inside `root`.
///
/// # Errors
/// Returns an error for escaping paths or unreadable files.
pub fn create_checkpoint(root: &Path, paths: &[PathBuf]) -> io::Result<TaskCheckpoint> {
    let root = root.canonicalize()?;
    let mut entries = Vec::new();
    for relative in paths {
        if relative.is_absolute()
            || relative
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "checkpoint path escapes workspace",
            ));
        }
        let path = root.join(relative);
        let existed = path.is_file();
        let bytes = if existed {
            fs::read(&path)?
        } else {
            Vec::new()
        };
        entries.push(CheckpointEntry {
            path: relative.to_string_lossy().into_owned(),
            existed,
            bytes,
        });
    }
    Ok(TaskCheckpoint { entries })
}

/// Restores only checkpointed paths, preserving unrelated user changes.
///
/// # Errors
/// Returns an error for escaping paths or failed filesystem restoration.
pub fn restore_checkpoint(root: &Path, checkpoint: &TaskCheckpoint) -> io::Result<()> {
    let root = root.canonicalize()?;
    for entry in &checkpoint.entries {
        let relative = Path::new(&entry.path);
        if relative.is_absolute()
            || relative
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "checkpoint path escapes workspace",
            ));
        }
        let path = root.join(relative);
        if entry.existed {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, &entry.bytes)?;
        } else if path.exists() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

/// Returns the workspace-relative files declared by a unified Git patch.
///
/// # Errors
/// Returns an error when the patch declares no paths or an unsafe path.
pub fn patch_paths(patch: &str) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for line in patch
        .lines()
        .filter(|line| line.starts_with("+++ ") || line.starts_with("--- "))
    {
        let raw = line
            .strip_prefix("+++ ")
            .or_else(|| line.strip_prefix("--- "))
            .unwrap_or_default()
            .split('\t')
            .next()
            .unwrap_or_default();
        if raw == "/dev/null" {
            continue;
        }
        let relative = raw
            .strip_prefix("b/")
            .or_else(|| raw.strip_prefix("a/"))
            .unwrap_or(raw);
        let target_path = PathBuf::from(relative);
        if target_path.is_absolute()
            || target_path
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "patch path escapes workspace",
            ));
        }
        if !paths.contains(&target_path) {
            paths.push(target_path);
        }
    }
    if paths.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "patch contains no target paths",
        ));
    }
    Ok(paths)
}

/// Applies a patch and durably stores the exact task-scoped rollback checkpoint.
///
/// # Errors
/// Returns an error when validation, checkpointing, patching, or persistence fails.
pub fn apply_transactional_patch(
    root: &Path,
    patch: &str,
    reverse: bool,
) -> io::Result<TaskCheckpoint> {
    let paths = patch_paths(patch)?;
    let checkpoint = create_checkpoint(root, &paths)?;
    if let Err(error) = apply_patch(root, patch, reverse) {
        restore_checkpoint(root, &checkpoint)?;
        return Err(error);
    }
    let directory = root.join(".helel");
    fs::create_dir_all(&directory)?;
    let temporary = directory.join("checkpoint.json.tmp");
    fs::write(
        &temporary,
        serde_json::to_vec(&checkpoint).map_err(io::Error::other)?,
    )?;
    fs::rename(temporary, directory.join("checkpoint.json"))?;
    Ok(checkpoint)
}

/// Restores and removes the most recent task checkpoint.
///
/// # Errors
/// Returns an error if the checkpoint is absent, corrupt, or cannot be restored.
pub fn rollback_last_patch(root: &Path) -> io::Result<()> {
    let path = root.join(".helel/checkpoint.json");
    let checkpoint: TaskCheckpoint =
        serde_json::from_slice(&fs::read(&path)?).map_err(io::Error::other)?;
    restore_checkpoint(root, &checkpoint)?;
    fs::remove_file(path)
}

#[must_use]
pub fn classify(command: &str, args: &[String]) -> Risk {
    let first = args.first().map(String::as_str).unwrap_or_default();
    let suspicious_path = args
        .iter()
        .any(|arg| arg == "-C" || arg.starts_with('/') || arg.split('/').any(|part| part == ".."));
    if matches!(
        command,
        "rm" | "sudo" | "shutdown" | "reboot" | "mkfs" | "dd" | "kill" | "killall"
    ) {
        return Risk::Dangerous;
    }
    if command == "git" && matches!(first, "reset" | "clean" | "restore" | "rebase") {
        return Risk::Dangerous;
    }
    if (command == "git"
        && !suspicious_path
        && matches!(first, "status" | "diff" | "log" | "show" | "branch"))
        || matches!(
            command,
            "pwd" | "ls" | "rg" | "grep" | "find" | "cat" | "head" | "tail" | "wc"
        )
    {
        return if suspicious_path {
            Risk::Dangerous
        } else {
            Risk::Safe
        };
    }
    if matches!(
        command,
        "git" | "cargo" | "rustc" | "pnpm" | "npm" | "mkdir" | "touch" | "mv" | "cp"
    ) {
        return Risk::Modify;
    }
    Risk::Dangerous
}

/// Runs a program directly without a command shell.
///
/// # Errors
///
/// Returns an error if approval is missing or the process cannot be run.
pub fn run(
    root: &Path,
    command: &str,
    args: &[String],
    approved: bool,
) -> io::Result<ProcessResult> {
    let risk = classify(command, args);
    if risk != Risk::Safe && !approved {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("{risk:?} command requires approval"),
        ));
    }
    let output = Command::new(command)
        .args(args)
        .current_dir(root)
        .env("HELEL_WORKSPACE", root)
        .output()?;
    Ok(result(command, &output, risk))
}

/// Reads branch, working-tree changes, and the current unstaged diff.
///
/// # Errors
///
/// Returns an error when Git is unavailable or the workspace is not a repository.
pub fn git_summary(root: &Path) -> io::Result<GitSummary> {
    let status = command_output(root, "git", &["status", "--porcelain=v1", "--branch"])?;
    if !status.status.success() {
        return Err(io::Error::other(
            String::from_utf8_lossy(&status.stderr).trim().to_owned(),
        ));
    }
    let text = String::from_utf8_lossy(&status.stdout);
    let mut lines = text.lines();
    let branch = lines
        .next()
        .unwrap_or("## detached")
        .trim_start_matches("## ")
        .split("...")
        .next()
        .unwrap_or("detached")
        .to_owned();
    let changes = lines.map(str::to_owned).collect();
    let diff = command_output(root, "git", &["diff", "--", "."])?;
    let staged_diff = command_output(root, "git", &["diff", "--cached", "--", "."])?;
    Ok(GitSummary {
        branch,
        changes,
        diff: String::from_utf8_lossy(&diff.stdout).into_owned(),
        staged_diff: String::from_utf8_lossy(&staged_diff.stdout).into_owned(),
    })
}

/// Stages explicitly selected workspace-relative paths.
///
/// # Errors
/// Returns an error for unsafe paths or a failed Git operation.
pub fn git_stage(root: &Path, paths: &[String]) -> io::Result<()> {
    if paths.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "no paths selected",
        ));
    }
    if paths.iter().any(|path| {
        Path::new(path).is_absolute()
            || Path::new(path)
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
    }) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Git path escapes workspace",
        ));
    }
    let mut command = Command::new("git");
    command.arg("add").arg("--").args(paths).current_dir(root);
    let output = command.output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ))
    }
}

/// Creates a local commit from the staged index.
///
/// # Errors
/// Returns an error for an invalid message or failed Git commit.
pub fn git_commit(root: &Path, message: &str) -> io::Result<String> {
    let message = message.trim();
    if message.is_empty() || message.len() > 4096 || message.contains('\0') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid commit message",
        ));
    }
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(root)
        .output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ))
    }
}

/// Checks and applies or reverses a unified Git patch.
///
/// # Errors
///
/// Returns an error when the patch is invalid or cannot be applied atomically.
pub fn apply_patch(root: &Path, patch: &str, reverse: bool) -> io::Result<()> {
    let mut check_args = vec!["apply", "--check", "--whitespace=error-all"];
    if reverse {
        check_args.push("--reverse");
    }
    feed_git(root, &check_args, patch)?;
    let mut apply_args = vec!["apply", "--whitespace=error-all"];
    if reverse {
        apply_args.push("--reverse");
    }
    feed_git(root, &apply_args, patch)
}

fn command_output(root: &Path, command: &str, args: &[&str]) -> io::Result<Output> {
    Command::new(command).args(args).current_dir(root).output()
}
fn feed_git(root: &Path, args: &[&str], input: &str) -> io::Result<()> {
    let mut child = Command::new("git")
        .args(args)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("Git stdin unavailable"))?
        .write_all(input.as_bytes())?;
    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ))
    }
}
fn result(command: &str, output: &Output, risk: Risk) -> ProcessResult {
    ProcessResult {
        command: command.to_owned(),
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        risk,
    }
}

#[cfg(test)]
mod tests {
    use super::{Risk, classify, create_checkpoint, patch_paths, restore_checkpoint};
    use std::{fs, path::PathBuf};
    #[test]
    fn classifies_commands() {
        assert_eq!(classify("git", &["status".into()]), Risk::Safe);
        assert_eq!(classify("git", &["commit".into()]), Risk::Modify);
        assert_eq!(classify("cargo", &["test".into()]), Risk::Modify);
        assert_eq!(classify("cat", &["../secret".into()]), Risk::Dangerous);
        assert_eq!(classify("rm", &["file".into()]), Risk::Dangerous);
        assert_eq!(classify("sh", &["-c".into()]), Risk::Dangerous);
    }
    #[test]
    fn checkpoint_preserves_unrelated_changes() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("owned.txt"), "before").unwrap();
        fs::write(dir.path().join("user.txt"), "user-before").unwrap();
        let checkpoint = create_checkpoint(
            dir.path(),
            &[PathBuf::from("owned.txt"), PathBuf::from("new.txt")],
        )
        .unwrap();
        fs::write(dir.path().join("owned.txt"), "after").unwrap();
        fs::write(dir.path().join("new.txt"), "new").unwrap();
        fs::write(dir.path().join("user.txt"), "user-after").unwrap();
        restore_checkpoint(dir.path(), &checkpoint).unwrap();
        assert_eq!(
            fs::read_to_string(dir.path().join("owned.txt")).unwrap(),
            "before"
        );
        assert!(!dir.path().join("new.txt").exists());
        assert_eq!(
            fs::read_to_string(dir.path().join("user.txt")).unwrap(),
            "user-after"
        );
    }
    #[test]
    fn patch_paths_reject_escape() {
        assert_eq!(
            patch_paths("+++ b/src/lib.rs\n").unwrap(),
            [PathBuf::from("src/lib.rs")]
        );
        assert!(patch_paths("+++ b/../outside\n").is_err());
    }

    #[test]
    fn patch_paths_include_deleted_files() {
        assert_eq!(
            patch_paths("--- a/src/old.rs\n+++ /dev/null\n").unwrap(),
            [PathBuf::from("src/old.rs")]
        );
    }
}
