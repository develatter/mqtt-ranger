///! mqtt-ranger: A terminal-based MQTT client with TUI interface.
///! Connects to an MQTT broker, subscribes to topics,
///! and displays incoming messages in a user-friendly terminal UI.

use std::sync::{Arc, Mutex};

pub mod app;
pub mod mqtt;
pub mod tui;

use app::{AppState as App};
use tui::run_topic_activity_screen;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Arc::new(Mutex::new(App::new()));

    let mut terminal = tui::init_terminal()?;

    let _ = tui::run_splash_screen(&mut terminal);

    let config = match tui::run_config_form_screen(&mut terminal) {
        Ok(cfg) => cfg,
        Err(e) => {
            let _ = tui::restore_terminal(&mut terminal);
            eprintln!("Error en configuración: {}", e);
            return Ok(());
        }
    };

    if let Err(e) = mqtt::run(app.clone(), config).await {
        let _ = tui::restore_terminal(&mut terminal);

        eprintln!("MQTT Error: {}", e);

        return Ok(());
    }

    let res = run_topic_activity_screen(&mut terminal, app);

    let _ = tui::restore_terminal(&mut terminal);

    if let Err(e) = res {
        eprintln!("Application error: {}", e);
    }

    Ok(())
}
