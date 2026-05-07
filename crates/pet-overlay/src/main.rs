//! Pet Overlay - Iced GUI application for open-pets desktop companion

mod app;

use app::{App, Message};

pub fn main() -> iced::Result {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    log::info!("Starting pet-overlay...");

    // Create default app state
    let default_state = App::new();

    // Configure window settings
    let settings = iced::Settings {
        window: iced::window::Settings {
            size: iced::Size::new(128.0, 128.0),
            position: iced::window::Position::Specific(iced::Point::new(50.0, 50.0)),
            decorations: false,
            transparent: true,
            always_on_top: true,
            resizable: false,
            icon: None,
            ..Default::default()
        },
        ..Default::default()
    };

    App::run(settings)
}
