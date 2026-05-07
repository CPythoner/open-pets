use pet_sync::{SessionTracker, OpenCodeState, AgentState, StateChange};
use pet_engine::{Engine, ReactionMode};
use pet_sync::reaction_pipeline::ReactionPipeline;

#[test]
fn test_session_tracker_new() {
    let tracker = SessionTracker::new();
    assert_eq!(tracker.session_count(), 0);
}

#[test]
fn test_session_tracker_add_session() {
    let mut tracker = SessionTracker::new();
    let state = OpenCodeState::idle();
    let changes = tracker.update_session("session-1", state);
    assert_eq!(tracker.session_count(), 1);
    // First session, no previous state → no changes
    assert!(changes.is_empty());
}

#[test]
fn test_session_tracker_state_change_detection() {
    let mut tracker = SessionTracker::new();

    // First update: Running state
    let mut running_state = OpenCodeState::idle();
    running_state.agent_state = AgentState::Running;
    running_state.current_task = Some("build".to_string());
    tracker.update_session("s1", running_state);

    // Second update: Idle state (task completed)
    let idle_state = OpenCodeState::idle();
    let changes = tracker.update_session("s1", idle_state);

    // Should detect: AgentStateChanged + TaskCompleted
    assert!(changes.len() >= 2);
    assert!(changes.iter().any(|c| matches!(c, StateChange::AgentStateChanged { .. })));
    assert!(changes.iter().any(|c| matches!(c, StateChange::TaskCompleted { .. })));
}

#[test]
fn test_session_tracker_error_detection() {
    let mut tracker = SessionTracker::new();

    let mut running_state = OpenCodeState::idle();
    running_state.agent_state = AgentState::Running;
    running_state.current_task = Some("deploy".to_string());
    tracker.update_session("s1", running_state);

    let mut error_state = OpenCodeState::idle();
    error_state.error_count = 3;
    let changes = tracker.update_session("s1", error_state);

    // Should include Error change
    assert!(changes.iter().any(|c| matches!(c, StateChange::Error { .. })));
}

#[test]
fn test_session_tracker_remove_session() {
    let mut tracker = SessionTracker::new();
    tracker.update_session("s1", OpenCodeState::idle());
    assert_eq!(tracker.session_count(), 1);
    tracker.remove_session("s1");
    assert_eq!(tracker.session_count(), 0);
}

#[test]
fn test_session_tracker_aggregate_state() {
    let mut tracker = SessionTracker::new();

    let mut s1 = OpenCodeState::idle();
    s1.agent_state = AgentState::Running;
    tracker.update_session("s1", s1);

    let mut s2 = OpenCodeState::idle();
    s2.agent_state = AgentState::Running;
    tracker.update_session("s2", s2);

    let mut s3 = OpenCodeState::idle();
    s3.agent_state = AgentState::Idle;
    tracker.update_session("s3", s3);

    let (running, _waiting, idle) = tracker.aggregate_state();
    assert_eq!(running, 2);
    assert_eq!(idle, 1);
}

#[test]
fn test_reaction_pipeline_cheerleader_success() {
    let mut pipeline = ReactionPipeline::from_config(ReactionMode::Cheerleader);
    let mut pet = Engine::hatch("test-pipeline-cheerleader");

    let summary = pet_engine::TaskSummary::new("test_task", true, 0);
    let reaction = pipeline.process(&mut pet, StateChange::TaskCompleted { summary });

    assert!(reaction.is_some());
    let r = reaction.unwrap();
    assert!(r.text.contains("test_task"));
}

#[test]
fn test_reaction_pipeline_backseat_error() {
    let mut pipeline = ReactionPipeline::new(ReactionMode::Backseat, 0); // 0 cooldown for testing
    let mut pet = Engine::hatch("test-pipeline-backseat");

    let summary = pet_engine::TaskSummary::new("broken_task", false, 3);
    let reaction = pipeline.process(&mut pet, StateChange::TaskCompleted { summary });

    assert!(reaction.is_some());
    let r = reaction.unwrap();
    assert!(r.text.contains("broken_task"));
}

#[test]
fn test_reaction_pipeline_none_mode() {
    let mut pipeline = ReactionPipeline::from_config(ReactionMode::None);
    let mut pet = Engine::hatch("test-pipeline-none");

    let summary = pet_engine::TaskSummary::new("any_task", true, 0);
    let reaction = pipeline.process(&mut pet, StateChange::TaskCompleted { summary });

    assert!(reaction.is_none());
}

#[test]
fn test_reaction_pipeline_cooldown() {
    let mut pipeline = ReactionPipeline::new(ReactionMode::Cheerleader, 3600); // 1 hour cooldown
    let mut pet = Engine::hatch("test-pipeline-cooldown");

    // First reaction works
    let summary1 = pet_engine::TaskSummary::new("task1", true, 0);
    let r1 = pipeline.process(&mut pet, StateChange::TaskCompleted { summary: summary1 });
    assert!(r1.is_some());

    // Second reaction blocked by cooldown
    let summary2 = pet_engine::TaskSummary::new("task2", true, 0);
    let r2 = pipeline.process(&mut pet, StateChange::TaskCompleted { summary: summary2 });
    assert!(r2.is_none());
}

#[test]
fn test_reaction_pipeline_mode_switch() {
    let mut pipeline = ReactionPipeline::from_config(ReactionMode::Cheerleader);
    assert_eq!(pipeline.mode(), &ReactionMode::Cheerleader);

    pipeline.set_mode(ReactionMode::Backseat);
    assert_eq!(pipeline.mode(), &ReactionMode::Backseat);

    pipeline.set_mode(ReactionMode::Both);
    assert_eq!(pipeline.mode(), &ReactionMode::Both);
}

#[test]
fn test_opencode_state_idle() {
    let state = OpenCodeState::idle();
    assert_eq!(state.agent_state, AgentState::Idle);
    assert!(state.current_task.is_none());
    assert_eq!(state.error_count, 0);
    assert!(state.files_changed.is_empty());
}

#[test]
fn test_agent_state_change_does_not_trigger_reaction() {
    let mut pipeline = ReactionPipeline::new(ReactionMode::Cheerleader, 0);
    let mut pet = Engine::hatch("test-agent-state-change");

    let reaction = pipeline.process(
        &mut pet,
        StateChange::AgentStateChanged {
            from: AgentState::Idle,
            to: AgentState::Running,
        },
    );
    assert!(reaction.is_none());
}

#[test]
fn test_error_state_triggers_reaction() {
    let mut pipeline = ReactionPipeline::new(ReactionMode::Backseat, 0);
    let mut pet = Engine::hatch("test-error-reaction");

    let reaction = pipeline.process(
        &mut pet,
        StateChange::Error {
            message: "compilation failed".to_string(),
        },
    );
    assert!(reaction.is_some());
}
