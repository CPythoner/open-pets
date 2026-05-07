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

## Testing

```bash
# Run all 51 tests
cargo test --workspace

# Run specific crate tests
cargo test -p pet-engine   # 37 tests (species, stats, mood, XP, reactions, memory)
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
