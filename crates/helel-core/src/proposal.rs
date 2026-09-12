//! Strict model-proposal decoder at the trusted Rust boundary.

use crate::agent::ToolRequest;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Proposal {
    pub version: u32,
    pub session_id: String,
    pub nonce: String,
    pub tool: ProposalTool,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum ProposalTool {
    SearchCode { query: String },
    ReadFile { path: String },
    InspectGit,
    RunCommand { program: String, args: Vec<String> },
    ApplyPatch { patch: String, reverse: bool },
}

#[derive(Debug, Default)]
pub struct ProposalGuard {
    used_nonces: HashSet<String>,
}
impl ProposalGuard {
    /// Converts untrusted JSON into a bounded typed request.
    ///
    /// # Errors
    /// Returns a description for malformed, stale, replayed, or oversized input.
    pub fn decode(&mut self, json: &str, expected_session: &str) -> Result<ToolRequest, String> {
        if json.len() > 262_144 {
            return Err("proposal is oversized".into());
        }
        let p: Proposal =
            serde_json::from_str(json).map_err(|e| format!("invalid proposal: {e}"))?;
        if p.version != 1 || p.session_id != expected_session {
            return Err("stale or incompatible proposal".into());
        }
        if p.nonce.len() < 8 || !self.used_nonces.insert(p.nonce) {
            return Err("invalid or replayed nonce".into());
        }
        let bounded = |value: &str| {
            if value.is_empty() || value.len() > 65_536 {
                Err("invalid argument size".to_string())
            } else {
                Ok(())
            }
        };
        Ok(match p.tool {
            ProposalTool::SearchCode { query } => {
                bounded(&query)?;
                ToolRequest::SearchCode { query, limit: 100 }
            }
            ProposalTool::ReadFile { path } => {
                bounded(&path)?;
                ToolRequest::ReadFile { path }
            }
            ProposalTool::InspectGit => ToolRequest::InspectGit,
            ProposalTool::RunCommand { program, args } => {
                bounded(&program)?;
                if args.len() > 128 || args.iter().any(|a| a.len() > 4096) {
                    return Err("invalid command arguments".into());
                }
                ToolRequest::RunCommand {
                    command: program,
                    args,
                }
            }
            ProposalTool::ApplyPatch { patch, reverse } => {
                bounded(&patch)?;
                ToolRequest::ApplyPatch { patch, reverse }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unknown_replay_and_stale() {
        let mut g = ProposalGuard::default();
        let p = r#"{"version":1,"sessionId":"s","nonce":"12345678","tool":{"kind":"inspectGit"}}"#;
        assert!(g.decode(p, "s").is_ok());
        assert!(g.decode(p, "s").is_err());
        let bad = r#"{"version":1,"sessionId":"s","nonce":"abcdefgh","extra":1,"tool":{"kind":"inspectGit"}}"#;
        assert!(g.decode(bad, "s").is_err());
    }
}
