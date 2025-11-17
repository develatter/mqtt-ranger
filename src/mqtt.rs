///! MQTT client module for connecting and handling MQTT events.
///! This module provides functionality to connect to an MQTT broker
///! and process incoming messages.
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, QoS};
use std::sync::{Arc, Mutex};
use time::{OffsetDateTime, UtcOffset, format_description::parse};
use tokio::sync::mpsc;

use crate::app::{self, TopicActivityMenuState};

const MQTT_TIMESTAMP_FORMAT: &str = "[year]-[month]-[day] [hour]:[minute]:[second]";

/// Represents an MQTT event containing a topic and its associated payload.
#[derive(Debug)]
pub struct MQTTEvent {
    pub(crate) topic: String,
    pub(crate) payload: String,
    pub(crate) timestamp: time::OffsetDateTime,
}

/// Wrapper struct that represents an MQTT client with its associated event loop.
pub struct MQTTClient {
    pub(crate) client: AsyncClient,
    pub(crate) event_loop: EventLoop,
}

#[derive(Debug, Clone)]
pub struct MQTTConfig {
    pub host: String,
    pub port: u16,
}

/// Connects to an MQTT broker and returns an MQTTClient instance.
pub fn create_mqtt_client(host: &str, port: u16) -> MQTTClient {
    let mut mqttoptions = MqttOptions::new("mqtt-ranger", host, port);
    mqttoptions.set_keep_alive(std::time::Duration::from_secs(5));

    let (client, event_loop) = AsyncClient::new(mqttoptions, 10);
    
    MQTTClient { client, event_loop }
}

/// Runs the MQTT client, subscribes to all topics, and processes incoming messages.
pub async fn run(
    menu_state: Arc<Mutex<app::TopicActivityMenuState>>,
    config: MQTTConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mqtt_client = configure_mqtt_client(&config.host, config.port).await?;

    let (tx, rx) = mpsc::channel::<MQTTEvent>(100);

    spawn_message_handler(mqtt_client, tx);

    spawn_menu_updater(Arc::clone(&menu_state), rx);

    Ok(())
}

/// Configures the MQTT client by subscribing to all topics.
async fn configure_mqtt_client(
    host: &str,
    port: u16,
) -> Result<MQTTClient, Box<dyn std::error::Error>> {
    let mqtt_client = create_mqtt_client(host, port);

    if let Err(e) = mqtt_client.client.subscribe("#", QoS::AtMostOnce).await {
        return Err(Box::new(e));
    }
    Ok(mqtt_client)
}

// Spawn a task to handle incoming MQTT messages.
fn spawn_message_handler(mqtt_client: MQTTClient, tx: mpsc::Sender<MQTTEvent>) {
    tokio::spawn(async move { handle_incoming_messages(mqtt_client, tx) });
}

/// Handles incoming MQTT messages and sends them through a channel.
async fn handle_incoming_messages(mut mqtt_client: MQTTClient, tx: mpsc::Sender<MQTTEvent>) {
    while let Ok(notification) = mqtt_client.event_loop.poll().await {
        if let Event::Incoming(incoming) = notification {
            if let rumqttc::Packet::Publish(publish) = incoming {
                let topic = publish.topic;
                let payload = String::from_utf8_lossy(&publish.payload).to_string();
                let timestamp = OffsetDateTime::now_local().unwrap_or(
                    OffsetDateTime::now_utc().to_offset(UtcOffset::current_local_offset().unwrap()),
                );

                let _ = tx
                    .send(MQTTEvent {
                        topic,
                        payload,
                        timestamp,
                    })
                    .await;
            }
        }
    }
}

// Spawn a task to update the application state with incoming MQTT messages.
fn spawn_menu_updater(app: Arc<Mutex<app::TopicActivityMenuState>>, rx: mpsc::Receiver<MQTTEvent>) {
    tokio::spawn(async move {
        update_topic_menu_state(app, rx).await;
    });
}

