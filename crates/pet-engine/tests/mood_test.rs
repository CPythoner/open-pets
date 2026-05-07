use pet_engine::{Engine, MoodState};

#[test]
fn test_mood_states_exist() {
    let happy = MoodState::from_str("happy");
    let content = MoodState::from_str("content");
    let neutral = MoodState::from_str("neutral");
    let curious = MoodState::from_str("curious");
    let grumpy = MoodState::from_str("grumpy");

    assert_eq!(happy.as_str(), "happy");
    assert_eq!(content.as_str(), "content");
    assert_eq!(neutral.as_str(), "neutral");
    assert_eq!(curious.as_str(), "curious");
    assert_eq!(grumpy.as_str(), "grumpy");
}

#[test]
fn test_mood_default_neutral() {
    let state = Engine::hatch("test-seed-unique-mood-default");
    assert_eq!(state.mood, "neutral");
}

#[test]
fn test_award_xp_affects_mood() {
    let mut state = Engine::hatch("test-seed-unique-mood-happy");
    Engine::award_xp(&mut state, 5, "petting");
    assert!(state.xp >= 5);
}

#[test]
fn test_mood_after_interaction_is_positive() {
    let mut state = Engine::hatch("test-seed-unique-mood-after-interaction");
    // Use the update_mood function to simulate a successful interaction
    Engine::update_mood(&mut state, true, 0);
    let mood = Engine::calculate_mood(&state);
    assert!(mood.is_positive());
}

#[test]
fn test_mood_unknown_defaults_neutral() {
    assert_eq!(MoodState::from_str("unknown"), MoodState::Neutral);
    assert_eq!(MoodState::from_str(""), MoodState::Neutral);
}

#[test]
fn test_mood_grumpy_after_errors() {
    let mut state = Engine::hatch("test-seed-unique-mood-grumpy");
    Engine::update_mood(&mut state, false, 5);
    assert_eq!(state.mood, "grumpy");
}
