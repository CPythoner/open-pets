//! Pet Overlay - Iced GUI application for open-pets desktop companion
//! Phase 5: Window dragging, position persistence, SQLite storage.

use iced::alignment::Horizontal;
use iced::widget::{button, column, container, image as iced_image, row, text, MouseArea};
use iced::window;
use iced::{border, Background, Color, Element, Length, Task};
use pet_engine::TaskSummary;
use pet_engine::{Engine, PetState, ReactionMode};
use pet_sync::reaction_pipeline::ReactionPipeline;
use pet_sync::{AgentState, OpenCodeState, SessionTracker, StateChange, SyncConfig};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Application screen
#[derive(Debug, Clone, PartialEq, Eq)]
enum Screen {
    Main,
    Status(crate::status_panel::State),
}

/// The main application state
#[derive(Debug)]
pub struct PetApp {
    pub pet: PetState,
    pub animation_frame: u32,
    screen: Screen,
    speech_bubble: Option<String>,
    speech_timer: std::time::Instant,
    pub running: bool,
    sprites: HashMap<String, Vec<iced_image::Handle>>,
    is_dragging: bool,
    db: crate::db::PetDb,
    window_id: Option<window::Id>,
    session_tracker: SessionTracker,
    sync_config: SyncConfig,
    last_sync_poll: std::time::Instant,
    reaction_pipeline: ReactionPipeline,
    observed_agent_state: AgentState,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Exit,
    PetPressed,
    DragWindow,
    WindowIdLoaded(window::Id),
    ShowStatus,
    HideStatus,
    ToggleMute,
    CycleReactionMode,
    SimulateTask(bool),
    SpriteFramesLoaded(HashMap<String, Vec<iced_image::Handle>>),
}

impl PetApp {
    pub fn run() -> iced::Result {
        iced::application("Open-Pets: Desktop Companion", PetApp::update, PetApp::view)
            .theme(|_| iced::Theme::Dark)
            .window_size(iced::Size::new(180.0, 180.0))
            .decorations(false)
            .transparent(true)
            .resizable(false)
            .subscription(Self::subscription)
            .run_with(Self::new)
    }

    pub fn new() -> (Self, Task<Message>) {
        // Try to open database; if it fails, continue with in-memory state
        let db = crate::db::PetDb::open().unwrap_or_else(|e| {
            log::warn!("Failed to open database: {}, using fresh state", e);
            // Create in-memory fallback — won't persist but won't crash
            crate::db::PetDb::open_in_memory().expect("in-memory SQLite should always work")
        });

        let pet = db.load_pet().ok().flatten().unwrap_or_else(|| {
            let pet = Engine::hatch("desktop-overlay-default-seed");
            if let Err(e) = db.save_pet(&pet) {
                log::warn!("Failed to save initial pet: {}", e);
            }
            pet
        });

        let sync_config = SyncConfig::load().unwrap_or_default();

        let app = Self {
            pet,
            animation_frame: 0,
            screen: Screen::Main,
            speech_bubble: None,
            speech_timer: std::time::Instant::now(),
            running: true,
            sprites: HashMap::new(),
            is_dragging: false,
            db,
            window_id: None,
            session_tracker: SessionTracker::new(),
            reaction_pipeline: ReactionPipeline::from_config(sync_config.reaction_mode.clone()),
            sync_config,
            last_sync_poll: std::time::Instant::now(),
            observed_agent_state: AgentState::Idle,
        };

        // Fetch window ID and load sprites in parallel
        let task = Task::batch(vec![
            Task::perform(
                Self::load_sprites(Self::asset_dirs()),
                |result| match result {
                    Ok(sprites) => Message::SpriteFramesLoaded(sprites),
                    Err(_) => Message::SpriteFramesLoaded(HashMap::new()),
                },
            ),
            window::get_oldest().then(|id_opt| match id_opt {
                Some(id) => Task::done(Message::WindowIdLoaded(id)),
                None => Task::none(),
            }),
        ]);

        (app, task)
    }

    fn asset_dirs() -> Vec<PathBuf> {
        vec![Self::bundled_asset_dir(), Self::user_asset_dir()]
    }

