//! pet-sync - OpenCode state monitoring and pet reaction pipeline
//!
//! Monitors OpenCode session state and triggers contextual pet reactions.

mod config;
mod opencode_state;
pub mod reaction_pipeline;
pub mod session_tracker;

pub use config::SyncConfig;
pub use opencode_state::{AgentState, OpenCodeState, StateChange};
pub use reaction_pipeline::ReactionPipeline;
pub use session_tracker::SessionTracker;
