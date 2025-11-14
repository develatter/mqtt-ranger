///! MQTT client module for connecting and handling MQTT events.
///! This module provides functionality to connect to an MQTT broker
///! and process incoming messages.

use rumqttc::{MqttOptions, AsyncClient, EventLoop, QoS};

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