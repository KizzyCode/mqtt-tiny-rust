//! Connects as client to an MQTT server, registers itself and disconnects itself gracefully after a few seconds

use mqtt_tiny::packets::{connack::MqttConnack, connect::MqttConnect, disconnect::MqttDisconnect};
use std::{env, net::TcpStream, thread, time::Duration};

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

    // Sleep 10s
    const PAUSE: Duration = Duration::from_secs(3);
    thread::sleep(PAUSE);

    // Disconnect
    let disconnect = MqttDisconnect::new();
    disconnect.write(connection).expect("failed to write DISCONNECT packet");
}
