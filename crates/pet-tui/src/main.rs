use pet_engine::{Engine, ReactionMode, TaskSummary};
use std::io::{self, Write};
use std::path::PathBuf;
use std::fs;

fn main() {
    println!("🐾 Open-Pets: Terminal Companion 🐾\n");

    // Load or create pet
    let mut pet = load_pet().unwrap_or_else(|| {
        let pet = Engine::hatch("desktop-terminal-default-seed");
        save_pet(&pet);
        pet
    });

    println!("Species: {} ({:?})", pet.species_name, pet.rarity_tier);
    println!("Stats: Debug {}, Patience {}, Chaos {}, Wisdom {}, Snark {}", 
        pet.stats.debugging, pet.stats.patience, 
        pet.stats.chaos, pet.stats.wisdom, pet.stats.snark);
    println!("Level: {} | Mood: {}\n", pet.level, pet.mood);

    println!("Commands: pet, status, sleep, exit\n");

    // Interactive loop
    loop {
        let _ = io::stdout().flush();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "pet" => {
                Engine::award_xp(&mut pet, 5, "petting");
                Engine::update_mood(&mut pet, true, 0);
                let reaction = Engine::generate_reaction(
                    &pet, 
                    &TaskSummary::new("petting", true, 0),
                    ReactionMode::Cheerleader
                );
                if let Some(r) = reaction {
                    println!("🐾 {}", r.text);
                }
                save_pet(&pet);
            }
            "status" => {
                println!("🐾 {} (Level {})", pet.species_name, pet.level);
                println!("📊 XP: {} / {}", pet.xp, xp_to_next(pet.level));
                println!("💭 Mood: {}", pet.mood);
                println!("🔇 Muted: {}", pet.muted);
                let mood = Engine::calculate_mood(&pet);
                println!("😄 Mood State: {:?}", mood);
            }
            "sleep" => {
                println!("🐾 {} goes to sleep... Zzz...", pet.species_name);
                pet.mood = "neutral".to_string();
                save_pet(&pet);
            }
            "exit" | "quit" => {
                println!("Goodbye! {} will wait for you.", pet.species_name);
                save_pet(&pet);
                break;
            }
            "" => continue,
            _ => println!("Unknown command. Try: pet, status, sleep, exit"),
        }
    }
}

fn xp_to_next(level: u32) -> u32 {
    level * 100 + level.pow(2) * 50
}

fn pet_state_dir() -> PathBuf {
    std::env::var("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(".open-pets")
}

fn save_pet(pet: &pet_engine::PetState) {
    let dir = pet_state_dir();
    fs::create_dir_all(&dir).ok();
    if let Ok(json) = serde_json::to_string_pretty(pet) {
        fs::write(dir.join("pet.json"), json).ok();
    }
}

fn load_pet() -> Option<pet_engine::PetState> {
    let state_file = pet_state_dir().join("pet.json");
    if let Ok(json) = fs::read_to_string(state_file) {
        return serde_json::from_str(&json).ok();
    }
    None
}
