use pet_engine::{xp_to_next_level, total_xp_for_level, Engine};

#[test]
fn test_xp_to_next_level_formula() {
    assert_eq!(xp_to_next_level(1), 150);
    assert_eq!(xp_to_next_level(5), 1750);
    assert_eq!(xp_to_next_level(10), 6000);
    assert_eq!(xp_to_next_level(20), 22000);
}

#[test]
fn test_total_xp_for_level() {
    assert_eq!(total_xp_for_level(1), 0);
    assert_eq!(total_xp_for_level(2), 150);
    assert_eq!(total_xp_for_level(3), 550);
}

#[test]
fn test_award_xp_no_level_up() {
    let mut state = Engine::hatch("test-seed-unique-pet-xp-no-level");
    state.level = 1;
    let initial_xp = state.xp;
    let events = Engine::award_xp(&mut state, 30, "test");
    assert_eq!(state.xp, initial_xp + 30);
    assert_eq!(state.level, 1);
    assert!(events.is_empty());
}

#[test]
fn test_award_xp_triggers_single_level_up() {
    let mut state = Engine::hatch("test-seed-unique-pet-levelup-single");
    state.level = 1;
    state.xp = 140;
    let events = Engine::award_xp(&mut state, 20, "task_complete");
    assert_eq!(state.level, 2);
    assert!(events.iter().any(|e| e.event_type == pet_engine::EventType::LevelUp));
}

#[test]
fn test_award_xp_multiple_level_ups() {
    let mut state = Engine::hatch("test-seed-unique-pet-multilevel");
    state.level = 1;
    state.xp = 100;
    let events = Engine::award_xp(&mut state, 10000, "massive_task");
    let level_ups = events.iter().filter(|e| e.event_type == pet_engine::EventType::LevelUp).count();
    assert!(level_ups >= 1);
    assert!(state.level >= 2);
}

#[test]
fn test_xp_curve_milestones() {
    let mut state = Engine::hatch("test-seed-unique-xp-curve");
    Engine::award_xp(&mut state, 50000, "milestone_test");
    assert!(state.level >= 5);
}
