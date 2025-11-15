///! mqtt-ranger: A terminal-based MQTT client with TUI interface.
///! Connects to an MQTT broker, subscribes to topics,
///! and displays incoming messages in a user-friendly terminal UI.

use std::sync::{Arc, Mutex};

pub mod app;
pub mod mqtt;
pub mod tui;

use app::{AppState};
use crate::tui::config_form::ConfigFormScreen;
use crate::tui::splash::SplashScreen;
use crate::tui::Screen;
use crate::tui::topic_activity::TopicActivityScreen;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_state = Arc::new(Mutex::new(AppState::new()));

    let mut terminal = tui::init_terminal()?;

    let mut splash_screen = SplashScreen::new(&mut terminal);
    splash_screen.run()?;
    
    let mut config_screen = ConfigFormScreen::new(&mut terminal);
    config_screen.run()?; 

    let config = config_screen.into_config().expect("No config produced");

    if let Err(e) = mqtt::run(app_state.clone(), config).await {
        let _ = tui::restore_terminal(&mut terminal);

        eprintln!("MQTT Error: {}", e);

        return Ok(());
    }

    let mut topic_activity_screen = TopicActivityScreen::new(&mut terminal, app_state);
    let res = topic_activity_screen.run();

    let _ = tui::restore_terminal(&mut terminal);

    if let Err(e) = res {
        eprintln!("Application error: {}", e);
    }

    Ok(())
}
