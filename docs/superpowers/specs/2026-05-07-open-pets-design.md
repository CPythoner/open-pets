# Open-Pets: Desktop Pet Companion for OpenCode

**Date**: 2026-05-07
**Status**: Draft
**Author**: Orchestrator

## Context

OpenCode is an open-source AI coding agent (Go/TypeScript) with a TUI built on Bubble Tea. It has a plugin/skill system via `oh-my-opencode.json` configuration. The goal is to build a desktop pet companion system inspired by:

- **Codex Pets** (OpenAI): Floating pixel-art overlay reflecting agent state (9 animation states), 8 built-in pets + custom creation, toggle with `/pet`
- **Claude Code Buddy**: Terminal ASCII companion with 21 species, personality stats, XP/leveling, mood system, persistent SQLite memory, MCP integration

This spec defines **open-pets**: a standalone Rust desktop overlay app with full companion mechanics, specifically designed to integrate with OpenCode.

---

## 1. Architecture

### System Overview
A three-module Rust crate running as a standalone desktop process:

```
open-pets/
├── pet-overlay/      # Iced GUI — floating window with pixel-art pet
├── pet-engine/       # Core logic — stats, mood, XP, personality, memories
└── pet-sync/         # OpenCode integration — state monitoring, commentary
```

### Pet Overlay (`pet-overlay`)

**Window properties:**
- Frameless, transparent background, always-on-top (OS-specific)
- Draggable by pet body, click-through toggle via settings
- Remembers position via config file
- Snaps near taskbar/Dock edges

**Rendering:**
- `iced::widget::image` for pixel-art sprite rendering (not canvas — canvas is for vector drawing, not pixel art)
- Sprites loaded as individual PNG frames or sprite-sheet atlases from `~/.open-pets/sprites/`
- Frame animation via `iced::Subscription` ticking at 60fps, frame index calculated from sprite state + elapsed time
- States: idle, running, waiting, thinking, error, happy, grumpy, sleeping
- Each state maps to a set of animation frames (3-6 frames per state)

**Interactions:**
- Left-click pet: Pet interaction → XP, mood boost, brief animation
- Right-click: Context menu (Status, Mute, Unmute, Settings, Dismiss)
- Double-click: Full status panel

### Pet Engine (`pet-engine`)

**Species Generation:**
```
Seed = SHA256(username + hostname + project_path)
Species = hash % species_table.len()
Traits = deterministic from seed bitfields
```
Same user always gets the same species — creates attachment.

**21 Species across 5 Rarity Tiers:**
| Tier | Probability | Species |
|------|-------------|---------|
| Common | 60% | Void Cat, Code Hound, Terminal Turtle, Pixel Parrot, Debug Dragon, Rust Fox |
| Uncommon | 25% | Schema Spider, Cache Crow, Null Pointer Neko, Lambda Lizard |
| Rare | 10% | Recursion Raccoon, Stack Overflow Owl |
| Epic | 4% | Memory Leak Kraken, Race Condition Chimera |
| Legendary | 1% | Shiny variant (cosmetic aura) of any species |

**5 Personality Stats** (range 0-20):
- `DEBUGGING`: How analytical the pet is in observations
- `PATIENCE`: Reversing behavior, tolerance for slow sessions
- `CHAOS`: How unpredictable and mischievous
- `WISDOM`: Quality and depth of observations
- `SNARK`: Sarcastic commentary level

**Mood System:**
- States: Happy, Content, Neutral, Curious, Grumpy
- Influenced by: petting frequency, task outcomes, error rates, session length
- Decay curve: no interaction → drift toward Neutral
- Sleep cycle after extended sessions

**XP & Leveling:**
```
XP to next level = base_xp + (level * 100) + (level² * 50)
L1 = 0 XP
L5 = 1,500 XP
L10 = 5,500 XP
L20 = 21,000 XP
```

XP earned from: task completion (+50), petting (+5), consecutive days (+100), level milestones (+10x new level)

**Reaction Modes:**
- `Backseat`: Comments on code smells, test gaps, architectural concerns
- `Cheerleader`: Celebrates completions, level-ups, streaks
- `Both`: Mixed mode
- `None`: Silent but visible

### Pet Sync (`pet-sync`)

**OpenCode State Monitoring:**
- Polls OpenCode SQLite database for session state changes
- Detects: idle → running → waiting → error → success transitions
- FileSystemWatcher on `.opencode/` directory for state files
- WebSocket fallback for real-time updates (if OpenCode exposes it)

