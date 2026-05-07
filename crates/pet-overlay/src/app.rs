use iced::advanced::graphics::image::image_rs;
use iced::alignment::Horizontal;
use iced::widget::{column, container, text};
use iced::{
    executor, window, Application, Command, Element, Length, Settings, Subscription, Theme,
};
use pet_engine::{Engine, Reaction, ReactionMode, TaskSummary, MoodState, Mood, PetState};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use std::fs;

/// The main application state
#[derive(Debug, Clone)]
pub struct App {
    // Pet state
    pub pet: PetState,
    pub animation_frame: u32,
    pub current_sprite: Option<iced::widget::image::Handle>,
    pub status_text: Option<String>,
    pub speech_bubble: Option<String>,
    pub speech_timer: Option<std::time::Instant>,

    // Window state
    window_id: Option<iced::window::Id>,
    position: (i32, i32),
    is_dragging: bool,

    // Animation
    pub animation_speed: u64,
    pub running: bool,

    // Sprite data (loaded from filesystem)
    sprites: HashMap<String, Vec<iced::widget::image::Handle>>,
    asset_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Tick for animation
    Tick,
    /// Load pet from storage or create new one
    LoadPet(PetState),
    /// Update pet state
    UpdatePetState,
    /// Pet interaction
    PetInteraction,
    /// Show speech bubble
    ShowSpeech(String),
    /// Hide speech bubble
    HideSpeech,
    /// Exit application
    Exit,
    /// Window was moved
    WindowMoved((i32, i32)),
    /// Load sprite frames
    SpriteFramesLoaded(HashMap<String, Vec<iced::widget::image::Handle>>),
}

impl App {
    pub fn new() -> Self {
        // Generate a default pet for now
        let pet = Engine::hatch("desktop-overlay-default-seed");

        Self {
            pet,
            animation_frame: 0,
            current_sprite: None,
            status_text: Some("Waiting for OpenCode...".to_string()),
            speech_bubble: None,
            speech_timer: None,
            window_id: None,
            position: (50, 50),
            is_dragging: false,
            animation_speed: 100, // ms between frames
            running: true,
            sprites: HashMap::new(),
            asset_dir: Self::default_asset_dir(),
        }
    }

