---
name: pet
description: Desktop pet companion for OpenCode - interact with your coding buddy
---

<Purpose>
The /pet skill lets you interact with your open-pets desktop companion directly from OpenCode. View your pet's status, pet it for XP, rename it, change reaction modes, or hatch a new species. Your pet observes your coding sessions and reacts to task outcomes.
</Purpose>

<Use_When>
- User says "/pet", "pet status", "my pet", "show pet", or mentions their open-pets companion
- User wants to check pet level, mood, stats, or XP progress
- User wants to interact with pet: pet it, feed it, rename it
- User wants to change reaction mode (cheerleader, backseat, both, silent)
- User wants to hatch a new pet species
- User says "petting", "feed pet", "pet name", "pet mood"
</Use_When>

<Do_Not_Use_When>
- User is asking about a different "pet" concept (URL parameters, HTTP pets, etc.)
- User wants to modify the open-pets codebase itself (use normal coding workflow)
</Do_Not_Use_When>

<Why_This_Exists>
The open-pets companion adds personality and fun to coding sessions. This skill provides direct CLI access to pet state without needing the GUI overlay, and integrates pet interactions into the OpenCode workflow.
</Why_This_Exists>

<Execution_Policy>
- Always read pet state from `~/.open-pets/state.json` first
- If no pet exists, offer to hatch one
- All interactions persist immediately to state file
- Use the pet-engine Rust crate logic (replicated in this skill) for consistency
- XP formulas: `total_xp_for_level(n) = sum(i=1..n-1) of (100*i + 50*i^2)`
- Mood states: happy, content, neutral, curious, grumpy
- Personality stats range: 0-20
</Execution_Policy>

<Commands>

## `/pet` or `/pet status`
Show current pet status:
```
🐾 Pixel Parrot (Lvl 5) ✨
Mood: happy | XP: 450/550
Mode: 📢 Cheerleader

Stats:
  Debug    8  [████░░░░░░]
  Patience 12 [██████░░░░]
  Chaos    15 [████████░░]
  Wisdom    6  [███░░░░░░░]
  Snark    18  [█████████░]
```

Read the state file and format it. If no state file exists, say "No pet found! Use `/pet hatch` to get one."

## `/pet pet` or `/pet stroke`
Interact with your pet:
- Award 5 XP for petting
- Set mood to "happy" (or "content" if already happy)
- Check for level-up and announce it
- Save state

## `/pet feed <item>`
Feed your pet a treat:
- Award 15 XP
- Set mood to "happy"
- Generate a food reaction message based on species personality
- Save state

## `/pet rename <name>`
Give your pet a custom name:
- Update `pet_name` in state
- Save state

## `/pet mode <cheerleader|backseat|both|silent>`
Change reaction mode:
- cheerleader: Celebrates successes ("Great job!")
- backseat: Comments on code quality and suggestions
- both: Mixed reactions
- silent: Pet is visible but doesn't react

## `/pet hatch [seed]`
Hatch a new pet from a seed string:
- Default seed: `username + hostname + timestamp`
- Custom seed: user-provided string
- WARNING: This replaces your current pet! Ask for confirmation first.
- Species determined by SHA256 hash of seed
- Rarity tiers: Common (60%), Uncommon (25%), Rare (10%), Epic (4%), Legendary (1%)
- 1% chance of shiny variant

## `/pet stats`
Show detailed personality stats with descriptions:
```
🔍 Debugging: 8/20 - Moderately analytical
⏳ Patience: 12/20 - Fairly tolerant
🎲 Chaos: 15/20 - Quite mischievous!
📖 Wisdom: 6/20 - Still learning
😏 Snark: 18/20 - Maximum sass
```

## `/pet history`
Show recent interaction log:
- List last 10 events (XP gained, mood changes, level-ups)
- Read from `~/.open-pets/history.json`

</Commands>

<Implementation>

### State File Location
- Windows: `%USERPROFILE%\.open-pets\state.json`
- Linux/macOS: `~/.open-pets/state.json`

### State File Format
```json
{
  "species_id": "pixel-parrot",
  "species_name": "Pixel Parrot",
  "pet_name": null,
  "rarity_tier": "Common",
  "is_shiny": false,
  "level": 5,
  "xp": 450,
  "stats": {
    "debugging": 8,
    "patience": 12,
    "chaos": 15,
    "wisdom": 6,
    "snark": 18
  },
  "mood": "happy",
  "muted": false,
  "last_interaction": 1715078400,
  "last_sleep": null,
  "created_at": 1714992000
}
```

### XP Formula
```
xp_to_next_level(level) = 100 * level + 50 * level^2
total_xp_for_level(n) = sum_{i=1}^{n-1} (100*i + 50*i^2)
```
Level 1→2: 150 XP, Level 5→6: 550 XP, Level 10→11: 1550 XP

### Mood Descriptions
| Mood | Description |
|------|-------------|
| happy | Your pet is thriving! Keep up the good work. |
| content | Your pet is satisfied and relaxed. |
| neutral | Your pet is chillin'. |
| curious | Your pet noticed something interesting. |
| grumpy | Your pet is not amused. Fix those errors! |

### Species Emoji Map
| Species | Emoji |
|---------|-------|
| void-cat | 🐱 |
| code-hound | 🐕 |
| terminal-turtle | 🐢 |
| pixel-parrot | 🦜 |
| debug-dragon | 🐉 |
| rust-fox | 🦊 |
| schema-spider | 🕷️ |
| cache-crow | 🐦‍⬛ |
| null-pointer-neko | 😸 |
| lambda-lizard | 🦎 |
| recursion-raccoon | 🦝 |
| stack-overflow-owl | 🦉 |
| memory-leak-kraken | 🐙 |
| race-condition-chimera | 🦄 |

### Reaction Messages

**Cheerleader (success):**
- High snark (>10): `'{task}'? I mean, sure, you did it.`
- Normal: `Great job on {task}! That was awesome!`

**Backseat (success):**
- High wisdom (>10): `'{task}' completed. Consider adding tests and error handling next time.`
- Normal: `'{task}' done. Looks clean!`

**Backseat (error):**
- High chaos (>15): `{n} errors in '{task}'. Chaos reigns. Good.`
- High debugging (>10): `'{task}' failed {n} times. Check error handling and edge cases.`
- Normal: `'{task}' had {n} errors. Something's off.`

**Cheerleader (error):**
- `{task} didn't work out, but you'll get there! Keep going!`

</Implementation>

<Examples>
<Good>
User: /pet
Response: Shows formatted pet status with species emoji, level, mood, stats bars
</Good>

<Good>
User: /pet pet
Response: "🦜 Pixel Parrot is happy! +5xp (450/550)" — with mood change and XP update
</Good>

<Good>
User: /pet rename Captain Code
Response: "Your Pixel Parrot is now called Captain Code! 🦜"
</Good>

<Good>
User: /pet mode backseat
Response: "Reaction mode set to 🧐 Backseat — your pet will comment on code quality"
</Good>

<Bad>
User: /pet (with no state file)
Response: "No pet found! Use `/pet hatch` to hatch a new companion."
</Bad>
</Examples>

<Final_Checklist>
- [ ] State file read correctly from platform-specific path
- [ ] Pet interactions award correct XP amounts
- [ ] Level-up detection works at XP thresholds
- [ ] Mood changes persist correctly
- [ ] Reaction mode persists in config
- [ ] All stat values within 0-20 range
</Final_Checklist>
