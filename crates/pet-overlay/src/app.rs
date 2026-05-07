//! Pet Overlay - Iced GUI application for open-pets desktop companion
//! Phase 4: Interactive pet overlay with context menu, status panel,
//! click interactions, speech bubbles, and reaction mode configuration.

use iced::alignment::Horizontal;
use iced::widget::{button, column, container, image, row, text, MouseArea};
use iced::{Task, Element, Length, Background, Color, border};
use pet_engine::{Engine, PetState, ReactionMode};
use pet_sync::reaction_pipeline::ReactionPipeline;
use pet_sync::{OpenCodeState, AgentState, SessionTracker, StateChange};
use pet_engine::TaskSummary;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use std::fs;

/// Application screen
#[derive(Debug, Clone, PartialEq, Eq)]
enum Screen {
    Main,
    Status(crate::status_panel::State),
}

/// The main application state
#[derive(Debug, Clone)]
pub struct PetApp {
    pub pet: PetState,
    pub animation_frame: u32,
    screen: Screen,
    speech_bubble: Option<String>,
    speech_timer: std::time::Instant,
    pub running: bool,
    sprites: HashMap<String, Vec<image::Handle>>,
    asset_dir: PathBuf,
    #[allow(dead_code)]
    session_tracker: SessionTracker,
    reaction_pipeline: ReactionPipeline,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Exit,
    PetClicked,
    ShowStatus,
    HideStatus,
    ToggleMute,
    CycleReactionMode,
    SimulateTask(bool),
    SpriteFramesLoaded(HashMap<String, Vec<image::Handle>>),
}

impl PetApp {
    pub fn run() -> iced::Result {
        iced::application(
            "Open-Pets: Desktop Companion",
            PetApp::update,
            PetApp::view,
        )
        .theme(|_| iced::Theme::Dark)
        .window_size(iced::Size::new(180.0, 180.0))
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .subscription(Self::subscription)
        .run_with(Self::new)
    }

    pub fn new() -> (Self, Task<Message>) {
        let pet = Engine::hatch("desktop-overlay-default-seed");
        let app = Self {
            pet,
            animation_frame: 0,
            screen: Screen::Main,
            speech_bubble: None,
            speech_timer: std::time::Instant::now(),
            running: true,
            sprites: HashMap::new(),
            asset_dir: Self::default_asset_dir(),
            session_tracker: SessionTracker::new(),
            reaction_pipeline: ReactionPipeline::from_config(ReactionMode::Cheerleader),
        };

        let task = Task::perform(
            Self::load_sprites(app.asset_dir.clone()),
            |result| match result {
                Ok(sprites) => Message::SpriteFramesLoaded(sprites),
                Err(_) => Message::SpriteFramesLoaded(HashMap::new()),
            },
        );

        (app, task)
    }

    fn default_asset_dir() -> PathBuf {
        if let Ok(up) = std::env::var("USERPROFILE") {
            PathBuf::from(up).join(".open-pets").join("sprites")
        } else {
            PathBuf::from("./.open-pets/sprites")
        }
    }

