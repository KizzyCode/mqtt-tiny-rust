[![License BSD-2-Clause](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![AppVeyor CI](https://ci.appveyor.com/api/projects/status/github/KizzyCode/mqtt-tiny-rust?svg=true)](https://ci.appveyor.com/project/KizzyCode/mqtt-tiny-rust)
[![docs.rs](https://docs.rs/mqtt-tiny/badge.svg)](https://docs.rs/mqtt-tiny)
[![crates.io](https://img.shields.io/crates/v/mqtt-tiny.svg)](https://crates.io/crates/mqtt-tiny)
[![Download numbers](https://img.shields.io/crates/d/mqtt-tiny.svg)](https://crates.io/crates/mqtt-tiny)
[![dependency status](https://deps.rs/crate/mqtt-tiny/latest/status.svg)](https://deps.rs/crate/mqtt-tiny)

# `mqtt-tiny`
Welcome to `mqtt-tiny` 🎉

`mqtt-tiny` is a tiny, `no-std`-compatible
[MQTT 3.1.1](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html) codec implementation. It is currently
limited to packet en- and decoding, and does not handle state or transport-level stuff.

## Example
```rust ignore
use mqtt_tiny::{
    packets::{ToWriter, TryFromReader},
    Connack, Connect, Disconnect,
};
use std::{net::TcpStream, thread, time::Duration};

// Connect to a server
let mut connection = TcpStream::connect("127.0.0.1:1883").expect("failed to connect to server");
Connect::new(30, true, b"mqtttinyexamplesconnect").expect("failed to create CONNECT packet")
    .write(&mut connection).expect("failed to send CONNECT packet");

// Await CONNACK
let connack = Connack::try_read(&mut connection).expect("failed to read CONNACK packet");
assert_eq!(connack.return_code(), 0, "connection was refused");

// Sleep 3s
const PAUSE: Duration = Duration::from_secs(3);
thread::sleep(PAUSE);

// Disconnect
Disconnect::new().write(&mut connection).expect("failed to write DISCONNECT packet");
```

## Storage Backings
You can configure different predefined storage backings via feature flags:
- `std::vec::Vec` via the `std` feature flag
- `heapless::Vec` via the `heapless` feature flag
- `arrayvec::ArrayVec` via the `arrayvec` feature flag

Please note that the different predefined backings are mutually exclusive.
