//! Tree-sitter parsing persisted in a local `SQLite` repository index.

use rusqlite::{Connection, params};
use serde::Serialize;
use std::{fs, io, path::Path};
use tree_sitter::{Language, Node, Parser};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticSummary {
    pub files: usize,
    pub symbols: usize,
    pub references: usize,
    pub database: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticLocation {
    pub name: String,
    pub kind: String,
    pub path: String,
    pub line: usize,
    pub column: usize,
}

fn database(root: &Path) -> std::path::PathBuf {
    root.join(".helel/index.sqlite")
}
fn sqlite(error: rusqlite::Error) -> io::Error {
    io::Error::other(error)
}
fn language(path: &str) -> Option<Language> {
    let extension = Path::new(path).extension()?.to_str()?;
    Some(match extension {
        "rs" => tree_sitter_rust::LANGUAGE.into(),
        "py" => tree_sitter_python::LANGUAGE.into(),
        "ts" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        "tsx" => tree_sitter_typescript::LANGUAGE_TSX.into(),
        "java" => tree_sitter_java::LANGUAGE.into(),
        _ => return None,
    })
}
fn walk(
    node: Node<'_>,
    source: &str,
    path: &str,
    symbols: &mut Vec<SemanticLocation>,
    references: &mut Vec<SemanticLocation>,
) {
    if matches!(
        node.kind(),
        "identifier" | "type_identifier" | "field_identifier"
    ) {
        if let Ok(name) = node.utf8_text(source.as_bytes()) {
            let point = node.start_position();
            references.push(SemanticLocation {
                name: name.into(),
                kind: "reference".into(),
                path: path.into(),
                line: point.row + 1,
                column: point.column + 1,
            });
        }
    }
    if (node.kind().ends_with("_declaration")
        || node.kind().ends_with("_definition")
        || node.kind().ends_with("_item"))
        && node.child_by_field_name("name").is_some()
    {
        let name_node = node.child_by_field_name("name").expect("checked");
        if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
            let point = name_node.start_position();
            symbols.push(SemanticLocation {
                name: name.into(),
                kind: node.kind().into(),
                path: path.into(),
                line: point.row + 1,
                column: point.column + 1,
            });
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(child, source, path, symbols, references);
    }
}

/// Rebuilds the compiler-grade `SQLite` index atomically from supported source files.
///
/// # Errors
/// Returns an error for unreadable sources, parser failures, or `SQLite` failures.
pub fn rebuild(root: &Path) -> io::Result<SemanticSummary> {
    let root = root.canonicalize()?;
    fs::create_dir_all(root.join(".helel"))?;
    let path = database(&root);
    let temporary = root.join(".helel/index.sqlite.tmp");
    let _ = fs::remove_file(&temporary);
    let mut connection = Connection::open(&temporary).map_err(sqlite)?;
    connection.execute_batch("PRAGMA journal_mode=OFF; CREATE TABLE files(path TEXT PRIMARY KEY, language TEXT NOT NULL, content TEXT NOT NULL); CREATE TABLE symbols(name TEXT NOT NULL, kind TEXT NOT NULL, path TEXT NOT NULL, line INTEGER NOT NULL, column INTEGER NOT NULL); CREATE INDEX symbols_name ON symbols(name); CREATE TABLE refs(name TEXT NOT NULL, path TEXT NOT NULL, line INTEGER NOT NULL, column INTEGER NOT NULL); CREATE INDEX refs_name ON refs(name); CREATE VIRTUAL TABLE context USING fts5(path UNINDEXED, content);").map_err(sqlite)?;
    let legacy = crate::intelligence::CodeIndex::build(&root)?;
    let transaction = connection.transaction().map_err(sqlite)?;
    let mut symbol_count = 0;
    let mut reference_count = 0;
    let mut file_count = 0;
    for file in &legacy.files {
        let Some(grammar) = language(&file.path) else {
            continue;
        };
        let source = fs::read_to_string(root.join(&file.path))?;
        let mut parser = Parser::new();
        parser.set_language(&grammar).map_err(io::Error::other)?;
        let tree = parser
            .parse(&source, None)
            .ok_or_else(|| io::Error::other("Tree-sitter parser returned no tree"))?;
        let mut symbols = Vec::new();
        let mut references = Vec::new();
        walk(
            tree.root_node(),
            &source,
            &file.path,
            &mut symbols,
            &mut references,
        );
        transaction
            .execute(
                "INSERT INTO files VALUES (?1,?2,?3)",
                params![file.path, file.language, source],
            )
            .map_err(sqlite)?;
        transaction
            .execute(
                "INSERT INTO context VALUES (?1,?2)",
                params![file.path, source],
            )
            .map_err(sqlite)?;
        for item in symbols {
            transaction
                .execute(
                    "INSERT INTO symbols VALUES (?1,?2,?3,?4,?5)",
                    params![item.name, item.kind, item.path, item.line, item.column],
                )
                .map_err(sqlite)?;
            symbol_count += 1;
        }
        for item in references {
            transaction
                .execute(
                    "INSERT INTO refs VALUES (?1,?2,?3,?4)",
                    params![item.name, item.path, item.line, item.column],
                )
                .map_err(sqlite)?;
            reference_count += 1;
        }
        file_count += 1;
    }
    transaction.commit().map_err(sqlite)?;
    drop(connection);
    fs::rename(&temporary, &path)?;
    Ok(SemanticSummary {
        files: file_count,
        symbols: symbol_count,
        references: reference_count,
        database: path.to_string_lossy().into_owned(),
    })
}

/// Finds exact symbol definitions from the persisted `SQLite` index.
///
/// # Errors
/// Returns an error when the database or query is unavailable.
pub fn definitions(root: &Path, name: &str) -> io::Result<Vec<SemanticLocation>> {
    let connection = Connection::open(database(root)).map_err(sqlite)?;
    let mut statement=connection.prepare("SELECT name,kind,path,line,column FROM symbols WHERE name=?1 ORDER BY path,line LIMIT 100").map_err(sqlite)?;
    statement
        .query_map([name], |row| {
            Ok(SemanticLocation {
                name: row.get(0)?,
                kind: row.get(1)?,
                path: row.get(2)?,
                line: row.get(3)?,
                column: row.get(4)?,
            })
        })
        .map_err(sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sqlite)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn indexes_multiple_languages_exactly() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("lib.rs"), "fn answer() -> i32 { answer() }").unwrap();
        fs::write(d.path().join("service.py"), "class Service:\n    pass\n").unwrap();
        let summary = rebuild(d.path()).unwrap();
        assert_eq!(summary.files, 2);
        assert_eq!(definitions(d.path(), "answer").unwrap()[0].line, 1);
        assert_eq!(
            definitions(d.path(), "Service").unwrap()[0].path,
            "service.py"
        );
    }
}
