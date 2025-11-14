///! Main entry point for mqtt-ranger.
///! This application connects to an MQTT broker, subscribes to all topics,
///! and displays incoming messages in a terminal user interface (TUI).
use std::sync::Arc;
use std::sync::Mutex;

pub mod app;
pub mod mqtt;
pub mod tui;

use app::{AppState as App, run_app};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the application state.
    let app = Arc::new(Mutex::new(App::new()));

    // Initialize the terminal UI.
    let mut terminal = tui::init_terminal().unwrap();

    let config = match tui::run_config_form(&mut terminal) {
        Ok(cfg) => cfg,
        Err(e) => {
            tui::restore_terminal(&mut terminal)?;
            println!("Error: {}", e);
            return Ok(());
        }
    };

    // Run the MQTT client.
    mqtt::run(app.clone(), config).await?;

    // Run the TUI application.
    run_app(&mut terminal, app)?;

    // Restore the terminal to its original state.
    tui::restore_terminal(&mut terminal)?;

    Ok(())
}
