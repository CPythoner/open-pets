use std::path::Path;

/// Represents the current state of an OpenCode agent
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

impl AgentState {
    pub fn from_label(label: &str) -> Self {
        match label.to_ascii_lowercase().as_str() {
            "running" | "working" | "busy" => AgentState::Running,
            "waiting" | "blocked" => AgentState::Waiting,
            "thinking" | "planning" => AgentState::Thinking,
            "error" | "failed" | "failure" => AgentState::Error,
            _ => AgentState::Idle,
        }
    }
}

/// Configuration for the sync layer
/// (Defined in config.rs as `config::SyncConfig`)

/// A change event in the OpenCode session state
#[derive(Debug, Clone)]
pub enum StateChange {
    /// Agent has changed its operational state
    AgentStateChanged { from: AgentState, to: AgentState },
    /// A task has been completed
    TaskCompleted { summary: pet_engine::TaskSummary },
    /// An error occurred during execution
    Error { message: String },
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

    pub fn from_json_str(content: &str) -> Result<Self, serde_json::Error> {
        let raw: RawOpenCodeState = serde_json::from_str(content)?;
        Ok(raw.into_state())
    }

    pub fn from_state_file(path: impl AsRef<Path>) -> Result<Option<Self>, String> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        Self::from_json_str(&content)
            .map(Some)
            .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
    }
}

#[derive(Debug, serde::Deserialize)]
struct RawOpenCodeState {
    session_id: Option<String>,
    agent_state: Option<String>,
    state: Option<String>,
    status: Option<String>,
    current_task: Option<String>,
    task: Option<String>,
    error_count: Option<u32>,
    files_changed: Option<Vec<String>>,
    last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

impl RawOpenCodeState {
    fn into_state(self) -> OpenCodeState {
        let state_label = self
            .agent_state
            .or(self.state)
            .or(self.status)
            .unwrap_or_else(|| "idle".to_string());

        OpenCodeState {
            session_id: self.session_id,
            agent_state: AgentState::from_label(&state_label),
            current_task: self.current_task.or(self.task),
            error_count: self.error_count.unwrap_or(0),
            files_changed: self.files_changed.unwrap_or_default(),
            last_updated: self.last_updated.unwrap_or_else(chrono::Utc::now),
        }
    }
}