**Observer Pipeline:**
1. Detect task completion via OpenCode state change
2. Summarize: what files changed, commands ran, errors occurred
3. Pass summary to pet-engine for reaction generation
4. If WISDOM + DEBUGGING high → technical insights
5. If CHAOS + SNARK high → snarky commentary
6. Deliver reaction → overlay shows speech bubble (2-4 seconds)
7. Award XP and adjust mood

**Multi-instance handling:**
- Tracks all active OpenCode session IDs
- Shows aggregate state ("2 agents active, 1 waiting")
- Attributes XP to pet correctly across sessions

### Crate Interfaces

**pet-engine** (pure logic, no I/O):
```rust
pub trait Engine {
    fn hatch(seed: &str) -> PetState;
    fn calculate_mood(&self, state: &PetState) -> Mood;
    fn award_xp(&mut self, state: &mut PetState, amount: u32, reason: &str) -> Vec<Event>;
    fn generate_reaction(&self, task_summary: &TaskSummary) -> Option<Reaction>;
    fn save_state(&self, pet_id: u64, state: &PetState);
}
```

**pet-sync** (external I/O, state observation):
```rust
pub trait Sync {
    fn init(config: &Config) -> Result<Self>;
    fn watch_opencode(&self) -> impl Stream<Item = StateChange>;
    async fn load_db(&self, path: &Path) -> Result<PetState>;
    async fn save_db(&self, pet_id: u64, state: &PetState) -> Result<()>;
}
```

**pet-overlay** (GUI, consumes engine + sync):
```rust
pub struct OverlayApp {
    engine: PetEngine,
    sync: PetSync,
    state: AppState,
    sprites: SpriteManager,
}

impl Application for OverlayApp { ... }
```

All three crates communicate through trait abstractions. pet-overlay depends on pet-engine and pet-sync. pet-sync does NOT depend on pet-engine or pet-overlay. pet-engine is completely independent.

### OpenCode Integration

**No code modification needed** — open-pets runs as a companion process:
- Reads OpenCode's SQLite session database at known paths
- Monitors `.opencode/session-state.json` if available
- Optional: OpenCode skill that calls pet-sync for richer integration (petting, custom commands)

---

## 2. Data Model

### SQLite Schema (`~/.open-pets/pets.db`)

