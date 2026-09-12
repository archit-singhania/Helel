//! Project detection and explicit, shell-free validation plans.

use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectKind {
    Rust,
    React,
    Angular,
    Node,
    Python,
    Maven,
    Gradle,
    Flutter,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationCommand {
    pub label: String,
    pub program: String,
    pub args: Vec<String>,
    pub working_directory: String,
    pub timeout_seconds: u64,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectProfile {
    pub kind: ProjectKind,
    pub commands: Vec<ValidationCommand>,
}

fn command(label: &str, program: &str, args: &[&str]) -> ValidationCommand {
    ValidationCommand {
        label: label.into(),
        program: program.into(),
        args: args.iter().map(|v| (*v).into()).collect(),
        working_directory: ".".into(),
        timeout_seconds: 300,
        requires_approval: true,
    }
}

/// Detects a project only from checked-in metadata and returns conservative commands.
#[must_use]
pub fn detect(root: &Path) -> ProjectProfile {
    if root.join("Cargo.toml").is_file() {
        return ProjectProfile {
            kind: ProjectKind::Rust,
            commands: vec![
                command("Check", "cargo", &["check", "--offline"]),
                command("Test", "cargo", &["test", "--offline"]),
            ],
        };
    }
    if root.join("pom.xml").is_file() {
        return ProjectProfile {
            kind: ProjectKind::Maven,
            commands: vec![command("Test", "mvn", &["--offline", "test"])],
        };
    }
    if root.join("build.gradle").is_file() || root.join("build.gradle.kts").is_file() {
        return ProjectProfile {
            kind: ProjectKind::Gradle,
            commands: vec![command("Test", "./gradlew", &["--offline", "test"])],
        };
    }
    if root.join("pubspec.yaml").is_file() {
        return ProjectProfile {
            kind: ProjectKind::Flutter,
            commands: vec![
                command("Analyze", "flutter", &["analyze"]),
                command("Test", "flutter", &["test"]),
            ],
        };
    }
    if root.join("pyproject.toml").is_file() || root.join("requirements.txt").is_file() {
        return ProjectProfile {
            kind: ProjectKind::Python,
            commands: vec![command("Test", "python3", &["-m", "unittest", "discover"])],
        };
    }
    if root.join("package.json").is_file() {
        let package = fs::read_to_string(root.join("package.json")).unwrap_or_default();
        let manager = if root.join("pnpm-lock.yaml").is_file() {
            "pnpm"
        } else {
            "npm"
        };
        let kind = if package.contains("@angular/core") {
            ProjectKind::Angular
        } else if package.contains("\"react\"") {
            ProjectKind::React
        } else {
            ProjectKind::Node
        };
        let mut commands = Vec::new();
        for (label, script) in [
            ("Lint", "lint"),
            ("Typecheck", "typecheck"),
            ("Test", "test"),
            ("Build", "build"),
        ] {
            if package.contains(&format!("\"{script}\"")) {
                commands.push(command(label, manager, &["run", script]));
            }
        }
        return ProjectProfile { kind, commands };
    }
    ProjectProfile {
        kind: ProjectKind::Unknown,
        commands: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_supported_projects() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"dependencies":{"react":"local"},"scripts":{"test":"vitest"}}"#,
        )
        .unwrap();
        let result = detect(dir.path());
        assert_eq!(result.kind, ProjectKind::React);
        assert_eq!(result.commands[0].program, "npm");
        assert_eq!(result.commands[0].args, ["run", "test"]);
    }
    #[test]
    fn commands_are_offline_when_supported() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
        let result = detect(dir.path());
        assert!(
            result
                .commands
                .iter()
                .all(|c| c.args.contains(&"--offline".into()))
        );
    }
}