    fn bundled_asset_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets")
            .join("sprites")
    }

    fn user_asset_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("open-pets")
            .join("sprites")
    }

    fn codex_pet_dirs() -> Vec<PathBuf> {
        let codex_home = std::env::var("CODEX_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::var("HOME")
                    .map(|home| PathBuf::from(home).join(".codex"))
                    .unwrap_or_else(|_| PathBuf::from(".codex"))
            });

        vec![
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../assets")
                .join("codex")
                .join("open-pets-codex"),
            codex_home.join("pets").join("open-pets-codex"),
        ]
    }

    async fn load_sprites(
        asset_dirs: Vec<PathBuf>,
    ) -> Result<HashMap<String, Vec<iced_image::Handle>>, ()> {
        let mut sprites = HashMap::new();
        fs::create_dir_all(Self::user_asset_dir()).ok();

        for codex_pet_dir in Self::codex_pet_dirs() {
            load_codex_pet_sprites(&codex_pet_dir, &mut sprites);
        }

        let states = [
            "idle", "running", "waiting", "thinking", "happy", "grumpy", "sleeping",
        ];

        for asset_dir in asset_dirs {
            for state in &states {
                let state_dir = asset_dir.join(state);
                if !state_dir.exists() {
                    continue;
                }

                let mut paths: Vec<PathBuf> = fs::read_dir(&state_dir)
                    .ok()
                    .into_iter()
                    .flat_map(|entries| entries.flatten().map(|entry| entry.path()))
                    .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("png"))
                    .collect();
                paths.sort();

                for path in paths {
                    let key = sprite_key_for_path(state, &path);
                    sprites
                        .entry(key)
                        .or_insert_with(Vec::new)
                        .push(iced_image::Handle::from_path(path));
                }
            }
        }

        if sprites.is_empty() {
            Err(())
        } else {
            Ok(sprites)
        }
    }

    fn show_speech(&mut self, msg: impl Into<String>) {
        self.speech_bubble = Some(msg.into());
        self.speech_timer = std::time::Instant::now();
    }

    fn simulate_opencode_task(&mut self, success: bool) {
        let task_name = if success {
            "build_api"
        } else {
            "deploy_failed"
        };
        let error_count = if success { 0 } else { 2 };

        let mut oc_state = OpenCodeState::idle();
        oc_state.current_task = Some(task_name.to_string());
        oc_state.error_count = error_count;
        oc_state.agent_state = AgentState::Idle;
        self.observed_agent_state = if success {
            AgentState::Idle
        } else {
            AgentState::Error
        };

        let _changes = self
            .session_tracker
            .update_session("test-session", oc_state);

        let summary = TaskSummary::new(task_name, success, error_count);
        if let Some(reaction) = self
            .reaction_pipeline
            .process(&mut self.pet, StateChange::TaskCompleted { summary })
        {
            self.show_speech(reaction.text);
        } else {
            self.show_speech(if success {
                "✓ Task done"
            } else {
                "✗ Task failed"
            });
        }

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
        self.sync_config.reaction_mode = next;
        if let Err(e) = self.sync_config.save() {
            log::warn!("Failed to save sync config: {}", e);
        }
        self.save_state();
    }

    fn save_state(&self) {
        if let Err(e) = self.db.save_pet(&self.pet) {
            log::warn!("Failed to save pet to database: {}", e);
        }
    }

    fn update(state: &mut Self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                state.animation_frame = (state.animation_frame + 1) % 4;
                if state.speech_bubble.is_some()
                    && state.speech_timer.elapsed() > Duration::from_secs(3)
                {
                    state.speech_bubble = None;
                }
                state.poll_opencode_state();
                Task::none()
            }
            Message::Exit => {
                if let Err(e) = state.db.save_pet(&state.pet) {
                    log::warn!("Failed to save on exit: {}", e);
                }
                iced::exit()
            }
            Message::PetPressed => {
                state.pet.mood = if state.pet.mood != "happy" {
                    "happy".to_string()
                } else {
                    "content".to_string()
                };
                let xp = Engine::award_xp(&mut state.pet, 5, "petting");
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
            Message::DragWindow => {
                state.is_dragging = true;
                if let Some(id) = state.window_id {
                    window::drag(id)
                } else {
                    Task::none()
                }
            }
            Message::WindowIdLoaded(id) => {
                state.window_id = Some(id);
                log::info!("Window ID loaded: {}", id);
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
                state.show_speech(if state.pet.muted {
                    "🔇 Muted"
                } else {
                    "🔊 Unmuted"
                });
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
        let current_state = self.visual_state();

        let pet_body = self.pet_body(current_state);
        let draggable_pet = MouseArea::new(pet_body).on_press(Message::DragWindow);

        let pet_stack = if let Some(speech) = &self.speech_bubble {
            column![make_speech_bubble(speech), draggable_pet]
                .spacing(4)
                .align_x(iced::Alignment::Center)
        } else {
            column![draggable_pet].align_x(iced::Alignment::Center)
        };

        // Reaction mode indicator
        let mode_label = match self.reaction_pipeline.mode() {
            ReactionMode::Cheerleader => "📢",
            ReactionMode::Backseat => "🧐",
            ReactionMode::Both => "🎭",
            ReactionMode::None => "🔇",
        };

        let controls = row![
            button(text("🐾").size(10))
                .on_press(Message::PetPressed)
                .width(Length::FillPortion(1)),
            button(text("📊").size(10))
                .on_press(Message::ShowStatus)
                .width(Length::FillPortion(1)),
            button(text(if self.pet.muted { "🔇" } else { "🔊" }).size(10))
                .on_press(Message::ToggleMute)
                .width(Length::FillPortion(1)),
            button(text(mode_label).size(10))
                .on_press(Message::CycleReactionMode)
                .width(Length::FillPortion(1)),
            button(text("✕").size(10))
                .on_press(Message::Exit)
                .width(Length::FillPortion(1)),
        ]
        .spacing(2)
        .width(Length::Fill);

        let simulator = row![
            button(text("✅ OK").size(8))
                .on_press(Message::SimulateTask(true))
                .width(Length::FillPortion(1)),
            button(text("❌ Fail").size(8))
                .on_press(Message::SimulateTask(false))
                .width(Length::FillPortion(1)),
        ]
        .spacing(2)
        .width(Length::Fill);

        container(
            column![pet_stack, controls, simulator]
                .spacing(3)
                .align_x(iced::Alignment::Center),
        )
        .width(Length::Shrink)
        .height(Length::Shrink)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_| transparent_container())
        .into()
    }

    fn pet_body(&self, current_state: &str) -> Element<'_, Message> {
        let species_state = format!("{}/{}", current_state, self.pet.species_id);
        if let Some(frames) = self
            .sprites
            .get(&format!("codex/{}", current_state))
            .or_else(|| self.sprites.get(&species_state))
            .or_else(|| self.sprites.get(current_state))
        {
            if let Some(frame) = frames.get(self.animation_frame as usize % frames.len()) {
                return iced_image::Image::new(frame.clone())
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .into();
            }
        }

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
        column![
            text(format!("{}", pet_emoji))
                .size(48)
                .align_x(Horizontal::Center),
            text(format!(
                "{} (Lvl {})",
                self.pet.species_name, self.pet.level
            ))
            .size(11)
            .align_x(Horizontal::Center),
            text(format!(
                "{} | XP: {}/{}",
                self.pet.mood, self.pet.xp, xp_needed
            ))
            .size(9)
            .align_x(Horizontal::Center),
        ]
        .align_x(iced::Alignment::Center)
        .spacing(2)
        .into()
    }

    fn poll_opencode_state(&mut self) {
        if self.last_sync_poll.elapsed() < Duration::from_millis(self.sync_config.poll_interval_ms)
        {
            return;
        }
        self.last_sync_poll = std::time::Instant::now();

        let Some(state) = self.read_opencode_state() else {
            return;
        };

        self.observed_agent_state = state.agent_state.clone();

        let session_id = state
            .session_id
            .clone()
            .unwrap_or_else(|| "default-session".to_string());
        let changes = self.session_tracker.update_session(&session_id, state);
        if changes.is_empty() {
            return;
        }

        for change in changes {
            if let Some(reaction) = self.reaction_pipeline.process(&mut self.pet, change) {
                self.show_speech(reaction.text);
            }
        }
        self.save_state();
    }

    fn read_opencode_state(&self) -> Option<OpenCodeState> {
        for path in self.opencode_state_paths() {
            match OpenCodeState::from_state_file(&path) {
                Ok(Some(state)) => return Some(state),
                Ok(None) => {}
                Err(e) => log::warn!("{}", e),
            }
        }
        None
    }

    fn opencode_state_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd.join(".opencode").join("session-state.json"));
        }
        paths.push(self.sync_config.state_dir.join("session-state.json"));
        paths
    }

    fn visual_state(&self) -> &'static str {
        visual_state_for(self.running, &self.pet.mood, &self.observed_agent_state)
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

