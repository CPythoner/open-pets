use crate::event::Event;
use crate::species::PetState;
use serde::{Deserialize, Serialize};

/// Calculate XP needed to reach the next level from the current level.
/// Formula: level * 100 + level^2 * 50
pub fn xp_to_next_level(level: u32) -> u32 {
    level * 100 + level.pow(2) * 50
}

/// Calculate cumulative XP required to reach a specific level from level 1.
pub fn total_xp_for_level(level: u32) -> u32 {
    if level <= 1 {
        return 0;
    }
    (1..level).map(|l| xp_to_next_level(l)).sum()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelInfo {
    pub current_level: u32,
    pub current_xp: u32,
    pub xp_for_next: u32,
    pub xp_in_level: u32,
}

/// Award XP to a pet state and return any triggered events.
pub fn award_xp(state: &mut PetState, amount: u32, _reason: &str) -> Vec<Event> {
    let _old_level = state.level;
    state.xp += amount;

    let mut events = Vec::new();

    // Check for level-ups (handle multiple level-ups in one award)
    loop {
        let xp_needed = total_xp_for_level(state.level + 1);
        if state.xp >= xp_needed {
            let from_level = state.level;
            state.level += 1;
            events.push(Event::level_up(from_level, state.level));
        } else {
            break;
        }
    }

    events
}