/// Updates the application state with incoming MQTT messages received through a channel.
async fn update_topic_menu_state(
    menu_state: Arc<Mutex<app::TopicActivityMenuState>>,
    mut rx: mpsc::Receiver<MQTTEvent>,
) {
    while let Some(mqtt_event) = rx.recv().await {
        push_message_into_topic(&menu_state, mqtt_event);
    }
}

/// Receives a MQTTEvent, transforms it into a TopicActivity and push it into the topics
/// list of the MenuState.
fn push_message_into_topic(menu_state: &Arc<Mutex<TopicActivityMenuState>>, mqtt_event: MQTTEvent) {
    let topic_name = mqtt_event.topic;
    let payload = mqtt_event.payload;

    let mut menu_lock = menu_state.lock().unwrap();

    let topic = menu_lock.topics.iter_mut().find(|t| t.name == topic_name);
    let date_format: Vec<time::format_description::BorrowedFormatItem<'_>> =
        parse(MQTT_TIMESTAMP_FORMAT).unwrap();
    let timestamp = mqtt_event.timestamp.format(&date_format).unwrap();

    if let Some(t) = topic {
        t.messages.push(app::MessageActivity {
            payload: payload.clone(),
            timestamp: timestamp.clone(),
        });
    } else {
        menu_lock.topics.push(app::TopicActivity {
            name: topic_name,
            messages: vec![app::MessageActivity {
                payload: payload.clone(),
                timestamp: timestamp.clone(),
            }],
        });
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_add_topic_inserts_into_topics() {
        let topic_menu_state = Arc::new(Mutex::new(app::TopicActivityMenuState {
            topics: Vec::new(),
            selected_index: 0,
        }));

        let menu_guard: std::sync::MutexGuard<'_, TopicActivityMenuState> =
            topic_menu_state.lock().unwrap();
            
        assert_eq!(menu_guard.topics.len(), 0);
        drop(menu_guard);

        let mqtt_event = MQTTEvent {
            topic: "Topic1".into(),
            payload: "Payload 1".into(),
            timestamp: OffsetDateTime::now_utc(),
        };

        push_message_into_topic(&topic_menu_state, mqtt_event);
        let menu_guard = topic_menu_state.lock().unwrap();

        assert_eq!(menu_guard.topics.len(), 1);
    }

    #[test]
    fn test_message_is_stored_in_correct_topic() {
        let topic_menu_state = Arc::new(Mutex::new(app::TopicActivityMenuState {
            topics: Vec::new(),
            selected_index: 0,
        }));

        let mqtt_event_1 = MQTTEvent {
            topic: "test/topic1".into(),
            payload: "Payload 1!".into(),
            timestamp: OffsetDateTime::now_utc()
        };

        let mqtt_event_2 = MQTTEvent {
            topic: "test/topic2".into(),
            payload: "Payload 2!".into(),
            timestamp: OffsetDateTime::now_utc()
        };

        let mqtt_event_3 = MQTTEvent {
            topic: "topic3".into(),
            payload: "Payload 3!".into(),
            timestamp: OffsetDateTime::now_utc()
        };

        let mqtt_event_4 = MQTTEvent {
            topic: "topic3".into(),
            payload: "Payload 4!".into(),
            timestamp: OffsetDateTime::now_utc()
        };

        push_message_into_topic(&topic_menu_state, mqtt_event_1);
        push_message_into_topic(&topic_menu_state, mqtt_event_2);
        push_message_into_topic(&topic_menu_state, mqtt_event_3);
        push_message_into_topic(&topic_menu_state, mqtt_event_4);

        let menu_guard = topic_menu_state.lock().unwrap();

        assert_eq!(menu_guard.topics[0].messages.len(), 1);
        assert_eq!(menu_guard.topics[1].messages.len(), 1);
        assert_eq!(menu_guard.topics[2].messages.len(), 2);

        assert_eq!(menu_guard.topics[0].messages[0].payload, "Payload 1!");
        assert_eq!(menu_guard.topics[1].messages[0].payload, "Payload 2!");
        assert_eq!(menu_guard.topics[2].messages[0].payload, "Payload 3!");
        assert_eq!(menu_guard.topics[2].messages[1].payload, "Payload 4!");
    }
}
