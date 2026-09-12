//! Auditable deterministic agent session state and tool contracts.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_WALL_TIME_MS: u128 = 30 * 60 * 1_000;

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentPhase {
    Planning,
    Gathering,
    Executing,
    Verifying,
    AwaitingApproval,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ToolRequest {
    SearchCode {
        query: String,
        limit: usize,
    },
    ReadFile {
        path: String,
    },
    InspectGit,
    RunCommand {
        command: String,
        args: Vec<String>,
    },
    ApplyPatch {
        patch: String,
        reverse: bool,
    },
    McpCall {
        server: String,
        name: String,
        arguments: Value,
    },
}

impl ToolRequest {
    #[must_use]
    pub const fn requires_approval(&self) -> bool {
        matches!(
            self,
            Self::RunCommand { .. } | Self::ApplyPatch { .. } | Self::McpCall { .. }
        )
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
    #[serde(default = "now_ms")]
    pub started_at_ms: u128,
    #[serde(default)]
    pub failures: usize,
    #[serde(default)]
    pub generated_bytes: usize,
    #[serde(default)]
    pub action_fingerprints: Vec<String>,
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
            started_at_ms: now_ms(),
            failures: 0,
            generated_bytes: 0,
            action_fingerprints: Vec::new(),
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
            | AgentPhase::Paused
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

    /// Accepts a planner-selected typed action for execution.
    ///
    /// # Errors
    /// Returns an error for terminal, paused, busy, or exhausted sessions.
    pub fn propose(&mut self, request: ToolRequest, maximum_steps: usize) -> Result<(), String> {
        if matches!(
            self.phase,
            AgentPhase::Completed | AgentPhase::Failed | AgentPhase::Cancelled
        ) {
            return Err("session is terminal".into());
        }
        if self.phase == AgentPhase::Paused {
            return Err("session is paused".into());
        }
        if self.pending_tool.is_some() {
            return Err("session already has a pending action".into());
        }
        if now_ms().saturating_sub(self.started_at_ms) > MAX_WALL_TIME_MS {
            self.phase = AgentPhase::Failed;
            return Err("agent wall-time budget exhausted".into());
        }
        if self.step >= maximum_steps {
            self.phase = AgentPhase::Failed;
            return Err("agent step budget exhausted".into());
        }
        let fingerprint = serde_json::to_string(&request).map_err(|error| error.to_string())?;
        if self.action_fingerprints.contains(&fingerprint) {
            return Err("agent repeated an identical action".into());
        }
        self.action_fingerprints.push(fingerprint);
        self.step += 1;
        self.phase = if request.requires_approval() {
            AgentPhase::AwaitingApproval
        } else {
            AgentPhase::Executing
        };
        self.pending_tool = Some(request);
        Ok(())
    }

    /// Records a tool result and returns control to the planner.
    ///
    /// # Errors
    /// Returns an error for mismatched or absent pending actions.
    pub fn observe(&mut self, observation: &Observation) -> Result<(), String> {
        if self.pending_tool.is_none() || observation.step != self.step {
            return Err("observation does not match a pending action".into());
        }
        if now_ms().saturating_sub(self.started_at_ms) > MAX_WALL_TIME_MS {
            self.pending_tool = None;
            self.phase = AgentPhase::Failed;
            return Err("agent wall-time budget exhausted".into());
        }
        self.pending_tool = None;
        self.observations.push(observation.clone());
        self.generated_bytes = self
            .generated_bytes
            .saturating_add(observation.summary.len());
        if self.generated_bytes > 256_000 {
            self.phase = AgentPhase::Failed;
            return Err("agent output budget exhausted".into());
        }
        if !observation.success {
            self.failures += 1;
        }
        self.phase = if observation.success || self.failures < 3 {
            AgentPhase::Planning
        } else {
            AgentPhase::Failed
        };
        Ok(())
    }

    pub fn pause(&mut self) {
        if !matches!(
            self.phase,
            AgentPhase::Completed | AgentPhase::Failed | AgentPhase::Cancelled
        ) {
            self.phase = AgentPhase::Paused;
        }
    }
    pub fn resume(&mut self) {
        if self.phase == AgentPhase::Paused {
            self.phase = if self
                .pending_tool
                .as_ref()
                .is_some_and(ToolRequest::requires_approval)
            {
                AgentPhase::AwaitingApproval
            } else {
                AgentPhase::Planning
            };
        }
    }
    pub fn complete(&mut self, summary: String) {
        self.pending_tool = None;
        self.observations.push(Observation {
            step: self.step,
            summary,
            success: true,
        });
        self.phase = AgentPhase::Completed;
    }

    /// Starts a fresh bounded attempt while preserving the prior observations.
    ///
    /// # Errors
    /// Returns an error unless the previous attempt reached the failed state.
    pub fn retry(&mut self) -> Result<(), String> {
        if self.phase != AgentPhase::Failed {
            return Err("only a failed session can be retried".into());
        }
        self.phase = AgentPhase::Planning;
        self.pending_tool = None;
        self.failures = 0;
        self.generated_bytes = 0;
        self.action_fingerprints.clear();
        self.started_at_ms = now_ms();
        Ok(())
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
    use super::{AgentPhase, AgentSession, MAX_WALL_TIME_MS, Observation, ToolRequest, now_ms};
    use std::fs;
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
    #[test]
    fn model_driven_loop_pauses_observes_and_completes() {
        let mut session = AgentSession::new(8, "repair bug".into());
        session
            .propose(
                ToolRequest::ReadFile {
                    path: "src/lib.rs".into(),
                },
                24,
            )
            .unwrap();
        assert_eq!(session.phase, AgentPhase::Executing);
        session
            .observe(&Observation {
                step: 1,
                summary: "read source".into(),
                success: true,
            })
            .unwrap();
        assert_eq!(session.phase, AgentPhase::Planning);
        session
            .propose(
                ToolRequest::ApplyPatch {
                    patch: "patch".into(),
                    reverse: false,
                },
                24,
            )
            .unwrap();
        assert_eq!(session.phase, AgentPhase::AwaitingApproval);
        session.pause();
        assert_eq!(session.phase, AgentPhase::Paused);
        session.resume();
        assert_eq!(session.phase, AgentPhase::AwaitingApproval);
        session.complete("task complete".into());
        assert_eq!(session.phase, AgentPhase::Completed);
    }

    #[test]
    fn rejects_sessions_over_wall_time_budget() {
        let mut session = AgentSession::new(9, "long task".into());
        session.started_at_ms = now_ms().saturating_sub(MAX_WALL_TIME_MS + 1);
        assert!(
            session
                .propose(ToolRequest::InspectGit, 24)
                .unwrap_err()
                .contains("wall-time")
        );
        assert_eq!(session.phase, AgentPhase::Failed);
        session.retry().unwrap();
        assert_eq!(session.phase, AgentPhase::Planning);
        assert!(session.propose(ToolRequest::InspectGit, 24).is_ok());
    }

    #[test]
    fn executes_read_patch_validate_complete_and_rollback_flow() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(directory.path().join("app.py"), "print('old')\n").unwrap();
        let mut session = AgentSession::new(10, "change old to new".into());

        session
            .propose(
                ToolRequest::ReadFile {
                    path: "app.py".into(),
                },
                24,
            )
            .unwrap();
        let source = fs::read_to_string(directory.path().join("app.py")).unwrap();
        session
            .observe(&Observation {
                step: 1,
                summary: source,
                success: true,
            })
            .unwrap();

        let patch = "--- a/app.py\n+++ b/app.py\n@@ -1 +1 @@\n-print('old')\n+print('new')\n";
        session
            .propose(
                ToolRequest::ApplyPatch {
                    patch: patch.into(),
                    reverse: false,
                },
                24,
            )
            .unwrap();
        crate::system::apply_transactional_patch(directory.path(), patch, false).unwrap();
        session
            .observe(&Observation {
                step: 2,
                summary: "patch applied".into(),
                success: true,
            })
            .unwrap();

        let args = vec![
            "-c".into(),
            "from pathlib import Path; assert \"new\" in Path(\"app.py\").read_text()".into(),
        ];
        session
            .propose(
                ToolRequest::RunCommand {
                    command: "python3".into(),
                    args: args.clone(),
                },
                24,
            )
            .unwrap();
        let validation = crate::system::run(directory.path(), "python3", &args, true).unwrap();
        session
            .observe(&Observation {
                step: 3,
                summary: validation.stdout,
                success: validation.exit_code == Some(0),
            })
            .unwrap();
        session.complete("changed and validated app.py".into());
        assert_eq!(session.phase, AgentPhase::Completed);

        crate::system::rollback_last_patch(directory.path()).unwrap();
        assert_eq!(
            fs::read_to_string(directory.path().join("app.py")).unwrap(),
            "print('old')\n"
        );
    }
}
