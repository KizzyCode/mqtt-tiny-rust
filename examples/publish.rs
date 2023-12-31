//! Connects as client to an MQTT server, registers itself, publishes the current datetime under
//! `mqtttinyexamplespublish/date` and disconnects itself gracefully

use mqtt_tiny::packets::{
    connack::MqttConnack, connect::MqttConnect, disconnect::MqttDisconnect, puback::MqttPuback, publish::MqttPublish,
};
use std::{env, net::TcpStream, time::UNIX_EPOCH};

pub fn main() {
    // Connect to a server
    let address = env::var("MQTT_ADDRESS").unwrap_or("127.0.0.1:1883".into());
    let mut connection = TcpStream::connect(address).expect("failed to connect to server");

    // Build connect packet
    const CLIENT_ID: &str = "mqtttinyexamplesconnect";
    let mut connect = MqttConnect::new(30, true, CLIENT_ID);
    if let Ok(username) = env::var("MQTT_USERNAME") {
        connect = connect.with_username(username)
    }
    if let Ok(password) = env::var("MQTT_PASSWORD") {
        connect = connect.with_password(password)
    }

    // Connect
    connect.write(&mut connection).expect("failed to write CONNECT packet");
    let connack = MqttConnack::read(&mut connection).expect("failed to read CONNACK packet");
    assert_eq!(connack.return_code(), 0, "connection was refused");

    // Build publish packet
    const TOPIC: &str = "mqtttinyexamplespublish/date";
    let unix_time = UNIX_EPOCH.elapsed().expect("failed to get unix timestamp");
    let packet_id = unix_time.as_nanos() as u16;
    let timestamp = format!("{}-unixtime", unix_time.as_secs());
    let publish = MqttPublish::new(TOPIC, false).with_qos(1, packet_id, false).with_payload(timestamp);

    // Publish topic
    publish.write(&mut connection).expect("failed to write PUBLISH packet");
    let puback = MqttPuback::read(&mut connection).expect("failed to read PUBACK packet");
    assert_eq!(puback.packet_id(), packet_id, "invalid packed ID for PUBACK packet");

    // Disconnect
    let disconnect = MqttDisconnect::new();
    disconnect.write(connection).expect("failed to write DISCONNECT packet");
}
