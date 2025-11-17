///! mqtt-ranger: A terminal-based MQTT client with TUI interface.
///! Connects to an MQTT broker, subscribes to topics,
///! and displays incoming messages in a user-friendly terminal UI.

use std::sync::{Arc, Mutex};

pub mod app;
pub mod mqtt;
pub mod tui;

use app::{TopicActivityMenuState};
use crate::tui::config_form::ConfigFormScreen;
use crate::tui::splash::SplashScreen;
use crate::tui::Screen;
use crate::tui::topic_activity::TopicActivityScreen;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let topic_activity_menu_state = Arc::new(Mutex::new(TopicActivityMenuState::new()));

    let mut terminal = tui::init_terminal()?;

    let mut splash_screen = SplashScreen::new(&mut terminal);
    splash_screen.run()?;
    
    let mut config_screen = ConfigFormScreen::new(&mut terminal);
    if let Err(e) = config_screen.run() {
        let _ = tui::restore_terminal(&mut terminal);
        eprintln!("Config form cancelled: {}", e);
        return Ok(());
    }

    let config = match config_screen.into_config() {
        Some(cfg) => cfg,
        None => {
            let _ = tui::restore_terminal(&mut terminal);
            eprintln!("No config produced");
            return Ok(());
        }
    };

    if let Err(e) = mqtt::run(topic_activity_menu_state.clone(), config).await {
        let _ = tui::restore_terminal(&mut terminal);

        eprintln!("MQTT Error: {}", e);

        return Ok(());
    }

    let mut topic_activity_screen = TopicActivityScreen::new(&mut terminal, topic_activity_menu_state);
    let res = topic_activity_screen.run();

    let _ = tui::restore_terminal(&mut terminal);

    if let Err(e) = res {
        eprintln!("Application error: {}", e);
    }

    Ok(())
}
