///! MQTT client module for connecting and handling MQTT events.
///! This module provides functionality to connect to an MQTT broker
///! and process incoming messages.

use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, QoS};
use time::{OffsetDateTime, UtcOffset, format_description::parse};
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};

use crate::app;

const MQTT_TIMESTAMP_FORMAT: &str = "[year]-[month]-[day] [hour]:[minute]:[second]";

/// Represents an MQTT event containing a topic and its associated payload.
#[derive(Debug)]
pub struct MQTTEvent {
    pub(crate) topic: String,
    pub(crate) payload: String,
    pub(crate) timestamp: time::OffsetDateTime,
}

/// Wrapper struct that represents an MQTT client with its associated event loop.
pub struct MQTTClient{
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
    MQTTClient {
        client,
        event_loop,
    }
}

/// Runs the MQTT client, subscribes to all topics, and processes incoming messages.
pub async fn run(app: Arc<Mutex<app::AppState>>, config: MQTTConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mqtt_client = configure_mqtt_client(
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

/// Configures the MQTT client by subscribing to all topics.
async fn configure_mqtt_client(host: &str, port: u16) -> Result<MQTTClient, Box<dyn std::error::Error>> {
    let mqtt_client = create_mqtt_client(host, port);

    if let Err(e) = mqtt_client.client.subscribe("#", QoS::AtMostOnce).await {
        return Err(Box::new(e));
    }
    Ok(mqtt_client)
}

/// Handles incoming MQTT messages and sends them through a channel.
fn handle_incoming_messages(mut mqtt_client: MQTTClient, tx: mpsc::Sender<MQTTEvent>) {
    tokio::spawn(async move {
        while let Ok(notification) = mqtt_client.event_loop.poll().await {
            if let Event::Incoming(incoming) = notification {
                if let rumqttc::Packet::Publish(publish) = incoming {
                    let topic = publish.topic;
                    let payload = String::from_utf8_lossy(&publish.payload).to_string();
                    let timestamp =  OffsetDateTime::now_local()
                    .unwrap_or(OffsetDateTime::now_utc().to_offset(
                        UtcOffset::current_local_offset().unwrap()
                    ));

                    let _ = tx.send(
                        MQTTEvent { 
                            topic, 
                            payload, 
                            timestamp 
                        }
                    ).await;
                }
            }
        }
    });
}

/// Updates the application state with incoming MQTT messages received through a channel.
fn update_app_state(app: Arc<Mutex<app::AppState>>, mut rx: mpsc::Receiver<MQTTEvent>) {
    tokio::spawn(async move {
        while let Some(mqtt_event) = rx.recv().await {
            let topic_name = mqtt_event.topic;
            let payload = mqtt_event.payload;
            
            let mut app_lock = app.lock().unwrap();
            let topic = app_lock.topics.iter_mut().find(|t| t.name == topic_name);
            let date_format: Vec<time::format_description::BorrowedFormatItem<'_>> = parse(MQTT_TIMESTAMP_FORMAT).unwrap();
            let timestamp = mqtt_event.timestamp.format(&date_format).unwrap();

            if let Some(t) = topic {
                t.messages.push(app::MessageActivity {
                    payload: payload.clone(),
                    timestamp: timestamp.clone(),
                });
            } else {
                app_lock.topics.push(app::TopicActivity {
                    name: topic_name,
                    messages: vec![app::MessageActivity {
                        payload: payload.clone(),
                        timestamp: timestamp.clone(),
                    }],
                });
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mqtt_config_with_valid_host_and_port() {
        let config = MQTTConfig {
            host: "localhost".to_string(),
            port: 1883,
        };
        
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 1883);
    }

    #[test]
    fn test_mqtt_config_with_different_valid_port() {
        let config = MQTTConfig {
            host: "broker.hivemq.com".to_string(),
            port: 8883,
        };
        
        assert_eq!(config.host, "broker.hivemq.com");
        assert_eq!(config.port, 8883);
    }

    #[test]
    fn test_reject_invalid_port_string() {
        // Test that parsing invalid port strings fails
        let invalid_ports = vec!["abc", "12.34", "-100", "70000"];
        
        for invalid_port in invalid_ports {
            let result = invalid_port.parse::<u16>();
            assert!(result.is_err(), "Port '{}' should be invalid", invalid_port);
        }
    }

    #[test]
    fn test_reject_negative_port() {
        // u16 cannot hold negative values, so parsing should fail
        let result = "-1".parse::<u16>();
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_port_exceeding_u16_max() {
        // Port value exceeding u16 max (65535) should fail
        let result = "70000".parse::<u16>();
        assert!(result.is_err());
    }

    #[test]
    fn test_accept_valid_numeric_ports() {
        // Test valid port numbers
        let valid_ports = vec!["0", "1883", "8883", "65535"];
        
        for valid_port in valid_ports {
            let result = valid_port.parse::<u16>();
            assert!(result.is_ok(), "Port '{}' should be valid", valid_port);
        }
    }
}