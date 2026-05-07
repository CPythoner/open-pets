use crate::app::Message;
use iced::widget::{button, column, row, text};
use iced::{Alignment, Element, Length};
use pet_engine::PetState;

/// Status panel showing pet details
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub species_name: String,
    pub level: u32,
    pub xp: u32,
    pub xp_for_next: u32,
    pub mood: String,
    pub muted: bool,
    pub debugging: u8,
    pub patience: u8,
    pub chaos: u8,
    pub wisdom: u8,
    pub snark: u8,
}

impl State {
    pub fn new(pet: &PetState) -> Self {
        Self {
            species_name: pet.species_name.clone(),
            level: pet.level,
            xp: pet.xp,
            xp_for_next: pet_engine::total_xp_for_level(pet.level + 1),
            mood: pet.mood.clone(),
            muted: pet.muted,
            debugging: pet.stats.debugging,
            patience: pet.stats.patience,
            chaos: pet.stats.chaos,
            wisdom: pet.stats.wisdom,
            snark: pet.stats.snark,
        }
    }
}

pub fn view(state: &State) -> Element<'_, Message> {
    let species = text(format!("🐾 {}", state.species_name)).size(14);
    let level_info = text(format!(
        "Level: {} | XP: {}/{}",
        state.level, state.xp, state.xp_for_next
    ))
    .size(10);
    let mood = text(format!("Mood: {}", state.mood)).size(10);
    let muted = text(if state.muted {
        "🔇 Muted"
    } else {
        "🔊 Listening"
    })
    .size(10);
    let stats = column![
        text("Stats:").size(10),
        stat_row("Debug", state.debugging),
        stat_row("Patience", state.patience),
        stat_row("Chaos", state.chaos),
        stat_row("Wisdom", state.wisdom),
        stat_row("Snark", state.snark),
    ]
    .spacing(2)
    .align_x(Alignment::Center);
    let back_btn = button(text("← Back").size(10)).on_press(Message::HideStatus);

    column![
        species,
        level_info,
        mood,
        muted,
        text("---").size(8),
        stats,
        text("---").size(8),
        back_btn
    ]
    .spacing(4)
    .align_x(Alignment::Center)
    .width(Length::Shrink)
    .into()
}

fn stat_row(name: &str, value: u8) -> Element<'_, Message> {
    let filled = (value as usize) / 2;
    let empty = 10 - filled;
    let bar = format!("[{}{}]", "█".repeat(filled), "·".repeat(empty));
    row![
        text(format!("{:<8}", name)).size(9),
        text(format!("{:>2}", value)).size(9),
        text(bar).size(9)
    ]
    .spacing(2)
    .into()
}