    async fn load_sprites(asset_dir: PathBuf) -> Result<HashMap<String, Vec<image::Handle>>, ()> {
        let mut sprites = HashMap::new();
        fs::create_dir_all(&asset_dir).ok();

        let states = ["idle", "running", "waiting", "thinking", "happy", "grumpy", "sleeping"];
        for state in &states {
            let state_dir = asset_dir.join(state);
            if state_dir.exists() {
                let mut frames = Vec::new();
                if let Ok(entries) = fs::read_dir(&state_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) == Some("png") {
                            frames.push(image::Handle::from_path(&path));
                        }
                    }
                }
                if !frames.is_empty() {
                    sprites.insert(state.to_string(), frames);
                }
            }
        }
        if sprites.is_empty() { Err(()) } else { Ok(sprites) }
    }

    fn show_speech(&mut self, msg: impl Into<String>) {
        self.speech_bubble = Some(msg.into());
        self.speech_timer = std::time::Instant::now();
    }

    fn simulate_opencode_task(&mut self, success: bool) {
        let task_name = if success { "build_api" } else { "deploy_failed" };
        let error_count = if success { 0 } else { 2 };

        let mut oc_state = OpenCodeState::idle();
        oc_state.current_task = Some(task_name.to_string());
        oc_state.error_count = error_count;
        oc_state.agent_state = AgentState::Idle;

        let _changes = self.session_tracker.update_session("test-session", oc_state);

        let summary = TaskSummary::new(task_name, success, error_count);
        if let Some(reaction) = self.reaction_pipeline.process(
            &mut self.pet,
            StateChange::TaskCompleted { summary }
        ) {
            self.show_speech(reaction.text);
        } else {
            // No reaction (mode is None or cooldown active) — show minimal feedback
            self.show_speech(if success { "✓ Task done" } else { "✗ Task failed" });
        }

        Engine::award_xp(&mut self.pet, if success { 50 } else { 10 }, task_name);
        Engine::update_mood(&mut self.pet, success, error_count);
        self.save_state();
    }

    fn cycle_reaction_mode(&mut self) {
        let next = match self.reaction_pipeline.mode() {
            ReactionMode::Cheerleader => ReactionMode::Backseat,
            ReactionMode::Backseat => ReactionMode::Both,
            ReactionMode::Both => ReactionMode::None,
            ReactionMode::None => ReactionMode::Cheerleader,
        };
        self.reaction_pipeline.set_mode(next.clone());
        let label = match &next {
            ReactionMode::Cheerleader => "📢 Cheerleader",
            ReactionMode::Backseat => "🧐 Backseat",
            ReactionMode::Both => "🎭 Both",
            ReactionMode::None => "🔇 Silent",
        };
        self.show_speech(label.to_string());
        self.save_state();
    }

    fn save_state(&self) {
        if let Ok(up) = std::env::var("USERPROFILE") {
            let sf = PathBuf::from(up).join(".open-pets").join("state.json");
            if let Some(parent) = sf.parent() {
                fs::create_dir_all(parent).ok();
            }
            if let Ok(json) = serde_json::to_string_pretty(&self.pet) {
                fs::write(&sf, json).ok();
            }
        }
    }

    fn update(state: &mut Self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                state.animation_frame = (state.animation_frame + 1) % 4;
                if state.speech_bubble.is_some() && state.speech_timer.elapsed() > Duration::from_secs(3) {
                    state.speech_bubble = None;
                }
                Task::none()
            }
            Message::Exit => iced::exit(),
            Message::PetClicked => {
                state.pet.mood = if state.pet.mood != "happy" {
                    "happy".to_string()
                } else {
                    "content".to_string()
                };
                let xp = Engine::award_xp(&mut state.pet, 5, "petting");
                // Show level-up message if applicable
                if let Some(event) = xp.first() {
                    if matches!(event.event_type, pet_engine::EventType::LevelUp) {
                        state.show_speech(format!("🎉 Level Up! Now Lvl {}!", state.pet.level));
                    } else {
                        state.show_speech(format!("{} is happy! +5xp", state.pet.species_name));
                    }
                } else {
                    state.show_speech(format!("{} loves you! +5xp", state.pet.species_name));
                }
                state.save_state();
                Task::none()
            }
            Message::ShowStatus => {
                state.screen = Screen::Status(crate::status_panel::State::new(&state.pet));
                Task::none()
            }
            Message::HideStatus => {
                state.screen = Screen::Main;
                Task::none()
            }
            Message::ToggleMute => {
                state.pet.muted = !state.pet.muted;
                state.show_speech(if state.pet.muted { "🔇 Muted" } else { "🔊 Unmuted" });
                state.save_state();
                Task::none()
            }
            Message::CycleReactionMode => {
                state.cycle_reaction_mode();
                Task::none()
            }
            Message::SimulateTask(success) => {
                state.simulate_opencode_task(success);
                Task::none()
            }
            Message::SpriteFramesLoaded(sprites) => {
                state.sprites = sprites;
                Task::none()
            }
        }
    }

    fn view(state: &Self) -> Element<'_, Message> {
        match &state.screen {
            Screen::Main => state.main_view(),
            Screen::Status(panel) => crate::status_panel::view(panel),
        }
    }

    fn main_view(&self) -> Element<'_, Message> {
        let current_state = if self.running { "idle" } else { "sleeping" };

        // Try sprite image
        if let Some(frames) = self.sprites.get(current_state) {
            if let Some(frame) = frames.get(self.animation_frame as usize % frames.len()) {
                let img = image::Image::new(frame.clone())
                    .width(Length::Shrink)
                    .height(Length::Shrink);
                let widget = if let Some(speech) = &self.speech_bubble {
                    column![
                        make_speech_bubble(speech),
                        img
                    ].spacing(4).align_x(iced::Alignment::Center)
                } else {
                    column![img].align_x(iced::Alignment::Center)
                };
                return widget.into();
            }
        }

        // Emoji fallback mode — clickable pet area
        let pet_emoji = match self.pet.species_id.as_str() {
            "void-cat" => "🐱",
            "code-hound" => "🐕",
            "terminal-turtle" => "🐢",
            "pixel-parrot" => "🦜",
            "debug-dragon" => "🐉",
            "rust-fox" => "🦊",
            "schema-spider" => "🕷️",
            "cache-crow" => "🐦‍⬛",
            "null-pointer-neko" => "😸",
            "lambda-lizard" => "🦎",
            "recursion-raccoon" => "🦝",
            "stack-overflow-owl" => "🦉",
            "memory-leak-kraken" => "🐙",
            "race-condition-chimera" => "🦄",
            _ => "🐾",
        };

        let xp_needed = pet_engine::total_xp_for_level(self.pet.level + 1);

        // Pet display — clickable via MouseArea
        let pet_display = column![
            text(format!("{}", pet_emoji))
                .size(48).align_x(Horizontal::Center),
            text(format!(
                "{} (Lvl {})",
                self.pet.species_name, self.pet.level
            ))
            .size(11).align_x(Horizontal::Center),
            text(format!(
                "{} | XP: {}/{}",
                self.pet.mood, self.pet.xp, xp_needed
            ))
            .size(9).align_x(Horizontal::Center),
        ]
        .align_x(iced::Alignment::Center)
        .spacing(2);

        // Wrap pet in clickable MouseArea
        let clickable_pet = MouseArea::new(pet_display)
            .on_press(Message::PetClicked);

        let inner = if let Some(speech) = &self.speech_bubble {
            column![
                make_speech_bubble(speech),
                clickable_pet
            ].spacing(4).align_x(iced::Alignment::Center)
        } else {
            column![clickable_pet].align_x(iced::Alignment::Center)
        };

        // Reaction mode indicator
        let mode_label = match self.reaction_pipeline.mode() {
            ReactionMode::Cheerleader => "📢",
            ReactionMode::Backseat => "🧐",
            ReactionMode::Both => "🎭",
            ReactionMode::None => "🔇",
        };

        // Action buttons row 1: pet management
        let row1 = row![
            button(text("📊").size(10)).on_press(Message::ShowStatus).width(Length::FillPortion(1)),
            button(text("🔊").size(10)).on_press(Message::ToggleMute).width(Length::FillPortion(1)),
            button(text(mode_label).size(10)).on_press(Message::CycleReactionMode).width(Length::FillPortion(1)),
            button(text("✕").size(10)).on_press(Message::Exit).width(Length::FillPortion(1)),
        ].spacing(2).width(Length::Fill);

        // Action buttons row 2: simulate OpenCode tasks (dev/test only)
        let row2 = row![
            button(text("✅ OK").size(8)).on_press(Message::SimulateTask(true)).width(Length::FillPortion(1)),
            button(text("❌ Fail").size(8)).on_press(Message::SimulateTask(false)).width(Length::FillPortion(1)),
        ].spacing(2).width(Length::Fill);

        container(
            column![inner, row1, row2]
                .spacing(3)
                .align_x(iced::Alignment::Center)
        )
        .width(Length::Shrink)
        .height(Length::Shrink)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_| transparent_container())
        .into()
    }

    fn subscription(_state: &Self) -> iced::Subscription<Message> {
        iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick)
    }
}

fn transparent_container() -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        ..Default::default()
    }
}

fn make_speech_bubble(msg: &str) -> Element<'static, Message> {
    container(text(msg.to_string()).size(10))
        .padding([2, 6])
        .style(|_| iced::widget::container::Style {
            background: Some(Background::Color(Color::from_rgb8(40, 40, 60))),
            border: border::rounded(6),
            text_color: Some(Color::from_rgb8(220, 220, 240)),
            ..Default::default()
        })
        .into()
}
