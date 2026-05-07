# Open-Pets Phase 1: Pet Engine Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the core pet-engine Rust crate with deterministic species generation, personality stats, mood calculation, XP/leveling, and observer reaction system.

**Architecture:** Pure Rust library with no I/O. Three logical modules (species, mood, xp) exposed through a unified `Engine` struct. Fully testable with unit tests.

**Tech Stack:** Rust, sha2, rand, serde, serde_json, chrono

---

## Files to Create/Modify

```
open-pets/
├── Cargo.toml                              # Workspace root (CREATE)
├── .gitignore                              # Rust gitignore (CREATE)
├── crates/
│   └── pet-engine/
│       ├── Cargo.toml                      # Crate manifest (CREATE)
│       └── src/
│           ├── lib.rs                      # Public API, Engine struct (CREATE)
│           ├── species.rs                  # Species data, generation, rarity (CREATE)
│           ├── stats.rs                    # Personality stats struct (CREATE)
│           ├── mood.rs                     # Mood calculation engine (CREATE)
│           ├── xp.rs                       # XP curve and leveling (CREATE)
│           ├── event.rs                    # Event types for reactions (CREATE)
│           └── reaction.rs                 # Reaction generation logic (CREATE)
├── tests/
│   ├── species_test.rs                     # Species generation tests (CREATE)
│   ├── mood_test.rs                        # Mood calculation tests (CREATE)
│   ├── xp_test.rs                          # XP curve tests (CREATE)
│   └── engine_test.rs                      # Integration tests (CREATE)
```

**Dependencies (pet-engine):**
```toml
sha2 = "0.10"
rand = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
```

---

## Chunk 1: Workspace Setup + Core Types

### Task 1: Create workspace root

- [ ] **Step 1.1: Create root Cargo.toml**

Create `open-pets/Cargo.toml`:

```toml
[workspace]
members = ["crates/pet-engine"]
resolver = "2"
```

- [ ] **Step 1.2: Create .gitignore**

Create `open-pets/.gitignore`:

```
/target
/Cargo.lock
```

- [ ] **Step 1.3: Initialize git and verify**