```sql
CREATE TABLE pet_state (
    id INTEGER PRIMARY KEY,
    species_id TEXT NOT NULL,
    species_name TEXT NOT NULL,
    pet_name TEXT,
    rarity_tier TEXT NOT NULL,
    is_shiny BOOLEAN DEFAULT 0,
    level INTEGER DEFAULT 1,
    xp INTEGER DEFAULT 0,
    stat_debugging INTEGER DEFAULT 0,
    stat_patience INTEGER DEFAULT 0,
    stat_chaos INTEGER DEFAULT 0,
    stat_wisdom INTEGER DEFAULT 0,
    stat_snark INTEGER DEFAULT 0,
    mood TEXT DEFAULT 'neutral',
    muted BOOLEAN DEFAULT 0,
    position_x INTEGER,
    position_y INTEGER,
    last_sleep_time DATETIME,
    last_interaction_time DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE pet_memory (
    id INTEGER PRIMARY KEY,
    memory_text TEXT NOT NULL,
    consolidated BOOLEAN DEFAULT 0,
    importance INTEGER DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE pet_level_log (
    id INTEGER PRIMARY KEY,
    level INTEGER NOT NULL,
    date_achieved DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE pet_interaction_log (
    id INTEGER PRIMARY KEY,
    interaction_type TEXT NOT NULL,
    xp_earned INTEGER DEFAULT 0,
    session_id TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### Config File (`~/.open-pets/config.json`)
```json
{
    "window": {
        "always_on_top": true,
        "click_through": false,
        "position": { "x": 50, "y": 50 }
    },
    "reaction_mode": "both",
    "speech_bubble_duration_ms": 3000,
    "animation_speed": "normal",
    "pet_name": null
}
```

---

## 3. Error Handling & Edge Cases

| Scenario | Behavior |
|----------|----------|
| OpenCode not running | Pet shows "waiting for master..." state, retries every 30s |
| Multiple OpenCode instances | Shows aggregate state, tracks per-session |
| SQLite corruption | Fallback to in-memory state, warns user, offers reset |
| Sprite files missing | Fallback to ASCII text rendering |
| High CHAOS species | More frequent but shorter reactions, never interrupts active typing |
| Window drag off-screen | Auto-returns to default position on restart |
| macOS always-on-top | Uses `NSWindow.level = .floating` |
| Windows always-on-top | Uses `HWND_TOPMOST` via Win32 |

---

## 4. Testing Strategy

### pet-engine (Unit Tests)
- Species generation determinism: same seed → same species
- XP curve formula correctness at all level boundaries
- Mood calculation: weighted formula produces valid moods
- State machine: idle → running → waiting → error → success → idle

### pet-sync (Integration Tests)
- Mock OpenCode SQLite state changes → verify sync detection
- Observer pipeline: task summary → reaction generation
- Multi-session tracking accuracy

### pet-overlay (Render Tests)
- Iced update tests for all state transitions
- Sprite rendering correctness per animation frame
- Click-through toggle behavior
- Position persistence across restarts

---

## 5. Project Structure

```
open-pets/
├── Cargo.toml                 # Workspace with 3 crates
├── Cargo.lock
├── README.md
├── .gitignore
│
├── crates/
│   ├── pet-overlay/           # Iced GUI application
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs          # Entry point, Iiced app initialization
│   │       ├── app.rs           # Main application state and update loop
│   │       ├── window.rs         # Window configuration (frameless, transparent)
│   │       ├── sprite/          # Sprite loading and rendering
│   │       │   ├── mod.rs
│   │       │   ├── loader.rs     # Load sprite sheets from filesystem
│   │       │   └── widget.rs     # Iced custom canvas widget
│   │       ├── menu.rs          # Context menu and status panel
│   │       └── speech_bubble.rs # Speech bubble component
│   │
│   ├── pet-engine/            # Core game logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── mod.rs
│   │       ├── species.rs       # Species generation, rarity, traits
│   │       ├── stats.rs         # 5 personality stats
│   │       ├── mood.rs          # Mood calculation engine
│   │       ├── xp.rs            # XP curve and leveling
│   │       ├── memory.rs         # Memory storage and consolidation
│   │       ├── observer.rs      # Reaction generation from task summaries
│   │       └── state_machine.rs # Pet state transitions
│   │
│   └── pet-sync/              # OpenCode integration layer
│       ├── Cargo.toml
│       └── src/
│           ├── mod.rs
│           ├── opencode_watcher.rs # OpenCode SQLite polling
│           ├── session_tracker.rs  # Multi-session management
│           ├── ipc.rs              # Future IPC communication
│           └── config.rs           # Config file management
│
├── sprites/                   # Default sprite sheets (pixel art)
│   ├── void-cat.png
│   ├── code-hound.png
│   └── ... (21 species)
│
└── assets/                    # Fonts, sound effects, UI assets
    └── fonts/
```

---

## 6. Key Dependencies

```toml
# pet-overlay
iced = { version = "0.13", features = ["image", "tokio"] }
iced_runtime = "0.13"

# pet-sync
rusqlite = { version = "0.32", features = ["bundled", "chrono"] }
tokio = { version = "1", features = ["fs", "sync", "rt-multi-thread"] }
notify = "7"
notify-debouncer-mini = "0.5"
dirs = "5"

# pet-engine
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
rand = "0.8"
chrono = { version = "0.4", features = ["serde"] }
```

---

## 7. Development Phases

### Phase 1: Core Engine (Week 1-2)
- [ ] Cargo workspace setup, 3 crates
- [ ] pet-engine: species generation, stats, mood, XP
- [ ] pet-sync: SQLite config, OpenCode database polling
- [ ] Unit tests for all engine logic

### Phase 2: Overlay GUI (Week 2-3)
- [ ] pet-overlay: Iced frameless window setup
- [ ] Sprite loading and canvas rendering
- [ ] State-driven animations (idle, running, waiting)
- [ ] Position persistence

### Phase 3: Interactions (Week 3-4)
- [ ] Context menu, status panel
- [ ] Speech bubble system
- [ ] Observer pipeline integration
- [ ] Reaction mode configuration

### Phase 4: Polish (Week 4-5)
- [ ] All 21 species sprite sheets
- [ ] Full test coverage (target 80% line coverage)
- [ ] Cross-platform packaging (MSI, DMG, AppImage)
- [ ] Performance profiling: sub-100ms startup, <20MB RAM idle

### Phase 5: OpenCode Skill Integration (Week 5-6)
- [ ] Create oh-my-opencode skill for pet commands (`/pet`, `/pet status`, `/pet feed`)
- [ ] Skill calls pet-sync via local API for richer in-TUI pet interactions
- [ ] Pet status visible in OpenCode TUI footer
