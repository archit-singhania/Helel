//! Filesystem access constrained to one explicitly selected workspace.

use serde::Serialize;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;
const SKIPPED_DIRECTORIES: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    ".venv",
    "__pycache__",
];

#[derive(Debug, Clone)]
pub struct Workspace {
    root: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub children: Vec<TreeEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchMatch {
    pub path: String,
    pub line: usize,
    pub preview: String,
}

impl Workspace {
    /// Registers an existing directory as the only accessible workspace.
    ///
    /// # Errors
    ///
    /// Returns an error when the path cannot be resolved or is not a directory.
    pub fn open(root: impl AsRef<Path>) -> io::Result<Self> {
        let root = root.as_ref().canonicalize()?;
        if !root.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "workspace must be a directory",
            ));
        }
        Ok(Self { root })
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
    /// Returns the visible workspace tree.
    ///
    /// # Errors
    ///
    /// Returns an error when a directory cannot be read.
    pub fn tree(&self) -> io::Result<Vec<TreeEntry>> {
        self.read_directory(&self.root)
    }

    /// Reads a bounded UTF-8 file inside the workspace.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid, inaccessible, binary, large, or escaping paths.
    pub fn read_text(&self, relative: &str) -> io::Result<String> {
        let path = self.resolve_existing(relative)?;
        let metadata = path.metadata()?;
        if !metadata.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path is not a file",
            ));
        }
        if metadata.len() > MAX_FILE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "file exceeds the 2 MiB editor limit",
            ));
        }
        fs::read_to_string(path).map_err(|error| {
            if error.kind() == io::ErrorKind::InvalidData {
                io::Error::new(io::ErrorKind::InvalidData, "file is not valid UTF-8 text")
            } else {
                error
            }
        })
    }

    /// Replaces a workspace text file with bounded content.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid, inaccessible, large, or escaping paths.
    pub fn write_text(&self, relative: &str, content: &str) -> io::Result<()> {
        if content.len() as u64 > MAX_FILE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "content exceeds the 2 MiB editor limit",
            ));
        }
        fs::write(self.resolve_existing(relative)?, content)
    }

    /// Creates an empty file without replacing an existing path.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid paths, missing parents, or existing targets.
    pub fn create_file(&self, relative: &str) -> io::Result<()> {
        let path = self.resolve_new(relative)?;
        if path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "path already exists",
            ));
        }
        fs::write(path, "")
    }
    /// Creates one directory without replacing an existing path.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid paths, missing parents, or existing targets.
    pub fn create_directory(&self, relative: &str) -> io::Result<()> {
        let path = self.resolve_new(relative)?;
        if path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "path already exists",
            ));
        }
        fs::create_dir(path)
    }
    /// Renames an existing entry within the workspace.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid paths or an existing destination.
    pub fn rename(&self, from: &str, to: &str) -> io::Result<()> {
        let source = self.resolve_existing(from)?;
        let destination = self.resolve_new(to)?;
        if destination.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "destination already exists",
            ));
        }
        fs::rename(source, destination)
    }
    /// Deletes a file or empty directory.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid paths or non-empty directories.
    pub fn delete(&self, relative: &str) -> io::Result<()> {
        let path = self.resolve_existing(relative)?;
        if path.is_dir() {
            fs::remove_dir(path)
        } else {
            fs::remove_file(path)
        }
    }

    /// Searches UTF-8 text files and returns at most the requested bounded result count.
    ///
    /// # Errors
    ///
    /// Returns an error when the workspace cannot be traversed.
    pub fn search(&self, query: &str, max_results: usize) -> io::Result<Vec<SearchMatch>> {
        if query.is_empty() {
            return Ok(Vec::new());
        }
        let mut matches = Vec::new();
        self.search_directory(&self.root, query, max_results.min(500), &mut matches)?;
        Ok(matches)
    }

    /// Replaces every exact match in bounded UTF-8 workspace files.
    ///
    /// # Errors
    ///
    /// Returns an error when the workspace cannot be traversed or a matched file cannot be saved.
    pub fn replace_all(&self, query: &str, replacement: &str) -> io::Result<usize> {
        if query.is_empty() {
            return Ok(0);
        }
        let mut paths: Vec<String> = self
            .search(query, 500)?
            .into_iter()
            .map(|item| item.path)
            .collect();
        paths.sort();
        paths.dedup();
        let mut count = 0;
        for path in paths {
            let content = self.read_text(&path)?;
            count += content.matches(query).count();
            self.write_text(&path, &content.replace(query, replacement))?;
        }
        Ok(count)
    }

    fn resolve_existing(&self, relative: &str) -> io::Result<PathBuf> {
        let joined = self.safe_join(relative)?;
        let canonical = joined.canonicalize()?;
        if !canonical.starts_with(&self.root) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "path escapes workspace",
            ));
        }
        Ok(canonical)
    }
    fn resolve_new(&self, relative: &str) -> io::Result<PathBuf> {
        let joined = self.safe_join(relative)?;
        let parent = joined
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no parent"))?
            .canonicalize()?;
        if !parent.starts_with(&self.root) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "path escapes workspace",
            ));
        }
        Ok(joined)
    }
    fn safe_join(&self, relative: &str) -> io::Result<PathBuf> {
        let path = Path::new(relative);
        if relative.is_empty()
            || path.is_absolute()
            || path
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "only normalized relative paths are allowed",
            ));
        }
        Ok(self.root.join(path))
    }

    fn read_directory(&self, directory: &Path) -> io::Result<Vec<TreeEntry>> {
        let mut entries = Vec::new();
        for item in fs::read_dir(directory)? {
            let item = item?;
            let path = item.path();
            if item.file_type()?.is_symlink() {
                continue;
            }
            let name = item.file_name().to_string_lossy().into_owned();
            if path.is_dir() && SKIPPED_DIRECTORIES.contains(&name.as_str()) {
                continue;
            }
            let is_directory = path.is_dir();
            let relative = path
                .strip_prefix(&self.root)
                .map_err(io::Error::other)?
                .to_string_lossy()
                .replace('\\', "/");
            let children = if is_directory {
                self.read_directory(&path)?
            } else {
                Vec::new()
            };
            entries.push(TreeEntry {
                name,
                path: relative,
                is_directory,
                children,
            });
        }
        entries.sort_by_key(|entry| (!entry.is_directory, entry.name.to_lowercase()));
        Ok(entries)
    }

    fn search_directory(
        &self,
        directory: &Path,
        query: &str,
        limit: usize,
        output: &mut Vec<SearchMatch>,
    ) -> io::Result<()> {
        if output.len() >= limit {
            return Ok(());
        }
        for item in fs::read_dir(directory)? {
            let item = item?;
            if item.file_type()?.is_symlink() {
                continue;
            }
            let path = item.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default();
                if !SKIPPED_DIRECTORIES.contains(&name) {
                    self.search_directory(&path, query, limit, output)?;
                }
            } else if path.metadata()?.len() <= MAX_FILE_BYTES {
                if let Ok(content) = fs::read_to_string(&path) {
                    for (index, line) in content
                        .lines()
                        .enumerate()
                        .filter(|(_, line)| line.contains(query))
                    {
                        output.push(SearchMatch {
                            path: path
                                .strip_prefix(&self.root)
                                .map_err(io::Error::other)?
                                .to_string_lossy()
                                .replace('\\', "/"),
                            line: index + 1,
                            preview: line.trim().chars().take(180).collect(),
                        });
                        if output.len() >= limit {
                            return Ok(());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Workspace;
    use std::fs;
    fn fixture() -> (tempfile::TempDir, Workspace) {
        let directory = tempfile::tempdir().expect("temporary directory");
        fs::write(directory.path().join("main.ts"), "const value = 1;\n").expect("fixture file");
        let workspace = Workspace::open(directory.path()).expect("open workspace");
        (directory, workspace)
    }
    #[test]
    fn reads_writes_and_searches_text() {
        let (_directory, workspace) = fixture();
        assert_eq!(
            workspace.read_text("main.ts").unwrap(),
            "const value = 1;\n"
        );
        workspace
            .write_text("main.ts", "const value = 2;\n")
            .unwrap();
        assert_eq!(workspace.search("value = 2", 10).unwrap()[0].line, 1);
    }
    #[test]
    fn rejects_escape_and_non_empty_directory_deletion() {
        let (directory, workspace) = fixture();
        assert!(workspace.read_text("../secret").is_err());
        fs::create_dir(directory.path().join("src")).unwrap();
        fs::write(directory.path().join("src/keep.ts"), "").unwrap();
        assert!(workspace.delete("src").is_err());
    }
    #[test]
    fn supports_create_rename_and_delete() {
        let (_directory, workspace) = fixture();
        workspace.create_directory("src").unwrap();
        workspace.create_file("src/new.ts").unwrap();
        workspace.rename("src/new.ts", "src/renamed.ts").unwrap();
        workspace.delete("src/renamed.ts").unwrap();
        workspace.delete("src").unwrap();
    }
    #[test]
    fn replaces_exact_text() {
        let (_directory, workspace) = fixture();
        assert_eq!(workspace.replace_all("value", "answer").unwrap(), 1);
        assert!(workspace.read_text("main.ts").unwrap().contains("answer"));
    }
}
