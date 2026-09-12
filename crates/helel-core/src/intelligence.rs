//! Bounded, local repository indexing and context retrieval.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_FILES: usize = 20_000;
const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;
const SKIP: &[&str] = &[
    ".git",
    ".helel",
    "node_modules",
    "target",
    "dist",
    ".venv",
    "__pycache__",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectProfile {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub manifests: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexedFile {
    pub path: String,
    pub language: String,
    pub bytes: u64,
    pub modified_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
    pub name: String,
    pub kind: String,
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    pub name: String,
    pub path: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextHit {
    pub path: String,
    pub line: usize,
    pub score: u32,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeIndex {
    pub version: u32,
    pub generated_at_ms: u128,
    pub profile: ProjectProfile,
    pub files: Vec<IndexedFile>,
    pub symbols: Vec<Symbol>,
    pub references: Vec<Reference>,
}

impl CodeIndex {
    /// Builds a deterministic index from bounded UTF-8 workspace files.
    ///
    /// # Errors
    /// Returns an error when the workspace cannot be traversed.
    pub fn build(root: &Path) -> io::Result<Self> {
        let root = root.canonicalize()?;
        let mut paths = Vec::new();
        collect(&root, &root, &mut paths)?;
        paths.sort();
        paths.truncate(MAX_FILES);
        let mut files = Vec::new();
        let mut symbols = Vec::new();
        let mut references = Vec::new();
        let mut language_counts = HashMap::<String, usize>::new();
        let mut manifests = Vec::new();
        for path in paths {
            let metadata = path.metadata()?;
            if metadata.len() > MAX_FILE_BYTES {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            let relative = relative_path(&root, &path)?;
            let language = language_for(&path).to_owned();
            if language == "text" && !is_manifest(&relative) {
                continue;
            }
            *language_counts.entry(language.clone()).or_default() += 1;
            if is_manifest(&relative) {
                manifests.push(relative.clone());
            }
            parse(
                &relative,
                &language,
                &content,
                &mut symbols,
                &mut references,
            );
            files.push(IndexedFile {
                path: relative,
                language,
                bytes: metadata.len(),
                modified_ms: metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .map_or(0, |duration| duration.as_millis()),
            });
        }
        let mut languages: Vec<_> = language_counts.into_iter().collect();
        languages.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let languages = languages.into_iter().map(|item| item.0).collect();
        let frameworks = detect_frameworks(&root, &manifests);
        Ok(Self {
            version: 1,
            generated_at_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            profile: ProjectProfile {
                languages,
                frameworks,
                manifests,
            },
            files,
            symbols,
            references,
        })
    }

    /// Persists the index beneath `.helel` in the workspace.
    ///
    /// # Errors
    /// Returns an error if serialization or the atomic write fails.
    pub fn save(&self, root: &Path) -> io::Result<()> {
        let directory = root.join(".helel");
        fs::create_dir_all(&directory)?;
        let temporary = directory.join("index.json.tmp");
        let output = serde_json::to_vec(self).map_err(io::Error::other)?;
        fs::write(&temporary, output)?;
        fs::rename(temporary, directory.join("index.json"))
    }

    /// Loads a previously saved index.
    ///
    /// # Errors
    /// Returns an error if the file is missing, unreadable, or incompatible.
    pub fn load(root: &Path) -> io::Result<Self> {
        let index: Self = serde_json::from_slice(&fs::read(root.join(".helel/index.json"))?)
            .map_err(io::Error::other)?;
        if index.version == 1 {
            Ok(index)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported index version",
            ))
        }
    }

    /// Reindexes one existing text file without rebuilding the repository.
    ///
    /// # Errors
    /// Returns an error when the path escapes the workspace or cannot be read.
    pub fn update_file(&mut self, root: &Path, relative: &str) -> io::Result<()> {
        let root = root.canonicalize()?;
        let path = root.join(relative).canonicalize()?;
        if !path.starts_with(&root) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "path escapes workspace",
            ));
        }
        self.files.retain(|file| file.path != relative);
        self.symbols.retain(|symbol| symbol.path != relative);
        self.references
            .retain(|reference| reference.path != relative);
        let metadata = path.metadata()?;
        if metadata.len() <= MAX_FILE_BYTES {
            let content = fs::read_to_string(&path)?;
            let language = language_for(&path).to_owned();
            if language == "text" && !is_manifest(relative) {
                self.generated_at_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis();
                return self.save(&root);
            }
            parse(
                relative,
                &language,
                &content,
                &mut self.symbols,
                &mut self.references,
            );
            self.files.push(IndexedFile {
                path: relative.to_owned(),
                language,
                bytes: metadata.len(),
                modified_ms: metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .map_or(0, |duration| duration.as_millis()),
            });
            self.files.sort_by(|a, b| a.path.cmp(&b.path));
        }
        self.generated_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        self.save(&root)
    }

    /// Removes one deleted or renamed file from the in-memory and persisted index.
    ///
    /// # Errors
    /// Returns an error when the updated index cannot be persisted.
    pub fn remove_file(&mut self, root: &Path, relative: &str) -> io::Result<()> {
        self.files.retain(|file| file.path != relative);
        self.symbols.retain(|symbol| symbol.path != relative);
        self.references
            .retain(|reference| reference.path != relative);
        self.generated_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        self.save(root)
    }

    #[must_use]
    pub fn definitions(&self, name: &str, limit: usize) -> Vec<Symbol> {
        self.symbols
            .iter()
            .filter(|symbol| symbol.name == name)
            .take(limit.min(100))
            .cloned()
            .collect()
    }
    #[must_use]
    pub fn references(&self, name: &str, limit: usize) -> Vec<Reference> {
        self.references
            .iter()
            .filter(|reference| reference.name == name)
            .take(limit.min(500))
            .cloned()
            .collect()
    }

    /// Retrieves ranked, bounded local context.
    ///
    /// # Errors
    /// Returns an error if an indexed file cannot be read.
    pub fn context(&self, root: &Path, query: &str, limit: usize) -> io::Result<Vec<ContextHit>> {
        let terms: Vec<String> = query
            .split(|character: char| !character.is_alphanumeric() && character != '_')
            .filter(|term| !term.is_empty())
            .map(str::to_lowercase)
            .collect();
        if terms.is_empty() {
            return Ok(Vec::new());
        }
        let mut hits = Vec::new();
        for file in &self.files {
            let content = fs::read_to_string(root.join(&file.path))?;
            for (offset, line) in content.lines().enumerate() {
                let lower = line.to_lowercase();
                let matched = u32::try_from(
                    terms
                        .iter()
                        .filter(|term| lower.contains(term.as_str()))
                        .count(),
                )
                .unwrap_or(u32::MAX);
                if matched > 0 {
                    let symbol_bonus = self.symbols.iter().any(|symbol| {
                        symbol.path == file.path
                            && symbol.line == offset + 1
                            && terms.iter().any(|term| term == &symbol.name.to_lowercase())
                    });
                    hits.push(ContextHit {
                        path: file.path.clone(),
                        line: offset + 1,
                        score: matched * 10 + u32::from(symbol_bonus) * 20,
                        preview: line.trim().chars().take(240).collect(),
                    });
                }
            }
        }
        hits.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then(a.path.cmp(&b.path))
                .then(a.line.cmp(&b.line))
        });
        hits.truncate(limit.min(100));
        Ok(hits)
    }
}

