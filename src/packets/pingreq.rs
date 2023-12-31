//! MQTT [`PINGREQ`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718081)

use crate::{
    coding::{Reader, Writer},
    error::MqttError,
};
use std::io::{Read, Write};

/// An MQTT [`PINGREQ` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718081)
#[derive(Debug, Clone)]
pub struct MqttPingreq {
    _private: (),
}
impl MqttPingreq {
    /// Creates a new packet
    pub const fn new() -> Self {
        Self { _private: () }
    }
}
impl MqttPingreq {
    /// The packet type constant
    pub const TYPE: u8 = 12;

    /// For this packet, the body length is fixed
    const BODY_LEN: [u8; 1] = [0];

    /// Reads `Self` from the given source
    pub fn read<T>(source: &mut T) -> Result<Self, MqttError>
    where
        T: Read,
    {
        // Read header:
        //  - header type and `0` flags
        //  - packet len
        let mut reader = Reader::new(source);
        let _ = reader.read_header(&Self::TYPE)?;
        let _ = reader.read_constant(&Self::BODY_LEN)?;

        // No body to read; init self
        Ok(Self { _private: () })
    }

    /// Writes `self` into the given sink
    pub fn write<T>(self, sink: T) -> Result<T, MqttError>
    where
        T: Write,
    {
        // Write header:
        //  - header type and `0` flags
        //  - packet len
        #[rustfmt::skip]
        return Writer::new(sink)
            .write_header(Self::TYPE, [false, false, false, false])?
            .write_array(Self::BODY_LEN)?
            .finalize();
    }
}
