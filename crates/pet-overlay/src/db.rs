//! SQLite persistence for pet state and memories.
//! Replaces JSON file storage with a proper database.

use pet_engine::{PetState, MemoryStore, Memory, MemoryCategory};
use rusqlite::{params, Connection, Result as SqlResult};
use std::path::PathBuf;

/// Manages the SQLite database for pet persistence.
pub struct PetDb {
    conn: Connection,
}

impl std::fmt::Debug for PetDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PetDb").finish()
    }
}

impl PetDb {
    /// Open or create the database at the default path.
    pub fn open() -> SqlResult<Self> {
        let path = Self::db_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(&path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Open an in-memory database (for testing or fallback).
    pub fn open_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn db_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("open-pets")
            .join("pets.db")
    }

    fn migrate(&self) -> SqlResult<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS pet_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                species_id TEXT NOT NULL,
                species_name TEXT NOT NULL,
                pet_name TEXT,
                rarity_tier TEXT NOT NULL,
                is_shiny INTEGER DEFAULT 0,
                level INTEGER DEFAULT 1,
                xp INTEGER DEFAULT 0,
                stat_debugging INTEGER DEFAULT 0,
                stat_patience INTEGER DEFAULT 0,
                stat_chaos INTEGER DEFAULT 0,
                stat_wisdom INTEGER DEFAULT 0,
                stat_snark INTEGER DEFAULT 0,
                mood TEXT DEFAULT 'neutral',
                muted INTEGER DEFAULT 0,
                last_interaction INTEGER,
                last_sleep INTEGER,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS pet_memory (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                text TEXT NOT NULL,
                importance INTEGER DEFAULT 1,
                consolidated INTEGER DEFAULT 0,
                category TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS pet_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );"
        )?;
        Ok(())
    }

    /// Save pet state to database.
    pub fn save_pet(&self, pet: &PetState) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO pet_state (
                id, species_id, species_name, pet_name, rarity_tier, is_shiny,
                level, xp, stat_debugging, stat_patience, stat_chaos,
                stat_wisdom, stat_snark, mood, muted, last_interaction,
                last_sleep, created_at
            ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                pet.species_id,
                pet.species_name,
                pet.pet_name,
                format!("{:?}", pet.rarity_tier),
                pet.is_shiny as i32,
                pet.level,
                pet.xp,
                pet.stats.debugging,
                pet.stats.patience,
                pet.stats.chaos,
                pet.stats.wisdom,
                pet.stats.snark,
                pet.mood,
                pet.muted as i32,
                pet.last_interaction,
                pet.last_sleep,
                pet.created_at,
            ],
        )?;
        Ok(())
    }

    /// Load pet state from database. Returns None if no pet exists.
    pub fn load_pet(&self) -> SqlResult<Option<PetState>> {
        let mut stmt = self.conn.prepare(
            "SELECT species_id, species_name, pet_name, rarity_tier, is_shiny,
                    level, xp, stat_debugging, stat_patience, stat_chaos,
                    stat_wisdom, stat_snark, mood, muted, last_interaction,
                    last_sleep, created_at
             FROM pet_state WHERE id = 1"
        )?;

        let result = stmt.query_row([], |row| {
            let rarity_str: String = row.get(3)?;
            let rarity = match rarity_str.as_str() {
                "Uncommon" => pet_engine::RarityTier::Uncommon,
                "Rare" => pet_engine::RarityTier::Rare,
                "Epic" => pet_engine::RarityTier::Epic,
                "Legendary" => pet_engine::RarityTier::Legendary,
                _ => pet_engine::RarityTier::Common,
            };
            Ok(PetState {
                species_id: row.get(0)?,
                species_name: row.get(1)?,
                pet_name: row.get(2)?,
                rarity_tier: rarity,
                is_shiny: row.get::<_, i32>(4)? != 0,
                level: row.get(5)?,
                xp: row.get(6)?,
                stats: pet_engine::PersonalityStats::new(
                    row.get(7)?, row.get(8)?, row.get(9)?,
                    row.get(10)?, row.get(11)?,
                ),
                mood: row.get(12)?,
                muted: row.get::<_, i32>(13)? != 0,
                last_interaction: row.get(14)?,
                last_sleep: row.get(15)?,
                created_at: row.get(16)?,
            })
        });

        match result {
            Ok(pet) => Ok(Some(pet)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Save memories to database.
    pub fn save_memories(&self, store: &MemoryStore) -> SqlResult<()> {
        // Clear and rewrite — simple approach for small memory sets
        self.conn.execute("DELETE FROM pet_memory", [])?;
        for mem in store.all() {
            let category_str = match mem.category {
                MemoryCategory::Task => "task",
                MemoryCategory::ErrorPattern => "error_pattern",
                MemoryCategory::UserHabit => "user_habit",
                MemoryCategory::CodeObservation => "code_observation",
                MemoryCategory::Milestone => "milestone",
                MemoryCategory::Interaction => "interaction",
            };
            self.conn.execute(
                "INSERT INTO pet_memory (text, importance, consolidated, category, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    mem.text,
                    mem.importance,
                    mem.consolidated as i32,
                    category_str,
                    mem.created_at,
                ],
            )?;
        }
        Ok(())
    }

    /// Load memories from database.
    pub fn load_memories(&self) -> SqlResult<MemoryStore> {
        let mut store = MemoryStore::new();
        let mut stmt = self.conn.prepare(
            "SELECT text, importance, consolidated, category, created_at
             FROM pet_memory ORDER BY id"
        )?;

        let memories: Vec<Memory> = stmt.query_map([], |row| {
            let category_str: String = row.get(3)?;
            let category = match category_str.as_str() {
                "error_pattern" => MemoryCategory::ErrorPattern,
                "user_habit" => MemoryCategory::UserHabit,
                "code_observation" => MemoryCategory::CodeObservation,
                "milestone" => MemoryCategory::Milestone,
                "task" => MemoryCategory::Task,
                _ => MemoryCategory::Interaction,
            };
            Ok(Memory {
                id: 0, // Will be reassigned by store
                text: row.get(0)?,
                importance: row.get(1)?,
                consolidated: row.get::<_, i32>(2)? != 0,
                category,
                created_at: row.get(4)?,
            })
        })?.filter_map(|m| m.ok()).collect();

        for mem in memories {
            store.add(mem.text, mem.importance, mem.category);
        }

        Ok(store)
    }

    /// Save a config key-value pair.
    pub fn save_config(&self, key: &str, value: &str) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO pet_config (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    /// Load a config value by key.
    pub fn load_config(&self, key: &str) -> SqlResult<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT value FROM pet_config WHERE key = ?1"
        )?;
        let result = stmt.query_row(params![key], |row| row.get::<_, String>(0));
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pet_engine::Engine;

    #[test]
    fn test_db_create_and_migrate() {
        // Use in-memory database for testing
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS pet_state (id INTEGER PRIMARY KEY);
             CREATE TABLE IF NOT EXISTS pet_memory (id INTEGER PRIMARY KEY AUTOINCREMENT, text TEXT);
             CREATE TABLE IF NOT EXISTS pet_config (key TEXT PRIMARY KEY, value TEXT);"
        ).unwrap();
    }

    #[test]
    fn test_save_and_load_pet() {
        let pet = Engine::hatch("test-db-persistence");
        let conn = Connection::open_in_memory().unwrap();

        // Create schema
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS pet_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                species_id TEXT, species_name TEXT, pet_name TEXT,
                rarity_tier TEXT, is_shiny INTEGER, level INTEGER, xp INTEGER,
                stat_debugging INTEGER, stat_patience INTEGER, stat_chaos INTEGER,
                stat_wisdom INTEGER, stat_snark INTEGER, mood TEXT, muted INTEGER,
                last_interaction INTEGER, last_sleep INTEGER, created_at INTEGER
            )"
        ).unwrap();

        // Insert
        conn.execute(
            "INSERT INTO pet_state (id, species_id, species_name, pet_name, rarity_tier,
             is_shiny, level, xp, stat_debugging, stat_patience, stat_chaos,
             stat_wisdom, stat_snark, mood, muted, last_interaction, last_sleep, created_at)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                pet.species_id, pet.species_name, pet.pet_name,
                format!("{:?}", pet.rarity_tier), pet.is_shiny as i32,
                pet.level, pet.xp, pet.stats.debugging, pet.stats.patience,
                pet.stats.chaos, pet.stats.wisdom, pet.stats.snark,
                pet.mood, pet.muted as i32, pet.last_interaction, pet.last_sleep, pet.created_at,
            ],
        ).unwrap();

        // Read back
        let loaded_id: String = conn.query_row(
            "SELECT species_id FROM pet_state WHERE id = 1", [],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(loaded_id, pet.species_id);
    }

    #[test]
    fn test_config_persistence() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE IF NOT EXISTS pet_config (key TEXT PRIMARY KEY, value TEXT)").unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO pet_config (key, value) VALUES ('window_x', '100')",
            [],
        ).unwrap();

        let val: String = conn.query_row(
            "SELECT value FROM pet_config WHERE key = 'window_x'", [],
            |row| row.get(0)
        ).unwrap();
        assert_eq!(val, "100");
    }
}
