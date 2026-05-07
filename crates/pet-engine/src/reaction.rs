use crate::species::PetState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReactionMode {
    Backseat,
    Cheerleader,
    Both,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub task_name: String,
    pub success: bool,
    pub error_count: u32,
}

impl TaskSummary {
    pub fn new(task_name: &str, success: bool, error_count: u32) -> Self {
        Self {
            task_name: task_name.to_string(),
            success,
            error_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub text: String,
    pub mood_impact: i8,
    pub is_technical: bool,
}

pub fn generate_reaction(
    state: &PetState,
    task: &TaskSummary,
    mode: &ReactionMode,
) -> Option<Reaction> {
    if matches!(mode, ReactionMode::None) {
        return None;
    }

    let allow_cheerleader = matches!(mode, ReactionMode::Cheerleader | ReactionMode::Both);
    let allow_backseat = matches!(mode, ReactionMode::Backseat | ReactionMode::Both);

    if task.success {
        if allow_cheerleader {
            return Some(cheerleader_reaction(task, state));
        }
        if allow_backseat {
            return Some(backseat_success_reaction(task, state));
        }
    } else {
        if allow_backseat {
            return Some(backseat_error_reaction(task, state));
        }
        if allow_cheerleader {
            return Some(encouragement_reaction(task, state));
        }
    }

    None
}

fn cheerleader_reaction(task: &TaskSummary, state: &PetState) -> Reaction {
    if state.stats.snark > 10 {
        Reaction {
            text: format!("\'{}\'? I mean, sure, you did it.", task.task_name),
            mood_impact: 1,
            is_technical: false,
        }
    } else {
        Reaction {
            text: format!("Great job on {}! That was awesome!", task.task_name),
            mood_impact: 2,
            is_technical: false,
        }
    }
}

pub fn backseat_error_reaction(task: &TaskSummary, state: &PetState) -> Reaction {
    if state.stats.chaos > 15 {
        Reaction {
            text: format!(
                "{} errors in '{}'. Chaos reigns. Good.",
                task.error_count, task.task_name
            ),
            mood_impact: 1,
            is_technical: false,
        }
    } else if state.stats.debugging > 10 {
        Reaction {
            text: format!(
                "'{}' failed {} times. Check error handling and edge cases.",
                task.task_name, task.error_count
            ),
            mood_impact: 1,
            is_technical: true,
        }
    } else {
        Reaction {
            text: format!(
                "'{}' had {} errors. Something's off.",
                task.task_name, task.error_count
            ),
            mood_impact: 1,
            is_technical: false,
        }
    }
}

pub fn backseat_success_reaction(task: &TaskSummary, state: &PetState) -> Reaction {
    if state.stats.wisdom > 10 {
        Reaction {
            text: format!(
                "'{}' completed. Consider adding tests and error handling next time.",
                task.task_name
            ),
            mood_impact: 0,
            is_technical: true,
        }
    } else {
        Reaction {
            text: format!("'{}' done. Looks clean!", task.task_name),
            mood_impact: 1,
            is_technical: false,
        }
    }
}

pub fn encouragement_reaction(task: &TaskSummary, _state: &PetState) -> Reaction {
    Reaction {
        text: format!(
            "{} didn't work out, but you'll get there! Keep going!",
            task.task_name
        ),
        mood_impact: 0,
        is_technical: false,
    }
}
