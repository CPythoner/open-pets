//! Core pet engine logic for open-pets - pure functions, no I/O.

mod event;
mod memory;
mod mood;
mod reaction;
mod species;
mod stats;
mod xp;

pub use event::{Event, EventType};
pub use memory::{Memory, MemoryCategory, MemoryStore};
pub use mood::{Mood, MoodState};
pub use reaction::{
    generate_reaction, backseat_error_reaction, backseat_success_reaction,
    encouragement_reaction, Reaction, ReactionMode, TaskSummary,
};
pub use species::{PetState, RarityTier, SpeciesDef, SPECIES_TABLE};
pub use stats::PersonalityStats;
pub use xp::{xp_to_next_level, total_xp_for_level, LevelInfo};

/// The main entry point for pet engine operations.
pub struct Engine;

impl Engine {
    /// Hatch a new pet from a seed string.
    /// Deterministic: same seed always produces the same pet.
    pub fn hatch(seed: &str) -> PetState {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.update(seed.as_bytes());
        let hash = hasher.finalize();
        let hash_bytes: Vec<u8> = hash.to_vec();

        // Determine rarity tier first
        let rarity_roll = u32::from_be_bytes([
            hash_bytes[0], hash_bytes[1], hash_bytes[2], hash_bytes[3],
        ]) % 100;

        let target_rarity = if rarity_roll < 60 {
            RarityTier::Common
        } else if rarity_roll < 85 {
            RarityTier::Uncommon
        } else if rarity_roll < 95 {
            RarityTier::Rare
        } else if rarity_roll < 99 {
            RarityTier::Epic
        } else {
            RarityTier::Legendary
        };

        // Filter species by rarity and pick one
        let matching: Vec<&SpeciesDef> = SPECIES_TABLE
            .iter()
            .filter(|s| s.rarity == target_rarity)
            .collect();

        let species_idx = if matching.is_empty() {
            // Fallback: pick from common tier
            let common: Vec<&SpeciesDef> = SPECIES_TABLE
                .iter()
                .filter(|s| s.rarity == RarityTier::Common)
                .collect();
            let idx = u32::from_be_bytes([
                hash_bytes[4], hash_bytes[5], hash_bytes[6], hash_bytes[7],
            ]) as usize
                % common.len();
            common[idx]
        } else {
            let idx = u32::from_be_bytes([
                hash_bytes[4], hash_bytes[5], hash_bytes[6], hash_bytes[7],
            ]) as usize
                % matching.len();
            matching[idx]
        };

        // Check for shiny (1% chance)
        let shiny_roll = u32::from_be_bytes([
            hash_bytes[8], hash_bytes[9], hash_bytes[10], hash_bytes[11],
        ]) % 100;
        let is_shiny = shiny_roll == 0;

        // Derive stats from seed
        let stats = PersonalityStats::derive_from_seed(&hash_bytes[12..]);

        let now = chrono::Utc::now().timestamp();

        PetState {
            species_id: species_idx.id.to_string(),
            species_name: species_idx.name.to_string(),
            pet_name: None,
            rarity_tier: species_idx.rarity,
            is_shiny,
            level: 1,
            xp: 0,
            stats,
            mood: "neutral".to_string(),
            muted: false,
            last_interaction: None,
            last_sleep: None,
            created_at: now,
        }
    }

    /// Calculate the current mood from pet state.
    pub fn calculate_mood(state: &PetState) -> MoodState {
        Mood::calculate(state)
    }

    /// Award XP and return any events (level-ups, etc.).
    pub fn award_xp(state: &mut PetState, amount: u32, reason: &str) -> Vec<Event> {
        xp::award_xp(state, amount, reason)
    }

    /// Generate a reaction for a completed task.
    pub fn generate_reaction(state: &PetState, task: &TaskSummary, mode: ReactionMode) -> Option<Reaction> {
        reaction::generate_reaction(state, task, &mode)
    }

    /// Update mood after an interaction.
    pub fn update_mood(state: &mut PetState, success: bool, error_count: u32) {
        mood::Mood::update_after_interaction(state, success, error_count);
    }
}
