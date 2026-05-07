use pet_engine::{Engine, RarityTier, SPECIES_TABLE};

#[test]
fn test_species_table_has_entries() {
    assert!(!SPECIES_TABLE.is_empty());
}

#[test]
fn test_all_rarity_tiers_present() {
    assert!(SPECIES_TABLE.iter().any(|s| s.rarity == RarityTier::Common));
    assert!(SPECIES_TABLE
        .iter()
        .any(|s| s.rarity == RarityTier::Uncommon));
    assert!(SPECIES_TABLE.iter().any(|s| s.rarity == RarityTier::Rare));
    assert!(SPECIES_TABLE.iter().any(|s| s.rarity == RarityTier::Epic));
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
fn test_hatch_different_seeds_dont_crash() {
    let state1 = Engine::hatch("seed-aaa");
    let state2 = Engine::hatch("seed-bbb");
    assert!(!state1.species_id.is_empty());
    assert!(!state2.species_id.is_empty());
}

#[test]
fn test_hatch_starts_at_level_1() {
    let state = Engine::hatch("any-seed-123");
    assert_eq!(state.level, 1);
    assert_eq!(state.xp, 0);
    assert_eq!(state.mood, "neutral");
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

#[test]
fn test_pet_state_has_default_values() {
    let state = Engine::hatch("default-values-test");
    assert_eq!(state.pet_name, None);
    assert_eq!(state.muted, false);
    assert_eq!(state.mood, "neutral");
}
