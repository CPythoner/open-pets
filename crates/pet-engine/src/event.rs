use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    LevelUp,
    MoodChange,
    Reaction,
    Petted,
    Milestone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_type: EventType,
    pub description: String,
    pub xp_earned: u32,
    pub details: serde_json::Value,
}

impl Event {
    pub fn level_up(from: u32, to: u32) -> Self {
        let xp_milestone = to * 10;
        Self {
            event_type: EventType::LevelUp,
            description: format!("Leveled up from {} to {}! (+{} bonus XP)", from, to, xp_milestone),
            xp_earned: xp_milestone,
            details: serde_json::json!({"from": from, "to": to}),
        }
    }

    pub fn mood_changed(from: &str, to: &str) -> Self {
        Self {
            event_type: EventType::MoodChange,
            description: format!("Mood changed from {} to {}", from, to),
            xp_earned: 0,
            details: serde_json::json!({"from": from, "to": to}),
        }
    }

    pub fn petted(xp: u32) -> Self {
        Self {
            event_type: EventType::Petted,
            description: "Pet interaction".to_string(),
            xp_earned: xp,
            details: serde_json::json!({"source": "petting"}),
        }
    }
}
