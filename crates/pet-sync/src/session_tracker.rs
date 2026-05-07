use super::opencode_state::{OpenCodeState, AgentState, StateChange};
use pet_engine::TaskSummary;

/// Tracks multiple OpenCode sessions and detects state changes.
#[derive(Debug, Clone)]
pub struct SessionTracker {
    sessions: std::collections::HashMap<String, OpenCodeState>,
    previous_states: std::collections::HashMap<String, AgentState>,
}

impl SessionTracker {
    pub fn new() -> Self {
        Self {
            sessions: std::collections::HashMap::new(),
            previous_states: std::collections::HashMap::new(),
        }
    }

    /// Get the aggregate state of all active sessions.
    pub fn aggregate_state(&self) -> (u32, u32, u32) {
        let mut running = 0;
        let mut waiting = 0;
        let mut idle = 0;

        for state in self.sessions.values() {
            match state.agent_state {
                AgentState::Running => running += 1,
                AgentState::Waiting => waiting += 1,
                AgentState::Idle => idle += 1,
                _ => {}
            }
        }

        (running, waiting, idle)
    }

    /// Update session state and return any state changes.
    pub fn update_session(
        &mut self,
        session_id: &str,
        new_state: OpenCodeState,
    ) -> Vec<StateChange> {
        let mut changes = Vec::new();

        // Store current state
        self.sessions.insert(session_id.to_string(), new_state.clone());

        // Check for state transitions
        if let Some(prev_state) = self.previous_states.get(session_id) {
            if *prev_state != new_state.agent_state {
                changes.push(StateChange::AgentStateChanged {
                    from: prev_state.clone(),
                    to: new_state.agent_state.clone(),
                });

                // Running → Idle transition means task completion
                if matches!(prev_state, AgentState::Running)
                   && matches!(new_state.agent_state, AgentState::Idle)
                {
                    let task_name = new_state.current_task.clone()
                        .unwrap_or_else(|| "unknown_task".to_string());

                    if new_state.error_count > 0 {
                        changes.push(StateChange::Error {
                            message: format!(
                                "{} errors in {}",
                                new_state.error_count, task_name
                            ),
                        });
                    }

                    changes.push(StateChange::TaskCompleted {
                        summary: TaskSummary::new(
                            &task_name,
                            new_state.error_count == 0,
                            new_state.error_count,
                        ),
                    });
                }
            }
        }

        self.previous_states
            .insert(session_id.to_string(), new_state.agent_state.clone());

        changes
    }

    /// Remove a session
    pub fn remove_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
        self.previous_states.remove(session_id);
    }

    /// Get all active sessions
    pub fn sessions(&self) -> &std::collections::HashMap<String, OpenCodeState> {
        &self.sessions
    }

    /// Number of active sessions
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}