#[derive(Debug, serde::Deserialize)]
struct CodexPetManifest {
    #[serde(rename = "spritesheetPath")]
    spritesheet_path: String,
}

fn load_codex_pet_sprites(pet_dir: &Path, sprites: &mut HashMap<String, Vec<iced_image::Handle>>) {
    let manifest_path = pet_dir.join("pet.json");
    let Ok(manifest_text) = fs::read_to_string(&manifest_path) else {
        return;
    };
    let Ok(manifest) = serde_json::from_str::<CodexPetManifest>(&manifest_text) else {
        log::warn!(
            "Failed to parse Codex pet manifest: {}",
            manifest_path.display()
        );
        return;
    };

    let spritesheet_path = pet_dir.join(manifest.spritesheet_path);
    let Ok(atlas) = ::image::open(&spritesheet_path) else {
        log::warn!(
            "Failed to open Codex pet spritesheet: {}",
            spritesheet_path.display()
        );
        return;
    };
    if atlas.width() != 1536 || atlas.height() != 1872 {
        log::warn!(
            "Ignoring Codex pet spritesheet with invalid size {}x{}: {}",
            atlas.width(),
            atlas.height(),
            spritesheet_path.display()
        );
        return;
    }

    for (state, row, frame_count) in codex_state_mapping() {
        let frames = sprites
            .entry(format!("codex/{}", state))
            .or_insert_with(Vec::new);
        if !frames.is_empty() {
            continue;
        }

        for col in 0..frame_count {
            let cell = atlas.crop_imm(col * 192, row * 208, 192, 208).to_rgba8();
            frames.push(iced_image::Handle::from_rgba(192, 208, cell.into_raw()));
        }
    }
}

