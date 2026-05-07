/// Represents the current state of an OpenCode agent
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentState {
    /// No active task, waiting for input
    Idle,
    /// Agent is actively working on a task
    Running,
    /// Agent is waiting for user input or tool response
    Waiting,
    /// Agent is in a planning/thinking phase
    Thinking,
    /// An error has occurred
    Error,
}

/// Configuration for the sync layer
/// (Defined in config.rs as `config::SyncConfig`)

/// A change event in the OpenCode session state
#[derive(Debug, Clone)]
pub enum StateChange {
    /// Agent has changed its operational state
    AgentStateChanged {
        from: AgentState,
        to: AgentState,
    },
    /// A task has been completed
    TaskCompleted {
        summary: pet_engine::TaskSummary,
    },
    /// An error occurred during execution
    Error {
        message: String,
    },
}

/// Snapshot of an OpenCode session state
#[derive(Debug, Clone)]
pub struct OpenCodeState {
    pub session_id: Option<String>,
    pub agent_state: AgentState,
    pub current_task: Option<String>,
    pub error_count: u32,
    pub files_changed: Vec<String>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl OpenCodeState {
    pub fn idle() -> Self {
        Self {
            session_id: None,
            agent_state: AgentState::Idle,
            current_task: None,
            error_count: 0,
            files_changed: Vec::new(),
            last_updated: chrono::Utc::now(),
        }
    }
}
