//! Classified, shell-free process and Git operations.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
    Ok(GitSummary {
        branch,
        changes,
        diff: String::from_utf8_lossy(&diff.stdout).into_owned(),
    })
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
    use super::{Risk, classify, create_checkpoint, restore_checkpoint};
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
}
