//! Strict model-proposal decoder at the trusted Rust boundary.

use crate::agent::ToolRequest;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Proposal {
    rationale: String,
    tool: String,
    arguments: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalAction {
    Tool(ToolRequest),
    Complete(String),
}

#[derive(Debug, Default)]
pub struct ProposalGuard {
    used_nonces: HashSet<String>,
}

fn exact(arguments: &HashMap<String, Value>, keys: &[&str]) -> bool {
    arguments.len() == keys.len() && keys.iter().all(|key| arguments.contains_key(*key))
}
fn string(arguments: &HashMap<String, Value>, key: &str) -> Result<String, String> {
    let value = arguments
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{key} must be a string"))?;
    if value.is_empty() || value.len() > 65_536 {
        return Err(format!("{key} has invalid size"));
    }
    Ok(value.into())
}

impl ProposalGuard {
    /// Converts untrusted model JSON into a bounded typed action.
    ///
    /// # Errors
    /// Returns a description for malformed, replayed, unknown, or oversized input.
    pub fn decode(
        &mut self,
        json: &str,
        session: &str,
        nonce: &str,
    ) -> Result<ProposalAction, String> {
        if json.len() > 262_144 {
            return Err("proposal is oversized".into());
        }
        let replay_key = format!("{session}:{nonce}");
        if session.is_empty() || nonce.len() < 8 || !self.used_nonces.insert(replay_key) {
            return Err("invalid or replayed proposal nonce".into());
        }
        let proposal: Proposal =
            serde_json::from_str(json).map_err(|error| format!("invalid proposal: {error}"))?;
        if proposal.rationale.len() > 16_384 {
            return Err("proposal rationale is oversized".into());
        }
        Ok(match proposal.tool.as_str() {
            "searchCode" if exact(&proposal.arguments, &["query"]) => {
                ProposalAction::Tool(ToolRequest::SearchCode {
                    query: string(&proposal.arguments, "query")?,
                    limit: 100,
                })
            }
            "readFile" if exact(&proposal.arguments, &["path"]) => {
                ProposalAction::Tool(ToolRequest::ReadFile {
                    path: string(&proposal.arguments, "path")?,
                })
            }
            "inspectGit" if exact(&proposal.arguments, &[]) => {
                ProposalAction::Tool(ToolRequest::InspectGit)
            }
            "runCommand" if exact(&proposal.arguments, &["command", "args"]) => {
                let command = string(&proposal.arguments, "command")?;
                let values = proposal
                    .arguments
                    .get("args")
                    .and_then(Value::as_array)
                    .ok_or("args must be an array")?;
                if values.len() > 128 {
                    return Err("too many command arguments".into());
                }
                let args = values
                    .iter()
                    .map(|value| {
                        value
                            .as_str()
                            .filter(|text| text.len() <= 4096)
                            .map(str::to_owned)
                            .ok_or_else(|| "command arguments must be bounded strings".to_owned())
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                ProposalAction::Tool(ToolRequest::RunCommand { command, args })
            }
            "applyPatch" if exact(&proposal.arguments, &["patch", "reverse"]) => {
                ProposalAction::Tool(ToolRequest::ApplyPatch {
                    patch: string(&proposal.arguments, "patch")?,
                    reverse: proposal
                        .arguments
                        .get("reverse")
                        .and_then(Value::as_bool)
                        .ok_or("reverse must be a boolean")?,
                })
            }
            "mcpCall" if exact(&proposal.arguments, &["server", "name", "arguments"]) => {
                let arguments = proposal
                    .arguments
                    .get("arguments")
                    .filter(|value| value.is_object())
                    .ok_or("MCP arguments must be an object")?
                    .clone();
                ProposalAction::Tool(ToolRequest::McpCall {
                    server: string(&proposal.arguments, "server")?,
                    name: string(&proposal.arguments, "name")?,
                    arguments,
                })
            }
            "complete" if exact(&proposal.arguments, &[]) => {
                ProposalAction::Complete(proposal.rationale)
            }
            _ => return Err("proposal tool or argument shape is invalid".into()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unknown_replay_and_bad_shapes() {
        let mut guard = ProposalGuard::default();
        let valid = r#"{"rationale":"inspect","tool":"inspectGit","arguments":{}}"#;
        assert!(matches!(
            guard.decode(valid, "1", "12345678"),
            Ok(ProposalAction::Tool(ToolRequest::InspectGit))
        ));
        assert!(guard.decode(valid, "1", "12345678").is_err());
        assert!(
            guard
                .decode(
                    r#"{"rationale":"x","tool":"inspectGit","arguments":{},"extra":1}"#,
                    "1",
                    "abcdefgh"
                )
                .is_err()
        );
        assert!(matches!(
            guard.decode(
                r#"{"rationale":"use local tool","tool":"mcpCall","arguments":{"server":"fixture","name":"read","arguments":{"path":"README.md"}}}"#,
                "1",
                "mcpnonce"
            ),
            Ok(ProposalAction::Tool(ToolRequest::McpCall { .. }))
        ));
    }
}
