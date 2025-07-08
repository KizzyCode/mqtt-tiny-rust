//! Connects as client to an MQTT server, registers itself and disconnects itself gracefully after 3 seconds

#[cfg(feature = "std")]
pub fn main() {
    use mqtt_tiny::packets::{ToWriter, TryFromReader};
    use mqtt_tiny::{Connack, Connect, Disconnect};
    use std::net::TcpStream;
    use std::thread;
    use std::time::Duration;

    // Connect to a server
    let mut connection = TcpStream::connect("127.0.0.1:1883").expect("failed to connect to server");

    // Build CONNECT packet...
    Connect::new(30, true, b"mqtttinyexamplesconnect").expect("failed to create CONNECT packet")
        // ...and connect
        .write(&mut connection).expect("failed to send CONNECT packet");
    let connack = Connack::try_read(&mut connection).expect("failed to read CONNACK packet");
    assert_eq!(connack.return_code(), 0, "connection was refused");

    // Sleep 10s
    const PAUSE: Duration = Duration::from_secs(3);
    thread::sleep(PAUSE);

    // Disconnect
    Disconnect::new().write(&mut connection).expect("failed to write DISCONNECT packet");
}

#[cfg(not(feature = "std"))]
pub fn main() {
    panic!("Example requires the `std`-feature");
}
