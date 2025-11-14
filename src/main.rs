///! Main entry point for mqtt-ranger. 
///! This application connects to an MQTT broker, subscribes to all topics,
///! and displays incoming messages in a terminal user interface (TUI).

use std::sync::Arc;
use std::sync::Mutex;

pub mod app;
pub mod tui;
pub mod mqtt;

use app::{AppState as App, run_app};



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the application state.
    let app = Arc::new(Mutex::new(App::new()));

    // Initialize the terminal UI.
    let mut terminal = tui::init_terminal().unwrap();

    // Run the MQTT client.
    mqtt::run(app.clone()).await?;

    // Run the TUI application.
    run_app(&mut terminal, app)?;
    
    // Restore the terminal to its original state.
    tui::restore_terminal(&mut terminal)?;

    Ok(())
}

