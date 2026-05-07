//! Pet Overlay - Iced GUI application for open-pets desktop companion

mod app;

pub fn main() -> iced::Result {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    log::info!("Starting pet-overlay...");

    app::PetApp::run()
}