fn collect(root: &Path, directory: &Path, output: &mut Vec<PathBuf>) -> io::Result<()> {
    if output.len() >= MAX_FILES {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_symlink() {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            if !SKIP.contains(&entry.file_name().to_string_lossy().as_ref()) {
                collect(root, &path, output)?;
            }
        } else if path.starts_with(root) {
            output.push(path);
        }
    }
    Ok(())
}
fn relative_path(root: &Path, path: &Path) -> io::Result<String> {
    Ok(path
        .strip_prefix(root)
        .map_err(io::Error::other)?
        .to_string_lossy()
        .replace('\\', "/"))
}
fn language_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
    {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" => "python",
        "go" => "go",
        "java" => "java",
        "c" | "h" => "c",
        "cc" | "cpp" | "hpp" => "cpp",
        "json" => "json",
        "toml" => "toml",
        "md" => "markdown",
        "css" => "css",
        "html" => "html",
        _ => "text",
    }
}
fn is_manifest(path: &str) -> bool {
    matches!(
        path,
        "Cargo.toml" | "package.json" | "pyproject.toml" | "requirements.txt" | "go.mod"
    )
}
fn detect_frameworks(root: &Path, manifests: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    for manifest in manifests {
        let text = fs::read_to_string(root.join(manifest))
            .unwrap_or_default()
            .to_lowercase();
        for (needle, name) in [
            ("tauri", "Tauri"),
            ("react", "React"),
            ("next", "Next.js"),
            ("django", "Django"),
            ("fastapi", "FastAPI"),
            ("axum", "Axum"),
        ] {
            if text.contains(needle) && !result.contains(&name.to_owned()) {
                result.push(name.to_owned());
            }
        }
    }
    result
}
fn parse(
    path: &str,
    language: &str,
    content: &str,
    symbols: &mut Vec<Symbol>,
    references: &mut Vec<Reference>,
) {
    for (offset, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        let patterns: &[(&str, &str)] = match language {
            "rust" => &[
                ("pub fn ", "function"),
                ("fn ", "function"),
                ("pub struct ", "struct"),
                ("struct ", "struct"),
                ("pub enum ", "enum"),
                ("enum ", "enum"),
                ("trait ", "trait"),
            ],
            "typescript" | "javascript" => &[
                ("export function ", "function"),
                ("function ", "function"),
                ("export class ", "class"),
                ("class ", "class"),
                ("export interface ", "interface"),
                ("interface ", "interface"),
                ("export const ", "constant"),
                ("const ", "constant"),
            ],
            "python" => &[("def ", "function"), ("class ", "class")],
            _ => &[],
        };
        for (prefix, kind) in patterns {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                let name: String = rest
                    .chars()
                    .take_while(|character| character.is_alphanumeric() || *character == '_')
                    .collect();
                if !name.is_empty() {
                    symbols.push(Symbol {
                        name,
                        kind: (*kind).to_owned(),
                        path: path.to_owned(),
                        line: offset + 1,
                        column: line.len() - trimmed.len() + prefix.len() + 1,
                        signature: trimmed.chars().take(240).collect(),
                    });
                }
                break;
            }
        }
        for (column, word) in identifiers(line) {
            references.push(Reference {
                name: word,
                path: path.to_owned(),
                line: offset + 1,
                column,
            });
        }
    }
}
fn identifiers(line: &str) -> Vec<(usize, String)> {
    let mut output = Vec::new();
    let mut start = None;
    for (index, character) in line
        .char_indices()
        .chain(std::iter::once((line.len(), ' ')))
    {
        if character.is_alphanumeric() || character == '_' {
            start.get_or_insert(index);
        } else if let Some(begin) = start.take() {
            let word = &line[begin..index];
            if word.len() > 2 {
                output.push((begin + 1, word.to_owned()));
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::CodeIndex;
    use std::fs;
    #[test]
    fn indexes_persists_and_retrieves_context() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("Cargo.toml"),
            "[dependencies]\ntauri = \"2\"\n",
        )
        .unwrap();
        fs::write(
            directory.path().join("main.rs"),
            "pub fn greet(name: &str) { println!(\"{name}\"); }\nfn main() { greet(\"Helel\"); }\n",
        )
        .unwrap();
        let index = CodeIndex::build(directory.path()).unwrap();
        assert_eq!(index.definitions("greet", 10).len(), 1);
        assert!(index.references("greet", 10).len() >= 2);
        assert_eq!(
            index.context(directory.path(), "greet", 3).unwrap()[0].score,
            30
        );
        index.save(directory.path()).unwrap();
        assert_eq!(CodeIndex::load(directory.path()).unwrap().files.len(), 2);
        fs::write(directory.path().join("main.rs"), "pub fn farewell() {}\n").unwrap();
        let mut loaded = CodeIndex::load(directory.path()).unwrap();
        loaded.update_file(directory.path(), "main.rs").unwrap();
        assert_eq!(loaded.definitions("farewell", 10).len(), 1);
        assert!(loaded.definitions("greet", 10).is_empty());
    }
}
