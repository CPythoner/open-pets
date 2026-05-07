use std::path::PathBuf;
use std::fs;

use pet_engine::ReactionMode;

/// Sync configuration
#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub reaction_mode: ReactionMode,
    pub poll_interval_ms: u64,
    pub state_dir: PathBuf,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            reaction_mode: ReactionMode::Both,
            poll_interval_ms: 1000,
            state_dir: default_state_dir(),
        }
    }
}

fn default_state_dir() -> PathBuf {
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        PathBuf::from(userprofile).join(".open-pets")
    } else {
        PathBuf::from("./.open-pets")
    }
}

impl SyncConfig {
    pub fn load() -> Result<Self, String> {
        let config_file = default_state_dir().join("sync-config.json");

        if config_file.exists() {
            let content = fs::read_to_string(&config_file)
                .map_err(|e| format!("Failed to read config: {}", e))?;
            let raw: RawConfig = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse config: {}", e))?;

            Ok(Self {
                reaction_mode: match raw.reaction_mode.as_str() {
                    "backseat" => ReactionMode::Backseat,
                    "cheerleader" => ReactionMode::Cheerleader,
                    "both" => ReactionMode::Both,
                    _ => ReactionMode::None,
                },
                poll_interval_ms: raw.poll_interval_ms,
                state_dir: default_state_dir(),
            })
        } else {
            // Use defaults
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<(), String> {
        fs::create_dir_all(&self.state_dir)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;

        let config_file = self.state_dir.join("sync-config.json");
        let raw = RawConfig {
            reaction_mode: match self.reaction_mode {
                ReactionMode::Backseat => "backseat".to_string(),
                ReactionMode::Cheerleader => "cheerleader".to_string(),
                ReactionMode::Both => "both".to_string(),
                ReactionMode::None => "none".to_string(),
            },
            poll_interval_ms: self.poll_interval_ms,
        };

        let content = serde_json::to_string_pretty(&raw)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_file, content)
            .map_err(|e| format!("Failed to write config: {}", e))?;

        Ok(())
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RawConfig {
    reaction_mode: String,
    poll_interval_ms: u64,
}
