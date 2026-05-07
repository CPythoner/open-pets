//! Pet memory system — short-term memories with importance scoring and consolidation.
//!
//! Memories are observations the pet makes about coding sessions.
//! High-importance memories get consolidated (kept long-term), low-importance ones decay.

use serde::{Deserialize, Serialize};

/// A single pet memory entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Memory {
    pub id: u64,
    pub text: String,
    pub importance: u8,
    pub consolidated: bool,
    pub category: MemoryCategory,
    pub created_at: i64,
}

/// Categories of pet memories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryCategory {
    /// Task completion or failure observation
    Task,
    /// Error pattern noticed
    ErrorPattern,
    /// User behavior pattern (e.g., "likes to refactor at night")
    UserHabit,
    /// Code quality observation
    CodeObservation,
    /// Milestone event (level-up, streak)
    Milestone,
    /// General interaction memory
    Interaction,
}

/// The pet's memory store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStore {
    memories: Vec<Memory>,
    next_id: u64,
    /// Maximum short-term memories before consolidation is required
    max_short_term: usize,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            memories: Vec::new(),
            next_id: 1,
            max_short_term: 50,
        }
    }

    /// Add a new memory with auto-assigned ID and current timestamp.
    pub fn add(&mut self, text: String, importance: u8, category: MemoryCategory) -> &Memory {
        let now = chrono::Utc::now().timestamp();
        let memory = Memory {
            id: self.next_id,
            text,
            importance: importance.min(10),
            consolidated: false,
            category,
            created_at: now,
        };
        self.next_id += 1;
        self.memories.push(memory);
        // Return the last element
        self.memories.last().unwrap()
    }

    /// Get all memories, sorted by importance (descending) then by recency.
    pub fn all(&self) -> Vec<&Memory> {
        let mut refs: Vec<&Memory> = self.memories.iter().collect();
        refs.sort_by(|a, b| {
            b.importance
                .cmp(&a.importance)
                .then_with(|| b.created_at.cmp(&a.created_at))
        });
        refs
    }

    /// Get recent N memories.
    pub fn recent(&self, count: usize) -> Vec<&Memory> {
        let mut refs: Vec<&Memory> = self.memories.iter().collect();
        refs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        refs.into_iter().take(count).collect()
    }

    /// Get only consolidated (long-term) memories.
    pub fn consolidated(&self) -> Vec<&Memory> {
        self.memories
            .iter()
            .filter(|m| m.consolidated)
            .collect()
    }

    /// Get only short-term (not consolidated) memories.
    pub fn short_term(&self) -> Vec<&Memory> {
        self.memories
            .iter()
            .filter(|m| !m.consolidated)
            .collect()
    }

    /// Consolidate memories: mark high-importance short-term memories as long-term,
    /// and prune low-importance short-term memories when capacity is exceeded.
    ///
    /// Returns the number of memories consolidated and the number pruned.
    pub fn consolidate(&mut self) -> (usize, usize) {
        let mut consolidated_count = 0;
        let mut pruned_count = 0;

        // Consolidate high-importance memories (>= 7)
        for memory in &mut self.memories {
            if !memory.consolidated && memory.importance >= 7 {
                memory.consolidated = true;
                consolidated_count += 1;
            }
        }

        // Prune low-importance short-term memories if over capacity
        let short_term_count = self.memories.iter().filter(|m| !m.consolidated).count();
        if short_term_count > self.max_short_term {
            // Find IDs of low-importance short-term memories to prune
            let mut candidates: Vec<(u64, u8)> = self
                .memories
                .iter()
                .filter(|m| !m.consolidated)
                .map(|m| (m.id, m.importance))
                .collect();
            candidates.sort_by_key(|(_, imp)| *imp);

            let to_prune = short_term_count - self.max_short_term;
            let prune_ids: std::collections::HashSet<u64> = candidates
                .into_iter()
                .take(to_prune)
                .map(|(id, _)| id)
                .collect();

            let before = self.memories.len();
            self.memories.retain(|m| !prune_ids.contains(&m.id));
            pruned_count = before - self.memories.len();
        }

        (consolidated_count, pruned_count)
    }

    /// Create a memory from a task completion.
    pub fn remember_task(&mut self, task_name: &str, success: bool, error_count: u32) {
        let importance = if success { 3 } else { 5 + (error_count.min(5)) as u8 };
        let text = if success {
            format!("Task '{}' completed successfully", task_name)
        } else {
            format!("Task '{}' failed with {} errors", task_name, error_count)
        };
        self.add(text, importance, MemoryCategory::Task);
    }

    /// Create a memory from an error pattern.
    pub fn remember_error_pattern(&mut self, pattern: &str) {
        self.add(
            format!("Error pattern: {}", pattern),
            6,
            MemoryCategory::ErrorPattern,
        );
    }

    /// Create a milestone memory.
    pub fn remember_milestone(&mut self, description: &str) {
        self.add(description.to_string(), 9, MemoryCategory::Milestone);
    }

    /// Get the total number of memories.
    pub fn len(&self) -> usize {
        self.memories.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.memories.is_empty()
    }

    /// Search memories by keyword.
    pub fn search(&self, keyword: &str) -> Vec<&Memory> {
        let keyword = keyword.to_lowercase();
        self.memories
            .iter()
            .filter(|m| m.text.to_lowercase().contains(&keyword))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_store_new() {
        let store = MemoryStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_add_memory() {
        let mut store = MemoryStore::new();
        let m = store.add("test memory".to_string(), 5, MemoryCategory::Interaction);
        assert_eq!(m.id, 1);
        assert_eq!(m.text, "test memory");
        assert_eq!(m.importance, 5);
        assert!(!m.consolidated);
    }

    #[test]
    fn test_add_memory_auto_increments_id() {
        let mut store = MemoryStore::new();
        store.add("first".to_string(), 1, MemoryCategory::Interaction);
        store.add("second".to_string(), 2, MemoryCategory::Interaction);
        assert_eq!(store.len(), 2);
        assert_eq!(store.memories[0].id, 1);
        assert_eq!(store.memories[1].id, 2);
    }

    #[test]
    fn test_importance_capped_at_10() {
        let mut store = MemoryStore::new();
        let m = store.add("important".to_string(), 99, MemoryCategory::Milestone);
        assert_eq!(m.importance, 10);
    }

    #[test]
    fn test_recent_memories() {
        let mut store = MemoryStore::new();
        // Use explicit timestamps to guarantee order
        store.memories.push(Memory {
            id: 1, text: "oldest".to_string(), importance: 1,
            consolidated: false, category: MemoryCategory::Interaction, created_at: 100,
        });
        store.memories.push(Memory {
            id: 2, text: "middle".to_string(), importance: 1,
            consolidated: false, category: MemoryCategory::Interaction, created_at: 200,
        });
        store.memories.push(Memory {
            id: 3, text: "newest".to_string(), importance: 1,
            consolidated: false, category: MemoryCategory::Interaction, created_at: 300,
        });
        store.next_id = 4;

        let recent = store.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].text, "newest");
        assert_eq!(recent[1].text, "middle");
    }

    #[test]
    fn test_consolidate_high_importance() {
        let mut store = MemoryStore::new();
        store.add("low".to_string(), 3, MemoryCategory::Interaction);
        store.add("high".to_string(), 8, MemoryCategory::Milestone);
        store.add("medium".to_string(), 5, MemoryCategory::Task);

        let (consolidated, _pruned) = store.consolidate();
        assert_eq!(consolidated, 1); // Only the importance-8 memory

        let consolidated_mems = store.consolidated();
        assert_eq!(consolidated_mems.len(), 1);
        assert_eq!(consolidated_mems[0].text, "high");
    }

    #[test]
    fn test_consolidate_prunes_low_importance() {
        let mut store = MemoryStore {
            memories: Vec::new(),
            next_id: 1,
            max_short_term: 3,
        };
        // Add 5 short-term memories
        for i in 0..5 {
            store.add(format!("mem-{}", i), 1, MemoryCategory::Interaction);
        }
        assert_eq!(store.short_term().len(), 5);

        let (_consolidated, pruned) = store.consolidate();
        assert_eq!(pruned, 2); // Pruned to max_short_term=3
        assert_eq!(store.short_term().len(), 3);
    }

    #[test]
    fn test_remember_task() {
        let mut store = MemoryStore::new();
        store.remember_task("build_api", true, 0);
        store.remember_task("deploy", false, 3);

        assert_eq!(store.len(), 2);
        let task_mems: Vec<&Memory> = store.memories.iter().filter(|m| m.category == MemoryCategory::Task).collect();
        assert_eq!(task_mems.len(), 2);
        // Failure has higher importance
        assert_eq!(task_mems[0].importance, 3); // success
        assert_eq!(task_mems[1].importance, 8); // failed with 3 errors (5+3)
    }

    #[test]
    fn test_remember_milestone() {
        let mut store = MemoryStore::new();
        store.remember_milestone("Reached Level 10!");
        let m = store.memories.last().unwrap();
        assert_eq!(m.importance, 9);
        assert_eq!(m.category, MemoryCategory::Milestone);
    }

    #[test]
    fn test_search_memories() {
        let mut store = MemoryStore::new();
        store.add("deployed to production".to_string(), 5, MemoryCategory::Task);
        store.add("fixed login bug".to_string(), 3, MemoryCategory::Task);
        store.add("deployed to staging".to_string(), 4, MemoryCategory::Task);

        let results = store.search("deploy");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_all_sorted_by_importance() {
        let mut store = MemoryStore::new();
        store.add("low".to_string(), 2, MemoryCategory::Interaction);
        store.add("critical".to_string(), 9, MemoryCategory::Milestone);
        store.add("medium".to_string(), 5, MemoryCategory::Task);

        let all = store.all();
        assert_eq!(all[0].importance, 9);
        assert_eq!(all[1].importance, 5);
        assert_eq!(all[2].importance, 2);
    }
}