    fn default_asset_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".open-pets")
            .join("sprites")
    }

    /// Load sprite frames for the current pet
    async fn load_sprites(asset_dir: PathBuf) -> Result<HashMap<String, Vec<iced::widget::image::Handle>>, String> {
        // For now, use a simple placeholder if sprites don't exist
        let mut sprites = HashMap::new();

        // Create directory if it doesn't exist
        fs::create_dir_all(&asset_dir).ok();

        // Try to load actual sprites
        let states = ["idle", "running", "waiting", "thinking", "happy", "grumpy", "sleeping"];
        for state in &states {
            let state_dir = asset_dir.join(state);
            if state_dir.exists() {
                // Load frames from state directory
                let mut frames = Vec::new();
                if let Ok(entries) = fs::read_dir(&state_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) == Some("png") {
                            frames.push(iced::widget::image::Handle::from_path(&path));
                        }
                    }
                }

                if !frames.is_empty() {
                    sprites.insert(state.to_string(), frames);
                }
            }
        }

        // If no sprites found, use a text-based fallback
        if sprites.is_empty() {
            // This is handled in view with a text placeholder
            sprites.insert("idle".to_string(), vec![iced::widget::image::Handle::from_path("")]);
        }

        Ok(sprites)
    }

    fn save_state(&self) {
        // Save pet state for persistence
        if let Some(home) = dirs::home_dir() {
            let state_file = home.join(".open-pets").join("state.json");
            fs::create_dir_all(state_file.parent().unwrap()).ok();
            if let Ok(json) = serde_json::to_string_pretty(&self.pet) {
                fs::write(&state_file, json).ok();
            }
        }
    }

    fn load_state() -> Option<PetState> {
        if let Some(home) = dirs::home_dir() {
            let state_file = home.join(".open-pets").join("state.json");
            if let Ok(json) = fs::read_to_string(&state_file) {
                return serde_json::from_str(&json).ok();
            }
        }
        None
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut app = App::new();

        // Try to load existing state
        if let Some(saved_state) = App::load_state() {
            app.pet = saved_state;
        }

        (
            app,
            Command::batch(vec![
                Command::perform(App::load_sprites(app.asset_dir.clone()), |result| {
                    match result {
                        Ok(sprites) => Message::SpriteFramesLoaded(sprites),
                        Err(_) => Message::LoadPet(Engine::hatch("desktop-default")),
                    }
                }),
                Command::perform(async {}, |_| Message::UpdatePetState),
            ]),
        )
    }

    fn title(&self) -> String {
        format!("Open-Pets: {}", self.pet.species_name)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick => {
                // Update animation frame
                self.animation_frame = (self.animation_frame + 1) % 4; // 4 frames per state

                // Update mood based on state
                let mood = Engine::calculate_mood(&self.pet);

                // Check for speech bubble timeout
                if let Some(timer) = self.speech_timer {
                    if timer.elapsed() > Duration::from_secs(3) {
                        self.speech_bubble = None;
                        self.speech_timer = None;
                    }
                }

                Command::none()
            }
            Message::LoadPet(pet) => {
                self.pet = pet;
                Command::none()
            }
            Message::UpdatePetState => {
                // Periodically check OpenCode state (placeholder)
                Command::none()
            }
            Message::PetInteraction => {
                let current_mood = self.pet.mood.clone();
                self.pet.mood = "happy".to_string();
                Engine::award_xp(&mut self.pet, 5, "petting");

                if current_mood != "happy" {
                    self.show_speech(format!("Petting makes me {}!", self.pet.species_name));
                }

                Command::none()
            }
            Message::ShowSpeech(text) => {
                self.speech_bubble = Some(text);
                self.speech_timer = Some(std::time::Instant::now());
                Command::none()
            }
            Message::HideSpeech => {
                self.speech_bubble = None;
                self.speech_timer = None;
                Command::none()
            }
            Message::Exit => {
                self.save_state();
                window::get_current().map(|_| Message::Exit)
            }
            Message::WindowMoved(pos) => {
                self.position = pos;
                Command::none()
            }
            Message::SpriteFramesLoaded(sprites) => {
                self.sprites = sprites;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let current_state = if self.running { "idle" } else { "sleeping" };

        // Try to show sprite image, fallback to text
        let content: Element<Message> = if let Some(frames) = self.sprites.get(current_state) {
            if let Some(frame) = frames.get(self.animation_frame as usize % frames.len()) {
                iced::widget::image::Image::new(frame.clone())
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .into()
            } else {
                self.fallback_view()
            }
        } else {
            self.fallback_view()
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            // Animation timer
            iced::time::every(Duration::from_millis(self.animation_speed))
                .map(|_| Message::Tick),
        ])
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

impl App {
    fn fallback_view(&self) -> Element<Message> {
        let pet_emoji = match self.pet.species_id.as_str() {
            "void-cat" => "🐱",
            "code-hound" => "🐕",
            "terminal-turtle" => "🐢",
            "pixel-parrot" => "🦜",
            "debug-dragon" => "🐉",
            "rust-fox" => "🦊",
            _ => "🐾",
        };

        let text_content = format!(
            "{} {} (Lvl {})\n{}",
            pet_emoji,
            self.pet.species_name,
            self.pet.level,
            self.pet.mood
        );

        text(text_content)
            .size(14)
            .align_x(Horizontal::Center)
            .into()
    }

    fn show_speech(&mut self, text: String) {
        self.speech_bubble = Some(text);
        self.speech_timer = Some(std::time::Instant::now());
    }
}
