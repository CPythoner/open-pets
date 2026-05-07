use iced::alignment::Horizontal;
use iced::widget::{container, image, text};
use iced::{Task, Element, Length, Background, Color};
use pet_engine::{Engine, PetState};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use std::fs;

/// The main application state
#[derive(Debug, Clone)]
pub struct PetApp {
    // Pet state
    pub pet: PetState,
    pub animation_frame: u32,
    pub status_text: Option<String>,
    pub speech_bubble: Option<String>,
    pub speech_bubble_visible: bool,

    // Animation
    pub animation_speed: u64,
    pub running: bool,

    // Sprite data (loaded from filesystem)
    sprites: HashMap<String, Vec<image::Handle>>,
    asset_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Tick for animation
    Tick,
    /// Pet interaction
    PetInteraction,
    /// Show speech bubble
    ShowSpeech(String),
    /// Hide speech bubble
    HideSpeech,
    /// Exit application
    Exit,
    /// Load sprite frames
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
        .window_size(iced::Size::new(128.0, 128.0))
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .subscription(PetApp::subscription)
        .run_with(PetApp::new)
    }

    pub fn new() -> (Self, Task<Message>) {
        // Default seed — same user always gets the same pet
        let pet = Engine::hatch("desktop-overlay-default-seed");

        let app = Self {
            pet,
            animation_frame: 0,
            status_text: Some("Waiting for OpenCode...".to_string()),
            speech_bubble: None,
            speech_bubble_visible: true,
            animation_speed: 100,
            running: true,
            sprites: HashMap::new(),
            asset_dir: Self::default_asset_dir(),
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
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            PathBuf::from(userprofile).join(".open-pets").join("sprites")
        } else {
            PathBuf::from("./.open-pets/sprites")
        }
    }

    /// Load sprite frames for the current pet
    async fn load_sprites(asset_dir: PathBuf) -> Result<HashMap<String, Vec<image::Handle>>, ()> {
        let mut sprites = HashMap::new();

        // Create directory if it doesn't exist
        fs::create_dir_all(&asset_dir).ok();

        // Try to load actual sprites
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

        if sprites.is_empty() {
            Err(())
        } else {
            Ok(sprites)
        }
    }

    fn update(state: &mut Self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                // Update animation frame
                state.animation_frame = (state.animation_frame + 1) % 4;

                // Check for speech bubble timeout
                if state.speech_bubble_visible && state.speech_bubble.is_some() {
                    state.speech_bubble_visible = false;
                } else if !state.speech_bubble_visible {
                    state.speech_bubble = None;
                }

                Task::none()
            }
            Message::PetInteraction => {
                state.pet.mood = if state.pet.mood != "happy" {
                    "happy".to_string()
                } else {
                    "content".to_string()
                };
                Engine::award_xp(&mut state.pet, 5, "petting");
                state.show_speech(format!("{} is happy!", state.pet.species_name));
                Task::none()
            }
            Message::ShowSpeech(text) => {
                state.speech_bubble = Some(text);
                state.speech_bubble_visible = true;
                Task::none()
            }
            Message::HideSpeech => {
                state.speech_bubble = None;
                state.speech_bubble_visible = false;
                Task::none()
            }
            Message::Exit => std::process::exit(0),
            Message::SpriteFramesLoaded(sprites) => {
                state.sprites = sprites;
                Task::none()
            }
        }
    }

    fn view(state: &Self) -> Element<'_, Message> {
        let current_state = if state.running { "idle" } else { "sleeping" };

        // Try to show sprite image, fallback to text
        if let Some(frames) = state.sprites.get(current_state) {
            if let Some(frame) = frames.get(state.animation_frame as usize % frames.len()) {
                return container(
                    image::Image::new(frame.clone())
                        .width(Length::Shrink)
                        .height(Length::Shrink)
                )
                .width(Length::Shrink)
                .height(Length::Shrink)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Background::Color(Color::TRANSPARENT)),
                    ..Default::default()
                })
                .into();
            }
        }

        // Emoji fallback when no sprites available
        let pet_emoji = match state.pet.species_id.as_str() {
            "void-cat" => "🐱",
            "code-hound" => "🐕",
            "terminal-turtle" => "🐢",
            "pixel-parrot" => "🦜",
            "debug-dragon" => "🐉",
            "rust-fox" => "🦊",
            _ => "🐾",
        };

        let text_content = format!(
            "{}\n{} (Lvl {})\n{}",
            pet_emoji,
            state.pet.species_name,
            state.pet.level,
            state.pet.mood
        );

        container(text(text_content).size(12).align_x(Horizontal::Center))
            .width(Length::Shrink)
            .height(Length::Shrink)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::TRANSPARENT)),
                ..Default::default()
            })
            .into()
    }

    fn subscription(_state: &Self) -> iced::Subscription<Message> {
        iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick)
    }

    fn show_speech(&mut self, text: String) {
        self.speech_bubble = Some(text);
        self.speech_bubble_visible = true;
    }
}
