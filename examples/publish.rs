//! Connects as client to an MQTT server, registers itself, publishes the current datetime under
//! `mqtttinyexamplespublish/date` and disconnects itself gracefully

#[cfg(feature = "std")]
pub fn main() {
    use mqtt_tiny::{
        packets::{ToWriter, TryFromReader},
        Connack, Connect, Disconnect, Puback, Publish,
    };
    use std::{net::TcpStream, time::UNIX_EPOCH};

    // Connect to a server
    let mut connection = TcpStream::connect("127.0.0.1:1883").expect("failed to connect to server");

    // Build connect packet...
    Connect::new(30, true, b"mqtttinyexamplesconnect").expect("failed to create CONNECT packet")
        // ...and connect
        .write(&mut connection).expect("failed to send CONNECT packet");
    let connack = Connack::try_read(&mut connection).expect("failed to read CONNACK packet");
    assert_eq!(connack.return_code(), 0, "connection was refused");

    // Prepare info for publish packet
    let unix_time = UNIX_EPOCH.elapsed().expect("failed to get unix timestamp");
    let packet_id = unix_time.as_nanos() as u16;
    let timestamp = format!("{}-unixtime", unix_time.as_secs());

    // Build PUBLISH packet...
    Publish::new(b"mqtttinyexamplespublish/date", timestamp.as_bytes(), false)
        .expect("failed to create PUBLISH packet")
        // ...and set QoS to 1, meaning we require an ACK...
        .with_qos(1, packet_id, false)
        // ...and publish message
        .write(&mut connection).expect("failed to write PUBLISH packet");
    let puback = Puback::try_read(&mut connection).expect("failed to read PUBACK packet");
    assert_eq!(puback.packet_id(), packet_id, "invalid packed ID for PUBACK packet");

    // Disconnect
    Disconnect::new().write(&mut connection).expect("failed to write DISCONNECT packet");
}

#[cfg(not(feature = "std"))]
pub fn main() {
    panic!("Example requires the `std`-feature");
}
