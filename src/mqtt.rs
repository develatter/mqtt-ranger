use std::sync::{Arc, Mutex};

///! MQTT client module for connecting and handling MQTT events.
///! This module provides functionality to connect to an MQTT broker
///! and process incoming messages.

use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, QoS};
use tokio::sync::mpsc;

use crate::app;

///! Represents an MQTT event containing a topic and its associated payload.
#[derive(Debug)]
pub struct MQTTEvent {
    pub(crate) topic: String,
    pub(crate) payload: String,
}

///! Wrapper struct that represents an MQTT client with its associated event loop.
pub struct MQTTClient{
    pub(crate) client: AsyncClient,
    pub(crate) event_loop: EventLoop,
}

#[derive(Debug, Clone)]
pub struct MQTTConfig {
    pub id: String,
    pub host: String,
    pub port: u16,
}

///! Connects to an MQTT broker and returns an MQTTClient instance.
pub fn connect_mqtt(id: &str, host: &str, port: u16) -> MQTTClient {
    let mut mqttoptions = MqttOptions::new(id, host, port);
    mqttoptions.set_keep_alive(std::time::Duration::from_secs(5));

    let (client, event_loop) = AsyncClient::new(mqttoptions, 10);
    MQTTClient {
        client,
        event_loop,
    }
}

///! Runs the MQTT client, subscribes to all topics, and processes incoming messages.
pub async fn run(app: Arc<Mutex<app::AppState>>, config: MQTTConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mqtt_client = configure_mqtt_client(
        &config.id, 
        &config.host, 
        config.port
    ).await?;

    let (tx, rx) = mpsc::channel::<MQTTEvent>(100);

    // Spawn a task to handle incoming MQTT messages.
    handle_incoming_messages(mqtt_client, tx);

    // Spawn a task to update the application state with incoming MQTT messages.
    update_app_state(Arc::clone(&app), rx);
    
    Ok(())
}

async fn configure_mqtt_client(id: &str, host: &str, port: u16) -> Result<MQTTClient, Box<dyn std::error::Error>> {
    let mqtt_client = connect_mqtt(id, host, port);
    mqtt_client.client.subscribe("#", QoS::AtMostOnce).await.unwrap();
    Ok(mqtt_client)
}

///! Handles incoming MQTT messages and sends them through a channel.
fn handle_incoming_messages(mut mqtt_client: MQTTClient, tx: mpsc::Sender<MQTTEvent>) {
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
}

///! Updates the application state with incoming MQTT messages received through a channel.
fn update_app_state(app: Arc<Mutex<app::AppState>>, mut rx: mpsc::Receiver<MQTTEvent>) {
    tokio::spawn(async move {
        while let Some(mqtt_event) = rx.recv().await {
            let topic_name = mqtt_event.topic;
            let payload = mqtt_event.payload;

            let mut app_lock = app.lock().unwrap();
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
}