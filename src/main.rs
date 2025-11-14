///! Main entry point for mqtt-ranger. 
///! This application connects to an MQTT broker, subscribes to all topics,
///! and displays incoming messages in a terminal user interface (TUI).

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::sync::Mutex;

pub mod app;
pub mod tui;
pub mod mqtt;

use app::{AppState as App, run_app};
use crossterm::{
    execute,
    terminal::{LeaveAlternateScreen, disable_raw_mode},
};
use rumqttc::{Event, QoS};
use tokio::sync::mpsc;

use crate::mqtt::MQTTEvent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = tui::init_terminal().unwrap();

    let tick_rate = Duration::from_millis(250);
    let app = Arc::new(Mutex::new(App::new()));

    let mut mqtt_client = mqtt::connect_mqtt("broker-mqtt", "localhost", 1883);
    mqtt_client.client.subscribe("#", QoS::AtMostOnce).await?;

    let (tx, mut rx) = mpsc::channel::<MQTTEvent>(100);

    // Spawn a task to handle incoming MQTT messages.
    tokio::spawn(async move {
        while let Ok(notification) = mqtt_client.event_loop.poll().await {
            if let Event::Incoming(incoming) = notification {
                if let rumqttc::Packet::Publish(publish) = incoming {
                    let topic = publish.topic;
                    let payload = String::from_utf8_lossy(&publish.payload).to_string();

                    let _ = tx.send(MQTTEvent { topic, payload }).await;
                }
            }
        }
    });

    // Spawn a task to update the application state with incoming MQTT messages.
    let app_clone = Arc::clone(&app);
    tokio::spawn(async move {
        while let Some(mqtt_event) = rx.recv().await {
            let topic_name = mqtt_event.topic;
            let payload = mqtt_event.payload;

            let mut app_lock = app_clone.lock().unwrap();
            let topic = app_lock.topics.iter_mut().find(|t| t.name == topic_name);
            if let Some(t) = topic {
                t.messages.push(payload);
            } else {
                app_lock.topics.push(app::TopicActivity {
                    name: topic_name,
                    messages: vec![payload],
                });
            }
        }
    });
    
    // Run the TUI application.
    run_app(&mut terminal, app, tick_rate)?;
    
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    let _ = terminal.show_cursor();
    Ok(())
}

