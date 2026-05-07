use crate::stats::PersonalityStats;
use serde::{Deserialize, Serialize};

// Species table organized by rarity tier.
// Probability: Common 60%, Uncommon 25%, Rare 10%, Epic 4%, Legendary 1%.
pub const SPECIES_TABLE: &[SpeciesDef] = &[
    // Common (6 species)
    SpeciesDef {
        id: "void-cat",
        name: "Void Cat",
        rarity: RarityTier::Common,
    },
    SpeciesDef {
        id: "code-hound",
        name: "Code Hound",
        rarity: RarityTier::Common,
    },
    SpeciesDef {
        id: "terminal-turtle",
        name: "Terminal Turtle",
        rarity: RarityTier::Common,
    },
    SpeciesDef {
        id: "pixel-parrot",
        name: "Pixel Parrot",
        rarity: RarityTier::Common,
    },
    SpeciesDef {
        id: "debug-dragon",
        name: "Debug Dragon",
        rarity: RarityTier::Common,
    },
    SpeciesDef {
        id: "rust-fox",
        name: "Rust Fox",
        rarity: RarityTier::Common,
    },
    // Uncommon (4 species)
    SpeciesDef {
        id: "schema-spider",
        name: "Schema Spider",
        rarity: RarityTier::Uncommon,
    },
    SpeciesDef {
        id: "cache-crow",
        name: "Cache Crow",
        rarity: RarityTier::Uncommon,
    },
    SpeciesDef {
        id: "null-pointer-neko",
        name: "Null Pointer Neko",
        rarity: RarityTier::Uncommon,
    },
    SpeciesDef {
        id: "lambda-lizard",
        name: "Lambda Lizard",
        rarity: RarityTier::Uncommon,
    },
    // Rare (2 species)
    SpeciesDef {
        id: "recursion-raccoon",
        name: "Recursion Raccoon",
        rarity: RarityTier::Rare,
    },
    SpeciesDef {
        id: "stack-overflow-owl",
        name: "Stack Overflow Owl",
        rarity: RarityTier::Rare,
    },
    // Epic (2 species)
    SpeciesDef {
        id: "memory-leak-kraken",
        name: "Memory Leak Kraken",
        rarity: RarityTier::Epic,
    },
    SpeciesDef {
        id: "race-condition-chimera",
        name: "Race Condition Chimera",
        rarity: RarityTier::Epic,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum RarityTier {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl RarityTier {
    pub fn weight(&self) -> u32 {
        match self {
            RarityTier::Common => 60,
            RarityTier::Uncommon => 25,
            RarityTier::Rare => 10,
            RarityTier::Epic => 4,
            RarityTier::Legendary => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpeciesDef {
    pub id: &'static str,
    pub name: &'static str,
    pub rarity: RarityTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetState {
    pub species_id: String,
    pub species_name: String,
    pub pet_name: Option<String>,
    pub rarity_tier: RarityTier,
    pub is_shiny: bool,
    pub level: u32,
    pub xp: u32,
    pub stats: PersonalityStats,
    pub mood: String,
    pub muted: bool,
    pub last_interaction: Option<i64>,
    pub last_sleep: Option<i64>,
    pub created_at: i64,
}
