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
            bytes[0] % 21, // debugging: 0-20
            bytes[1] % 21, // patience: 0-20
            bytes[2] % 21, // chaos: 0-20
            bytes[3] % 21, // wisdom: 0-20
            bytes[4] % 21, // snark: 0-20
        )
    }
}
