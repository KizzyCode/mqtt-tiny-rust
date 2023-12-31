[![License BSD-2-Clause](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![AppVeyor CI](https://ci.appveyor.com/api/projects/status/github/KizzyCode/mqtt-tiny-rust?svg=true)](https://ci.appveyor.com/project/KizzyCode/mqtt-tiny-rust)
[![docs.rs](https://docs.rs/mqtt-tiny/badge.svg)](https://docs.rs/mqtt-tiny)
[![crates.io](https://img.shields.io/crates/v/mqtt-tiny.svg)](https://crates.io/crates/mqtt-tiny)
[![Download numbers](https://img.shields.io/crates/d/mqtt-tiny.svg)](https://crates.io/crates/mqtt-tiny)
[![dependency status](https://deps.rs/crate/mqtt-tiny/latest/status.svg)](https://deps.rs/crate/mqtt-tiny)

# `mqtt-tiny`
Welcome to `mqtt-tiny` 🎉

`mqtt-tiny` is a tiny [MQTT 3.1.1](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html) codec
implementation. It is limited to packet en-/decoding, and does not handle state or transport-level stuff.

## Example
```rust
use mqtt_tiny::packets::connect::MqttConnect;

// Create a simple connect packet
const CLIENT_ID: &str = "mqtttinyreadmeexample";
let connect = MqttConnect::new(
    30,        //keep_alive_secs: u16,
    true,      //clean_session: bool,
    CLIENT_ID, //client_id: impl ToString,
);

// Serialize the connect packet
let connect_bytes = connect.write(Vec::new()).unwrap();
assert_eq!(connect_bytes.len(), 35);
```

### Running provided examples
To run the provided example, just set the `MQTT_ADDRESS`, `MQTT_USERNAME` and `MQTT_PASSWORD` environment variables
respectively to connect to your server.
