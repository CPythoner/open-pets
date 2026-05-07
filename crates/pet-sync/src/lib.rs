//! pet-sync - OpenCode state monitoring and pet reaction pipeline
//!
//! Monitors OpenCode session state and triggers contextual pet reactions.

mod opencode_state;
pub mod session_tracker;
mod config;
pub mod reaction_pipeline;

pub use opencode_state::{AgentState, StateChange, OpenCodeState};
pub use session_tracker::SessionTracker;
pub use config::SyncConfig;
pub use reaction_pipeline::ReactionPipeline;
