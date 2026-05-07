use pet_engine::{Engine, ReactionMode, TaskSummary, SPECIES_TABLE};

#[test]
fn test_reaction_none_mode() {
    let state = Engine::hatch("test-seed-unique-reaction-none");
    let task = TaskSummary::new("task", true, 0);
    let reaction = Engine::generate_reaction(&state, &task, ReactionMode::None);
    assert!(reaction.is_none());
}

#[test]
fn test_reaction_cheerleader_on_success() {
    let state = Engine::hatch("test-seed-unique-reaction-cheer");
    let task = TaskSummary::new("refactor_user_service", true, 0);
    let reaction = Engine::generate_reaction(&state, &task, ReactionMode::Cheerleader);
    assert!(reaction.is_some());
    let r = reaction.unwrap();
    assert!(r.text.contains("refactor_user_service"));
}

#[test]
fn test_reaction_backseat_on_error() {
    let state = Engine::hatch("test-seed-unique-reaction-backseat");
    let task = TaskSummary::new("deploy_config", false, 3);
    let reaction = Engine::generate_reaction(&state, &task, ReactionMode::Backseat);
    assert!(reaction.is_some());
}

#[test]
fn test_reaction_both_mode() {
    let state = Engine::hatch("test-seed-unique-reaction-both");
    let success_task = TaskSummary::new("build_api", true, 0);
    let reaction = Engine::generate_reaction(&state, &success_task, ReactionMode::Both);
    assert!(reaction.is_some());
}

#[test]
fn test_hatch_determinism() {
    let state1 = Engine::hatch("deterministic-seed-123");
    let state2 = Engine::hatch("deterministic-seed-123");
    assert_eq!(state1.species_id, state2.species_id);
    assert_eq!(state1.stats.debugging, state2.stats.debugging);
    assert_eq!(state1.stats.patience, state2.stats.patience);
}

#[test]
fn test_hatch_species_from_table() {
    for i in 0..100 {
        let seed = format!("species-table-test-{}", i);
        let state = Engine::hatch(&seed);
        assert!(
            SPECIES_TABLE.iter().any(|s| s.id == state.species_id),
            "Species '{}' not found in table for seed '{}'",
            state.species_id,
            seed
        );
    }
}
