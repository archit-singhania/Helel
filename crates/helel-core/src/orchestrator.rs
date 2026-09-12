//! Bounded agent orchestration independent of any particular local model.

use crate::agent::ToolRequest;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentBudget {
    pub max_steps: usize,
    pub max_failures: usize,
    pub max_generated_bytes: usize,
}
impl Default for AgentBudget {
    fn default() -> Self {
        Self {
            max_steps: 24,
            max_failures: 3,
            max_generated_bytes: 256_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopReason {
    Completed,
    AwaitingApproval,
    StepLimit,
    FailureLimit,
    OutputLimit,
    RepeatedAction,
    Cancelled,
}
#[derive(Debug, Clone, Default)]
pub struct LoopState {
    pub steps: usize,
    pub failures: usize,
    pub generated_bytes: usize,
    pub cancelled: bool,
    seen: HashSet<String>,
}
impl LoopState {
    /// Accounts for an action before continuing the loop.
    ///
    /// # Errors
    /// Returns the precise stop reason when any budget or safety gate is reached.
    pub fn accept(
        &mut self,
        request: &ToolRequest,
        output_bytes: usize,
        success: bool,
        budget: &AgentBudget,
    ) -> Result<(), StopReason> {
        if self.cancelled {
            return Err(StopReason::Cancelled);
        }
        if self.steps >= budget.max_steps {
            return Err(StopReason::StepLimit);
        }
        if self.generated_bytes.saturating_add(output_bytes) > budget.max_generated_bytes {
            return Err(StopReason::OutputLimit);
        }
        let fingerprint = serde_json::to_string(request).unwrap_or_default();
        if !self.seen.insert(fingerprint) {
            return Err(StopReason::RepeatedAction);
        }
        self.steps += 1;
        self.generated_bytes += output_bytes;
        if !success {
            self.failures += 1;
            if self.failures >= budget.max_failures {
                return Err(StopReason::FailureLimit);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stops_repeated_actions_and_limits() {
        let mut s = LoopState::default();
        let b = AgentBudget::default();
        let r = ToolRequest::InspectGit;
        assert!(s.accept(&r, 10, true, &b).is_ok());
        assert_eq!(s.accept(&r, 10, true, &b), Err(StopReason::RepeatedAction));
    }
}
