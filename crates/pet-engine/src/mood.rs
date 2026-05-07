use crate::species::PetState;
use chrono::Utc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoodState {
    Happy,
    Content,
    Neutral,
    Curious,
    Grumpy,
}

impl MoodState {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "happy" => MoodState::Happy,
            "content" => MoodState::Content,
            "curious" => MoodState::Curious,
            "grumpy" => MoodState::Grumpy,
            _ => MoodState::Neutral,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            MoodState::Happy => "happy",
            MoodState::Content => "content",
            MoodState::Neutral => "neutral",
            MoodState::Curious => "curious",
            MoodState::Grumpy => "grumpy",
        }
    }

    pub fn is_positive(&self) -> bool {
        matches!(
            self,
            MoodState::Happy | MoodState::Content | MoodState::Curious
        )
    }
}

pub struct Mood;

impl Mood {
    pub fn new(mood_str: &str) -> MoodState {
        MoodState::from_str(mood_str)
    }

    /// Calculate current mood from pet state.
    pub fn calculate(state: &PetState) -> MoodState {
        let now = Utc::now().timestamp() as u64;
        let last_interaction = state.last_interaction.unwrap_or(0) as u64;

        let hours_since_interaction = if last_interaction > 0 {
            (now.saturating_sub(last_interaction)) / 3600
        } else {
            999
        };

        // Mood decay: over time without interaction, drift toward neutral
        if hours_since_interaction > 48 {
            return MoodState::Neutral;
        }

        // High chaos → more prone to content/grumpy swings
        if state.stats.chaos > 15 {
            return MoodState::Content;
        }

        MoodState::from_str(&state.mood)
    }

    pub fn update_after_interaction(state: &mut PetState, success: bool, error_count: u32) {
        let now = Utc::now().timestamp();
        state.last_interaction = Some(now);

        if success {
            if state.mood == "neutral" || state.mood == "grumpy" {
                state.mood = "content".to_string();
            } else if state.mood == "content" {
                state.mood = "happy".to_string();
            }
        } else if error_count > 2 {
            state.mood = "grumpy".to_string();
        } else if error_count > 0 {
            state.mood = "curious".to_string();
        }
    }
}
