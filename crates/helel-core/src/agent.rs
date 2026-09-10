//! Auditable deterministic agent session state and tool contracts.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentPhase {
    Planning,
    Gathering,
    Executing,
    Verifying,
    AwaitingApproval,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ToolRequest {
    SearchCode { query: String, limit: usize },
    ReadFile { path: String },
    InspectGit,
    RunCommand { command: String, args: Vec<String> },
    ApplyPatch { patch: String, reverse: bool },
}

impl ToolRequest {
    #[must_use]
    pub const fn requires_approval(&self) -> bool {
        matches!(self, Self::RunCommand { .. } | Self::ApplyPatch { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    pub step: usize,
    pub summary: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSession {
    pub id: u64,
    pub objective: String,
    pub phase: AgentPhase,
    pub plan: Vec<String>,
    pub pending_tool: Option<ToolRequest>,
    pub observations: Vec<Observation>,
    pub step: usize,
}

impl AgentSession {
    #[must_use]
    pub fn new(id: u64, objective: String) -> Self {
        Self {
            id,
            objective,
            phase: AgentPhase::Planning,
            plan: vec![
                "Understand the request and retrieve repository context".into(),
                "Produce a bounded proposed change".into(),
                "Run deterministic verification".into(),
            ],
            pending_tool: None,
            observations: Vec::new(),
            step: 0,
        }
    }

    /// Advances the state machine and returns the next typed tool request.
    ///
    /// # Errors
    /// Returns an error for terminal sessions, missing approval, or a failed observation.
    pub fn advance(
        &mut self,
        observation: Option<Observation>,
        approved: bool,
    ) -> Result<Option<ToolRequest>, String> {
        if matches!(
            self.phase,
            AgentPhase::Completed | AgentPhase::Failed | AgentPhase::Cancelled
        ) {
            return Err("session is already terminal".into());
        }
        if let Some(item) = observation {
            if item.step != self.step {
                return Err("observation step does not match the pending action".into());
            }
            if !item.success {
                self.phase = AgentPhase::Failed;
                self.observations.push(item);
                return Ok(None);
            }
            self.observations.push(item);
            self.pending_tool = None;
        }
        if self.phase == AgentPhase::AwaitingApproval {
            if !approved {
                return Err("approval is required for the pending tool".into());
            }
            self.phase = AgentPhase::Verifying;
            return Ok(self.pending_tool.clone());
        }
        self.step += 1;
        let request = match self.phase {
            AgentPhase::Planning => {
                self.phase = AgentPhase::Gathering;
                ToolRequest::SearchCode {
                    query: self.objective.clone(),
                    limit: 12,
                }
            }
            AgentPhase::Gathering => {
                self.phase = AgentPhase::Executing;
                ToolRequest::InspectGit
            }
            AgentPhase::Executing => {
                self.phase = AgentPhase::Verifying;
                ToolRequest::RunCommand {
                    command: "cargo".into(),
                    args: vec!["test".into(), "--workspace".into()],
                }
            }
            AgentPhase::Verifying => {
                self.phase = AgentPhase::Completed;
                return Ok(None);
            }
            AgentPhase::AwaitingApproval
            | AgentPhase::Completed
            | AgentPhase::Failed
            | AgentPhase::Cancelled => unreachable!(),
        };
        self.pending_tool = Some(request.clone());
        if request.requires_approval() {
            self.phase = AgentPhase::AwaitingApproval;
        }
        Ok(Some(request))
    }

    pub fn cancel(&mut self) {
        if !matches!(self.phase, AgentPhase::Completed | AgentPhase::Failed) {
            self.phase = AgentPhase::Cancelled;
            self.pending_tool = None;
        }
    }
}

/// Persists the complete session ledger atomically inside the workspace.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn save_sessions(root: &Path, sessions: &[AgentSession]) -> io::Result<()> {
    let directory = root.join(".helel");
    fs::create_dir_all(&directory)?;
    let temporary = directory.join("agents.json.tmp");
    fs::write(
        &temporary,
        serde_json::to_vec(sessions).map_err(io::Error::other)?,
    )?;
    fs::rename(temporary, directory.join("agents.json"))
}

/// Loads the workspace's session ledger, returning an empty ledger when absent.
///
/// # Errors
/// Returns an error when an existing ledger is unreadable or invalid.
pub fn load_sessions(root: &Path) -> io::Result<Vec<AgentSession>> {
    let path = root.join(".helel/agents.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    serde_json::from_slice(&fs::read(path)?).map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::{AgentPhase, AgentSession, Observation, ToolRequest};
    #[test]
    fn follows_deterministic_state_and_approval_gate() {
        let mut session = AgentSession::new(1, "find greeting".into());
        assert!(matches!(
            session.advance(None, false).unwrap(),
            Some(ToolRequest::SearchCode { .. })
        ));
        let first = Observation {
            step: 1,
            summary: "context found".into(),
            success: true,
        };
        assert_eq!(
            session.advance(Some(first), false).unwrap(),
            Some(ToolRequest::InspectGit)
        );
        let second = Observation {
            step: 2,
            summary: "tree clean".into(),
            success: true,
        };
        assert!(matches!(
            session.advance(Some(second), false).unwrap(),
            Some(ToolRequest::RunCommand { .. })
        ));
        assert_eq!(session.phase, AgentPhase::AwaitingApproval);
        assert!(session.advance(None, false).is_err());
        assert!(matches!(
            session.advance(None, true).unwrap(),
            Some(ToolRequest::RunCommand { .. })
        ));
        let third = Observation {
            step: 3,
            summary: "checks passed".into(),
            success: true,
        };
        assert_eq!(session.advance(Some(third), false).unwrap(), None);
        assert_eq!(session.phase, AgentPhase::Completed);
    }
    #[test]
    fn failure_and_cancel_are_terminal() {
        let mut session = AgentSession::new(2, "task".into());
        session.advance(None, false).unwrap();
        session
            .advance(
                Some(Observation {
                    step: 1,
                    summary: "failed".into(),
                    success: false,
                }),
                false,
            )
            .unwrap();
        assert_eq!(session.phase, AgentPhase::Failed);
        assert!(session.advance(None, false).is_err());
        let mut cancelled = AgentSession::new(3, "task".into());
        cancelled.cancel();
        assert_eq!(cancelled.phase, AgentPhase::Cancelled);
    }
    #[test]
    fn session_ledger_round_trips() {
        let directory = tempfile::tempdir().unwrap();
        let sessions = vec![AgentSession::new(7, "persist me".into())];
        super::save_sessions(directory.path(), &sessions).unwrap();
        let loaded = super::load_sessions(directory.path()).unwrap();
        assert_eq!(loaded[0].id, 7);
        assert_eq!(loaded[0].objective, "persist me");
    }
}
