use rumqttc::{MqttOptions, AsyncClient, EventLoop, QoS};

#[derive(Debug)]
pub struct MQTTEvent {
    pub(crate) topic: String,
    pub(crate) payload: String,
}

pub struct MQTTClient{
    pub(crate) client: AsyncClient,
    pub(crate) event_loop: EventLoop,
}

pub fn connect_mqtt(id: &str, host: &str, port: u16) -> MQTTClient {
    let mut mqttoptions = MqttOptions::new(id, host, port);
    mqttoptions.set_keep_alive(std::time::Duration::from_secs(5));

    let (client, event_loop) = AsyncClient::new(mqttoptions, 10);
    MQTTClient {
        client,
        event_loop,
    }
}