fn codex_state_mapping() -> [(&'static str, u32, u32); 6] {
    [
        ("idle", 0, 6),
        ("running", 7, 6),
        ("waiting", 6, 6),
        ("thinking", 8, 6),
        ("happy", 3, 4),
        ("grumpy", 5, 8),
    ]
}

fn sprite_key_for_path(state: &str, path: &Path) -> String {
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return state.to_string();
    };

    let base = stem
        .rsplit_once('-')
        .filter(|(_, suffix)| suffix.chars().all(|ch| ch.is_ascii_digit()))
        .map(|(prefix, _)| prefix)
        .unwrap_or(stem);

    if base == state || base == "frame" {
        state.to_string()
    } else {
        format!("{}/{}", state, base)
    }
}

fn visual_state_for(running: bool, mood: &str, agent_state: &AgentState) -> &'static str {
    if !running {
        return "sleeping";
    }

    match agent_state {
        AgentState::Running => return "running",
        AgentState::Waiting => return "waiting",
        AgentState::Thinking => return "thinking",
        AgentState::Error => return "grumpy",
        AgentState::Idle => {}
    }

    match mood {
        "happy" | "content" => "happy",
        "curious" => "thinking",
        "grumpy" => "grumpy",
        _ => "idle",
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

#[cfg(test)]
mod tests {
    use super::{sprite_key_for_path, visual_state_for};
    use pet_sync::AgentState;
    use std::path::Path;

    #[test]
    fn sprite_key_groups_species_frames() {
        assert_eq!(
            sprite_key_for_path("idle", Path::new("idle/void-cat-1.png")),
            "idle/void-cat"
        );
    }

    #[test]
    fn sprite_key_keeps_generic_frames_by_state() {
        assert_eq!(
            sprite_key_for_path("happy", Path::new("happy/frame-0.png")),
            "happy"
        );
    }

    #[test]
    fn visual_state_tracks_mood_and_running_state() {
        assert_eq!(visual_state_for(true, "happy", &AgentState::Idle), "happy");
        assert_eq!(
            visual_state_for(true, "curious", &AgentState::Idle),
            "thinking"
        );
        assert_eq!(
            visual_state_for(true, "grumpy", &AgentState::Idle),
            "grumpy"
        );
        assert_eq!(visual_state_for(true, "neutral", &AgentState::Idle), "idle");
        assert_eq!(
            visual_state_for(false, "happy", &AgentState::Running),
            "sleeping"
        );
    }

    #[test]
    fn visual_state_prefers_agent_state_over_mood() {
        assert_eq!(
            visual_state_for(true, "happy", &AgentState::Running),
            "running"
        );
        assert_eq!(
            visual_state_for(true, "happy", &AgentState::Waiting),
            "waiting"
        );
        assert_eq!(
            visual_state_for(true, "happy", &AgentState::Thinking),
            "thinking"
        );
        assert_eq!(
            visual_state_for(true, "happy", &AgentState::Error),
            "grumpy"
        );
    }

    #[test]
    fn codex_state_mapping_matches_overlay_states() {
        let states: Vec<&str> = super::codex_state_mapping()
            .into_iter()
            .map(|(state, _, _)| state)
            .collect();
        assert_eq!(
            states,
            vec!["idle", "running", "waiting", "thinking", "happy", "grumpy"]
        );
    }
}
