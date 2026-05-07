# open-pets

Desktop pet companion for OpenCode — a Rust-coded coding buddy that watches your sessions and reacts.

## Features

- **14 species** across 5 rarity tiers (Common → Legendary), deterministically generated from seed
- **5 personality stats**: Debugging, Patience, Chaos, Wisdom, Snark (0-20 range)
- **Mood system**: Happy, Content, Neutral, Curious, Grumpy — influenced by interactions
- **XP & leveling**: Awarded from task completions, petting, and milestones
- **4 reaction modes**: Cheerleader, Backseat, Both, Silent
- **Memory system**: Pet remembers task outcomes, error patterns, and milestones
- **Desktop overlay**: Frameless transparent window with sprite animation or emoji fallback
- **Terminal companion**: TUI interface for terminal-only environments
- **OpenCode integration**: Monitors agent state and reacts to task outcomes

## Architecture

```
open-pets/
├── crates/
│   ├── pet-engine/    # Core logic — species, stats, mood, XP, reactions, memories
│   ├── pet-overlay/   # Iced GUI — floating transparent desktop window
│   ├── pet-tui/       # Terminal companion — Bubble Tea-style TUI
│   └── pet-sync/      # OpenCode state monitoring and reaction pipeline
├── assets/
│   └── sprites/       # Bundled transparent PNG pet sprites
└── skills/
    └── pet/           # OpenCode /pet skill — interact from within OpenCode
```

## Quick Start

### Build

```bash
cargo build --release
```

### Run Desktop Overlay

```bash
cargo run --release -p pet-overlay
```

### Run Terminal Companion

```bash
cargo run --release -p pet-tui
```

### Regenerate Sprites

```bash
python3 tools/generate_sprites.py
```

The overlay loads user sprites from your data directory first, then falls back to
the bundled sprites in `assets/sprites`.

### Build Codex-Compatible Pet

```bash
python3 tools/package_codex_pet.py --install
```

This creates a Codex custom pet package at `assets/codex/open-pets-codex` and
installs it to `~/.codex/pets/open-pets-codex` using the Codex contract:
`pet.json` plus a transparent `1536x1872` `spritesheet.webp` atlas.

## Usage

### Desktop Overlay Controls

| Button | Action |
|--------|--------|
| Click pet | Pet it (+5 XP, mood boost) |
| 📊 | Show stats panel |
| 🔊 | Toggle mute |
| 📢/🧐/🎭 | Cycle reaction mode |
| ✅ | Simulate successful task |
| ❌ | Simulate failed task |
| ✕ | Exit |

### OpenCode Skill

Install the `/pet` skill and use commands:

```
/pet            → Show pet status
/pet pet        → Pet your companion
/pet feed       → Feed it a treat
/pet rename X   → Rename your pet
/pet mode backseat → Change reaction mode
/pet hatch      → Hatch a new species
/pet stats      → Detailed personality stats
```

### OpenCode State File

The overlay polls these files and reacts to state changes:

- `<current-project>/.opencode/session-state.json`
- `$OPEN_PETS_HOME/session-state.json` or `~/.open-pets/session-state.json`

Supported JSON fields:

```json
{
  "session_id": "default",
  "state": "running",
  "task": "build_overlay",
  "error_count": 0,
  "files_changed": ["crates/pet-overlay/src/app.rs"]
}
```

## Testing

```bash
# Run all 61 tests
cargo test --workspace

# Run specific crate tests
cargo test -p pet-engine   # Core logic tests (species, stats, mood, XP, reactions, memory)
cargo test -p pet-sync     # 14 tests (session tracking, reaction pipeline)
```

## Cross-Platform Packaging

This project uses [cargo-dist](https://opensource.axo.dev/cargo-dist/) for building distributable binaries:

```bash
cargo dist build --target x86_64-pc-windows-msvc
cargo dist build --target x86_64-unknown-linux-gnu
cargo dist build --target aarch64-apple-darwin
```

## License

MIT
