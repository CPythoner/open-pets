use pet_engine::Reaction;
use pet_engine::{generate_reaction, PetState, ReactionMode, TaskSummary};

/// The reaction pipeline connects OpenCode state changes to pet reactions.
#[derive(Debug, Clone)]
pub struct ReactionPipeline {
    mode: ReactionMode,
    cooldown_secs: u64,
    last_reaction: Option<chrono::DateTime<chrono::Utc>>,
}

impl ReactionPipeline {
    pub fn new(mode: ReactionMode, cooldown_secs: u64) -> Self {
        Self {
            mode,
            cooldown_secs,
            last_reaction: None,
        }
    }

    pub fn from_config(mode: ReactionMode) -> Self {
        // Default 30 second cooldown between reactions
        Self::new(mode, 30)
    }

    /// Process a state change and optionally return a pet reaction.
    pub fn process(&mut self, pet: &mut PetState, change: super::StateChange) -> Option<Reaction> {
        match &change {
            super::StateChange::TaskCompleted { summary } => {
                pet_engine::Engine::award_xp(pet, 50, &format!("task:{}", summary.task_name));
                pet_engine::Engine::update_mood(pet, summary.success, summary.error_count);

                if self.is_on_cooldown() {
                    return None;
                }

                let reaction = generate_reaction(pet, summary, &self.mode);
                if reaction.is_some() {
                    self.last_reaction = Some(chrono::Utc::now());
                }
                reaction
            }
            super::StateChange::AgentStateChanged { .. } => {
                // State changes don't trigger reactions, only visual state updates
                None
            }
            super::StateChange::Error { message } => {
                pet_engine::Engine::award_xp(pet, 10, &format!("error:{}", message));
                pet_engine::Engine::update_mood(pet, false, 1);

                if self.is_on_cooldown() {
                    return None;
                }

                let summary = TaskSummary::new(&format!("error: {}", message), false, 1);
                let reaction = generate_reaction(pet, &summary, &self.mode);
                if reaction.is_some() {
                    self.last_reaction = Some(chrono::Utc::now());
                }
                reaction
            }
        }
    }

    fn is_on_cooldown(&self) -> bool {
        if let Some(last) = self.last_reaction {
            let elapsed = chrono::Utc::now().signed_duration_since(last);
            elapsed.num_seconds() < self.cooldown_secs as i64
        } else {
            false
        }
    }

    pub fn set_mode(&mut self, mode: ReactionMode) {
        self.mode = mode;
    }

    pub fn mode(&self) -> &ReactionMode {
        &self.mode
    }
}