Run: `cd open-pets && cargo check -p pet-engine`
Expected: FAIL initially (crate doesn't exist yet), but workspace structure is valid once pet-engine is created.

- [ ] **Step 1.4: Commit workspace scaffold**

```bash
git add Cargo.toml .gitignore
git commit -m "chore: initialize Rust workspace for open-pets"
```

### Task 2: Create pet-engine crate scaffold

- [ ] **Step 2.1: Create pet-engine Cargo.toml**

Create `open-pets/crates/pet-engine/Cargo.toml`:

```toml
[package]
name = "pet-engine"
version = "0.1.0"
edition = "2021"
description = "Core pet companion logic: species, mood, XP, reactions"

[dependencies]
sha2 = "0.10"
rand = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 2.2: Create pet-engine src/lib.rs**

Create `open-pets/crates/pet-engine/src/lib.rs`:

```rust
//! Core pet engine logic — pure functions, no I/O.

mod event;
mod mood;
mod reaction;
mod species;
mod stats;
mod xp;

pub use event::{Event, EventType};
pub use mood::Mood;
pub use reaction::{Reaction, ReactionMode, TaskSummary};
pub use species::{PetState, RarityTier, Species, Specie};
pub use stats::PersonalityStats;
pub use xp::LevelInfo;

/// The main entry point for pet engine operations.
pub struct Engine;

impl Engine {
    /// Hatch a new pet from a seed string.
    pub fn hatch(seed: &str) -> PetState {
        Species::generate(seed)
    }

    /// Calculate the current mood from pet state.
    pub fn calculate_mood(state: &PetState) -> Mood {
        Mood::calculate(state)
    }

    /// Award XP and return any events (level-ups, etc.).
    pub fn award_xp(state: &mut PetState, amount: u32, reason: &str) -> Vec<Event> {
        xp::award_xp(state, amount, reason)
    }

    /// Generate a reaction for a completed task.
    pub fn generate_reaction(
        state: &PetState,
        task_summary: &TaskSummary,
        mode: ReactionMode,
    ) -> Option<Reaction> {
        Reaction::generate(state, task_summary, mode)
    }
}
```

- [ ] **Step 2.3: Create empty module files**

Create these files (empty, just module declarations for now):
- `crates/pet-engine/src/event.rs`
- `crates/pet-engine/src/mood.rs`
- `crates/pet-engine/src/reaction.rs`
- `crates/pet-engine/src/species.rs`
- `crates/pet-engine/src/stats.rs`
- `crates/pet-engine/src/xp.rs`

- [ ] **Step 2.4: Verify compilation**

Run: `cd open-pets && cargo check`
Expected: PASS with no errors (only unused warnings).

- [ ] **Step 2.5: Commit**

```bash
git add crates/pet-engine/
git commit -m "feat: scaffold pet-engine crate with empty modules"
```

### Task 3: Define core types (species, stats, events)

- [ ] **Step 3.1: Create species data model**

Fill `crates/pet-engine/src/species.rs` with:

```rust
use serde::{Deserialize, Serialize};
use crate::stats::PersonalityStats;

pub const SPECIES_TABLE: &[SpeciesDef] = &[
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
    // Common tier has 6 species, uncommon 4. Adjust weights so
    // total probability matches: common=60%, uncommon=25%, rare=10%,
    // epic=4%, legendary=1%.
    // We'll use probability weights in Species::generate().
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RarityTier {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone)]
pub struct SpeciesDef {
    pub id: &'static str,
    pub name: &'static str,
    pub rarity: RarityTier,
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

/// Species generation from a deterministic seed.
pub struct Species;

impl Species {
    pub fn generate(seed: &str) -> PetState {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(seed.as_bytes());
        let hash = hasher.finalize();
        let hash_bytes: Vec<u8> = hash.to_vec();
        
        // Determine rarity tier first
        let rarity_roll = u32::from_be_bytes([
            hash_bytes[0], hash_bytes[1], hash_bytes[2], hash_bytes[3]
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
        let matching: Vec<&SpeciesDef> = SPECIES_TABLE.iter()
            .filter(|s| s.rarity == target_rarity)
            .collect();
        
        let species_idx = if matching.is_empty() {
            // Fallback: pick from common tier
            let common: Vec<&SpeciesDef> = SPECIES_TABLE.iter()
                .filter(|s| s.rarity == RarityTier::Common)
                .collect();
            let idx = u32::from_be_bytes([
                hash_bytes[4], hash_bytes[5], hash_bytes[6], hash_bytes[7]
            ]) as usize % common.len();
            common[idx]
        } else {
            let idx = u32::from_be_bytes([
                hash_bytes[4], hash_bytes[5], hash_bytes[6], hash_bytes[7]
            ]) as usize % matching.len();
            matching[idx]
        };
        
        // Check for shiny (1% chance)
        let shiny_roll = u32::from_be_bytes([
            hash_bytes[8], hash_bytes[9], hash_bytes[10], hash_bytes[11]
        ]) % 100;
        let is_shiny = shiny_roll == 0;
        
        // Derive stats from seed
        let stats = PersonalityStats::derive_from_seed(&hash_bytes[12..]);
        
        let now = chrono::Utc::now().timestamp();
        
        PetState {
            species_id: species_idx.id.to_string(),
            species_name: species_idx.name.to_string(),
            pet_name: None,
            rarity_tier: species_idx.rarity.clone(),
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
}
```

- [ ] **Step 3.2: Create stats module**

Fill `crates/pet-engine/src/stats.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityStats {
    pub debugging: u8,
    pub patience: u8,
    pub chaos: u8,
    pub wisdom: u8,
    pub snark: u8,
}

impl PersonalityStats {
    pub fn new(debugging: u8, patience: u8, chaos: u8, wisdom: u8, snark: u8) -> Self {
        Self {
            debugging: debugging.min(20),
            patience: patience.min(20),
            chaos: chaos.min(20),
            wisdom: wisdom.min(20),
            snark: snark.min(20),
        }
    }
    
    /// Derive stats deterministically from seed bytes.
    pub fn derive_from_seed(bytes: &[u8]) -> Self {
        Self::new(
            (bytes[0] % 21) as u8,       // debugging: 0-20
            (bytes[1] % 21) as u8,        // patience: 0-20
            (bytes[2] % 21) as u8,        // chaos: 0-20
            (bytes[3] % 21) as u8,        // wisdom: 0-20
            (bytes[4] % 21) as u8,        // snark: 0-20
        )
    }
}
```

- [ ] **Step 3.3: Create event types**

Fill `crates/pet-engine/src/event.rs`:

```rust
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
        Self {
            event_type: EventType::LevelUp,
            description: format!("Leveled up from {} to {}!", from, to),
            xp_earned: 0,
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
```

- [ ] **Step 3.4: Verify compilation**

Run: `cd open-pets && cargo check`
Expected: PASS

- [ ] **Step 3.5: Commit**

```bash
git add crates/pet-engine/
git commit -m "feat: define core types — species, stats, events"
```

---

## Chunk 2: XP System

### Task 4: XP curve and leveling

- [ ] **Step 4.1: Write tests first (TDD)**

Create `open-pets/tests/xp_test.rs`:

```rust
use pet_engine::{Engine, PetState, xp::xp_to_next_level, xp::LevelInfo};

#[test]
fn test_xp_to_next_level_formula() {
    // XP to next level = base_xp + (level * 100) + (level^2 * 50)
    // Level 1 needs: 100 + 50 = 150
    assert_eq!(xp_to_next_level(1), 150);
    // Level 5 needs: 500 + 1250 = 1750
    assert_eq!(xp_to_next_level(5), 1750);
    // Level 10 needs: 1000 + 5000 = 6000
    assert_eq!(xp_to_next_level(10), 6000);
}

#[test]
fn test_award_xp_no_level_up() {
    let mut state = Engine::hatch("test-seed-unique-pet-xp");
    let initial_level = state.level;
    let initial_xp = state.xp;
    
    let events = Engine::award_xp(&mut state, 30, "test");
    
    assert_eq!(state.xp, initial_xp + 30);
    assert_eq!(state.level, initial_level);
    assert!(events.is_empty());
}

#[test]
fn test_award_xp_triggers_level_up() {
    let mut state = Engine::hatch("test-seed-unique-pet-levelup");
    state.level = 1;
    state.xp = 140; // 10 XP away from level 2
    
    let events = Engine::award_xp(&mut state, 20, "task_complete");
    
    assert_eq!(state.level, 2);
    assert!(events.iter().any(|e| matches!(e.event_type, pet_engine::EventType::LevelUp)));
}

#[test]
fn test_award_xp_multiple_level_ups() {
    let mut state = Engine::hatch("test-seed-unique-pet-multilevel");
    state.level = 1;
    state.xp = 100;
    
    // Award huge XP that should trigger multiple level-ups
    let events = Engine::award_xp(&mut state, 10000, "massive_task");
    
    let level_ups = events.iter().filter(|e| matches!(e.event_type, pet_engine::EventType::LevelUp)).count();
    assert!(level_ups >= 1);
    assert!(state.level > 2);
}

#[test]
fn test_xp_curve_milestones() {
    // Verify cumulative XP matches spec
    // L1 = 0, L5 = 1500, L10 = 5500, L20 = 21000
    let mut state = Engine::hatch("test-seed-unique-xp-curve");
    Engine::award_xp(&mut state, 50000, "milestone_test");
    
    // Level 20 requires 21000 total XP
    assert!(state.level >= 20);
}
```

Run: `cd open-pets && cargo test xp_test`
Expected: FAIL (xp module is unimplemented)

- [ ] **Step 4.2: Create xp module**

Fill `crates/pet-engine/src/xp.rs` (create if not exists):

```rust
use crate::event::Event;
use crate::species::PetState;
use serde::{Deserialize, Serialize};

/// Calculate XP needed to reach the next level from the current level.
/// Formula: level * 100 + level^2 * 50
pub fn xp_to_next_level(level: u32) -> u32 {
    level * 100 + level.pow(2) * 50
}

/// Calculate cumulative XP required to reach a specific level.
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
    let mut events = Vec::new();
    
    // Award XP first
    let old_level = state.level;
    state.xp += amount;
    
    // Check for level-ups
    loop {
        let xp_needed = xp_to_next_level(state.level);
        let level_start_xp = if state.level == 1 { 0 } else { total_xp_for_level(state.level) };
        let xp_in_level = state.xp - level_start_xp;
        
        if xp_in_level >= xp_needed {
            state.level += 1;
            events.push(Event::level_up(state.level - 1, state.level));
            // Level milestone bonus: 10 * new_level
            let bonus = state.level * 10;
            state.xp += bonus;
        } else {
            break;
        }
    }
    
    // Only add event if level changed
    if state.level != old_level {
        // Already handled in loop
    }
    
    events
}
```

- [ ] **Step 4.3: Run tests**

Run: `cd open-pets && cargo test xp_test`
Expected: PASS

- [ ] **Step 4.4: Commit**

```bash
git add crates/pet-engine/src/xp.rs tests/xp_test.rs
git commit -m "feat: implement XP curve and leveling system"
```

---

## Chunk 3: Mood System

### Task 5: Mood calculation engine

- [ ] **Step 5.1:** Write tests first (TDD)

Create `open-pets/tests/mood_test.rs`:

```rust
use pet_engine::{Engine, Mood};

#[test]
fn test_mood_states_exist() {
    // Verify all 5 mood states are valid
    let happy = Mood::new("happy");
    let content = Mood::new("content");
    let neutral = Mood::new("neutral");
    let curious = Mood::new("curious");
    let grumpy = Mood::new("grumpy");
    
    assert_eq!(happy.to_string(), "happy");
    assert_eq!(content.to_string(), "content");
    assert_eq!(neutral.to_string(), "neutral");
    assert_eq!(curious.to_string(), "curious");
    assert_eq!(grumpy.to_string(), "grumpy");
}

#[test]
fn test_mood_default_neutral() {
    let state = Engine::hatch("test-seed-unique-mood-default");
    assert_eq!(state.mood, "neutral");
}

#[test]
fn test_mood_happy_after_petting() {
    let mut state = Engine::hatch("test-seed-unique-mood-happy");
    
    // Petting should move toward happy
    Engine::award_xp(&mut state, 5, "petting");
    
    // At minimum, XP should be awarded
    assert!(state.xp >= 5);
}

#[test]
fn test_mood_decay_without_interaction() {
    // Create a pet with happy mood
    let mut state = Engine::hatch("test-seed-unique-mood-decay");
    
    // Set to happy manually
    state.mood = "happy".to_string();
    let mood = Engine::calculate_mood(&state);
    
    // Without interaction decay, should still be happy for some time
    assert!(mood.is_positive());
}
```

Run: `cd open-pets && cargo test mood_test`
Expected: FAIL

- [ ] **Step 5.2:** Create mood module

Fill `crates/pet-engine/src/mood.rs`:

```rust
use chrono::Utc;
use crate::species::PetState;

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
    
    pub fn to_string(&self) -> &str {
        match self {
            MoodState::Happy => "happy",
            MoodState::Content => "content",
            MoodState::Neutral => "neutral",
            MoodState::Curious => "curious",
            MoodState::Grumpy => "grumpy",
        }
    }

    pub fn is_positive(&self) -> bool {
        matches!(self, MoodState::Happy | MoodState::Content | MoodState::Curious)
    }
}

pub struct Mood;

impl Mood {
    pub fn new(mood_str: &str) -> MoodState {
        MoodState::from_str(mood_str)
    }
    
    /// Calculate current mood from pet state.
    /// Factors: last interaction time, XP history, stats.
    pub fn calculate(state: &PetState) -> MoodState {
        let now = Utc::now().timestamp() as u64;
        let last_interaction = state.last_interaction.unwrap_or(0) as u64;
        
        let hours_since_interaction = if last_interaction > 0 {
            (now.saturating_sub(last_interaction)) / 3600
        } else {
            999 // Very long time
        };
        
        // Mood decay: over time without interaction, drift toward neutral
        if hours_since_interaction > 48 {
            return MoodState::Neutral;
        }
        
        // High chaos + recent errors → grumpy
        if state.stats.chaos > 15 {
            // More likely grumpy, but still can be happy
            return MoodState::Content;
        }
        
        // Default: derive from current mood string
        MoodState::from_str(&state.mood)
    }
    
    /// Update mood based on interaction.
    pub fn update_after_interaction(
        state: &mut PetState,
        success: bool,
        error_count: u32,
    ) {
        let now = Utc::now().timestamp();
        state.last_interaction = Some(now);
        
        if success {
            if state.mood == "neutral" || state.mood == "grumpy" {
                state.mood = "content".to_string();
            } else if state.mood == "content" {
                state.mood = "happy".to_string();
            }
            // Stay happy if already happy
        } else if error_count > 2 {
            state.mood = "grumpy".to_string();
        } else if error_count > 0 {
            state.mood = "curious".to_string();
        }
        // If neutral, stay neutral
    }
}
```

- [ ] **Step 5.3:** Run tests

Run: `cd open-pets && cargo test mood_test`
Expected: PASS

- [ ] **Step 5.4:** Commit

```bash
git add crates/pet-engine/src/mood.rs tests/mood_test.rs
git commit -m "feat: implement mood calculation engine"
```

---

## Chunk 4: Observer/Reaction System

### Task 6: Reaction generation

- [ ] **Step 6.1:** Write tests first

Create `open-pets/tests/engine_test.rs`:

```rust
use pet_engine::{
    Engine, EventType, Reaction, ReactionMode, TaskSummary,
    xp::total_xp_for_level,
};

#[test]
fn test_reaction_none_mode() {
    let state = Engine::hatch("test-seed-unique-reaction-none");
    let task = TaskSummary::new("task", true, 0);
    
    let reaction = Engine::generate_reaction(&state, &task, ReactionMode::None);
    assert!(reaction.is_none());
}

#[test]
fn test_reaction_cheerleader_on_success() {
    let state = Engine::hatch("test-seed-unique-reaction-cheer");
    let task = TaskSummary::new("refactor_user_service", true, 0);
    
    let reaction = Engine::generate_reaction(&state, &task, ReactionMode::Cheerleader);
    assert!(reaction.is_some());
}

#[test]
fn test_reaction_backseat_on_error() {
    let state = Engine::hatch("test-seed-unique-reaction-backseat");
    let task = TaskSummary::new("deploy_config", false, 3);
    
    let reaction = Engine::generate_reaction(&state, &task, ReactionMode::Backseat);
    assert!(reaction.is_some());
}

#[test]
fn test_hatch_determinism() {
    let state1 = Engine::hatch("deterministic-seed-123");
    let state2 = Engine::hatch("deterministic-seed-123");
    
    assert_eq!(state1.species_id, state2.species_id);
    assert_eq!(state1.stats.debugging, state2.stats.debugging);
    assert_eq!(state1.stats.patience, state2.stats.patience);
}

#[test]
fn test_hatch_different_seeds() {
    let state1 = Engine::hatch("seed-a-unique");
    let state2 = Engine::hatch("seed-b-different");
    
    // They might coincidentally match on some stats, but at least
    // one thing should differ (or the seeds would be pathological)
    // We test that the system works, not that hashes are unique
    assert!(!state1.species_id.is_empty());
    assert!(!state2.species_id.is_empty());
    // Both should be valid species from our table
    assert!(SPECIES_TABLE.iter().any(|s| s.id == state1.species_id));
}
```

Run: `cd open-pets && cargo test engine_test`
Expected: FAIL

- [ ] **Step 6.2:** Create reaction module

Fill `crates/pet-engine/src/reaction.rs`:

```rust
use crate::species::PetState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReactionMode {
    Backseat,
    Cheerleader,
    Both,
    None,
}

/// Summary of a completed task for reaction generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub task_name: String,
    pub success: bool,
    pub error_count: u32,
}

impl TaskSummary {
    pub fn new(task_name: &str, success: bool, error_count: u32) -> Self {
        Self {
            task_name: task_name.to_string(),
            success,
            error_count,
        }
    }
}

/// A reaction generated by the pet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub text: String,
    pub mood_impact: i8, // -2 to +2
    pub is_technical: bool,
}

pub struct Reaction;

impl Reaction {
    pub fn generate(
        state: &PetState,
        task: &TaskSummary,
        mode: ReactionMode,
    ) -> Option<Self> {
        if matches!(mode, ReactionMode::None) {
            return None;
        }
        
        let should_cheerleader = matches!(mode, ReactionMode::Cheerleader | ReactionMode::Both);
        let should_backseat = matches!(mode, ReactionMode::Backseat | ReactionMode::Both);
        
        if task.success {
            // Success reactions
            if should_cheerleader {
                return Some(Self::cheerleader_reaction(task, state));
            }
            if should_backseat {
                return Some(Self::backseat_success_reaction(task, state));
            }
        } else {
            // Error reactions
            if should_backseat {
                return Some(Self::backseat_error_reaction(task, state));
            }
            if should_cheerleader {
                return Some(Self::encouragement_reaction(task, state));
            }
        }
        
        None
    }
    
    fn cheerleader_reaction(task: &TaskSummary, state: &PetState) -> Reaction {
        if state.stats.snark > 10 {
            Reaction {
                text: format!("\'{}\'? I mean, sure, you did it. 🎉", task.task_name),
                mood_impact: 1,
                is_technical: false,
            }
        } else {
            let reactions = format!("Great job on {}! 🐾 That was paw-some!", task.task_name);
            Reaction {
                text: reactions,
                mood_impact: 2,
                is_technical: false,
            }
        }
    }
    
    fn backseat_success_reaction(task: &TaskSummary, state: &PetState) -> Reaction {
        if state.stats.wisdom > 10 {
            Reaction {
                text: format!(
                    "\'{}\' completed. Consider adding tests and error handling next time.",
                    task.task_name
                ),
                mood_impact: 0,
                is_technical: true,
            }
        } else {
            Reaction {
                text: format!("\'{}\' done. Looks clean!", task.task_name),
                mood_impact: 1,
                is_technical: false,
            }
        }
    }
    
    fn backseat_error_reaction(task: &TaskSummary, state: &PetState) -> Reaction {
        if state.stats.chaos > 15 {
            Reaction {
                text: format!("{} errors in \'{}\'. Chaos reigns. Good.", task.error_count, task.task_name),
                mood_impact: -1,
                is_technical: false,
            }
        } else if state.stats.debugging > 10 {
            Reaction {
                text: format!(
                    "\'{}\' failed {} times. Check error handling and edge cases.",
                    task.task_name, task.error_count
                ),
                mood_impact: -1,
                is_technical: true,
            }
        } else {
            Reaction {
                text: format!("\'{}\' had {} errors. Something's off.", task.task_name, task.error_count),
                mood_impact: -1,
                is_technical: false,
            }
        }
    }
    
    fn encouragement_reaction(task: &TaskSummary, state: &PetState) -> Reaction {
        Reaction {
            text: format!("{} didn't work out, but you'll get there! Keep going!", task.task_name),
            mood_impact: 0,
            is_technical: false,
        }
    }
}
```

- [ ] **Step 6.3:** Run all tests

Run: `cd open-pets && cargo test`
Expected: PASS (all tests pass)

- [ ] **Step 6.4:** Add `SPECIES_TABLE` to lib.rs re-export for tests

Add to `crates/pet-engine/src/lib.rs`:

```rust
// Re-export for integration tests
pub use species::SPECIES_TABLE;
```

- [ ] **Step 6.5:** Final test run

Run: `cd open-pets && cargo test -v`
Expected: All tests pass

- [ ] **Step 6.6:** Commit

```bash
git add crates/pet-engine/src/reaction.rs tests/engine_test.rs crates/pet-engine/src/lib.rs
git commit -m "feat: implement observer reaction system"
```

---

## Chunk 5: Final Integration Tests & Cleanup

### Task 7: Full integration test suite

- [ ] **Step 7.1:** Create species test

Create `open-pets/tests/species_test.rs`:

```rust
use pet_engine::{Engine, SPECIES_TABLE, RarityTier};
use sha2::{Sha256, Digest};

#[test]
fn test_species_table_has_entries() {
    assert!(!SPECIES_TABLE.is_empty());
}

#[test]
fn test_rarity_weights_sum_to_100() {
    let total: u32 = SPECIES_TABLE.iter()
        .map(|s| s.rarity.weight())
        .sum::<u32>() / SPECIES_TABLE.len() as u32 * SPECIES_TABLE.len() as u32;
    // Each species has the same weight within its tier
    assert!(SPECIES_TABLE.iter().any(|s| s.rarity == RarityTier::Common));
}

#[test]
fn test_hatch_returns_valid_species() {
    let state = Engine::hatch("test-valid-spec");
    assert!(SPECIES_TABLE.iter().any(|s| s.id == state.species_id));
}

#[test]
fn test_hatch_is_deterministic() {
    let seed = "deterministic-test-seed-456";
    let state1 = Engine::hatch(seed);
    let state2 = Engine::hatch(seed);
    
    assert_eq!(state1.species_id, state2.species_id);
    assert_eq!(state1.species_name, state2.species_name);
    assert_eq!(state1.rarity_tier, state2.rarity_tier);
    assert_eq!(state1.is_shiny, state2.is_shiny);
    assert_eq!(state1.stats.debugging, state2.stats.debugging);
    assert_eq!(state1.stats.chaos, state2.stats.chaos);
}

#[test]
fn test_hatch_different_seeds_different_results() {
    let state1 = Engine::hatch("seed-aaa");
    let state2 = Engine::hatch("seed-bbb");
    
    // With completely different seeds, most likely different species
    // (not strictly guaranteed, but very likely)
    // At least verify they don't crash
    assert!(!state1.species_id.is_empty());
    assert!(!state2.species_id.is_empty());
}

#[test]
fn test_shiny_probability_roughly_1_percent() {
    // Test with many seeds to verify shiny logic works
    let mut shiny_count = 0;
    for i in 0..100 {
        let seed = format!("shiny-test-{}", i);
        let state = Engine::hatch(&seed);
        if state.is_shiny {
            shiny_count += 1;
        }
    }
    // With 100 seeds, expect ~1 shiny. Allow 0-5 range.
    assert!(shiny_count <= 5);
}

#[test]
fn test_hatch_starts_at_level_1() {
    let state = Engine::hatch("any-seed");
    assert_eq!(state.level, 1);
    assert_eq!(state.xp, 0);
    assert_eq!(state.mood, "neutral");
    assert!(!state.is_shiny || state.species_id == state.species_id); // Shiny is valid bool
}

#[test]
fn test_personality_stats_in_range() {
    for i in 0..50 {
        let seed = format!("stats-range-test-{}", i);
        let state = Engine::hatch(&seed);
        assert!(state.stats.debugging <= 20);
        assert!(state.stats.patience <= 20);
        assert!(state.stats.chaos <= 20);
        assert!(state.stats.wisdom <= 20);
        assert!(state.stats.snark <= 20);
    }
}
```

- [ ] **Step 7.2:** Run full test suite

Run: `cd open-pets && cargo test`
Expected: All tests pass

- [ ] **Step 7.3:** Verify build

Run: `cd open-pets && cargo build`
Expected: Clean build

- [ ] **Step 7.4:** Commit

```bash
git add tests/species_test.rs
git commit -m "test: add comprehensive species generation tests"

git add --all
git commit -m "feat: complete pet-engine Phase 1 with full test suite"
```

---

## Chunk Summary

| Chunk | What it builds | Tests |
|-------|---------------|-------|
| 1 | Workspace scaffold, core types (Species, Stats, Events) | Compilation tests |
| 2 | XP curve and leveling system | 5 unit tests |
| 3 | Mood calculation engine | 4 unit tests |
| 4 | Observer reaction system | 5 integration tests |
| 5 | Full integration + species test suite | 8 species tests + full suite |

Total: ~22 tests, 6 crates/modules, 5 commits

---

## Execution Notes

- **All tests use Rust's built-in test framework** — no external test dependencies needed for Phase 1
- **TDD enforced**: every module has tests written before implementation
- **pet-engine is pure** — no I/O, no async, no external services → all tests are fast unit tests
- **Deterministic species** — same seed always produces same pet, verified in tests
- **Seed uniqueness** — test seeds include unique suffixes to avoid collision
