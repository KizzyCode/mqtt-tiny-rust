//! MQTT [`CONNACK`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718033)

use crate::{
    coding::{Reader, Writer},
    error::MqttError,
};
use std::io::{Read, Write};

/// An MQTT [`CONNACK` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718033)
#[derive(Debug, Clone)]
pub struct MqttConnack {
    /// The acknowledgement flag
    ack_flag: bool,
    /// The return code
    return_code: u8,
}
impl MqttConnack {
    /// Creates a new packet
    pub const fn new(ack_flag: bool, return_code: u8) -> Self {
        Self { ack_flag, return_code }
    }

    /// The acknowledgement flag
    pub const fn ack_flag(&self) -> bool {
        self.ack_flag
    }
    /// The return code
    pub const fn return_code(&self) -> u8 {
        self.return_code
    }
}
impl MqttConnack {
    /// The packet type constant
    pub const TYPE: u8 = 2;

    /// For this packet, the body length is fixed
    const BODY_LEN: [u8; 1] = [2];

    /// Reads `Self` from the given source
    pub fn read<T>(source: &mut T) -> Result<Self, MqttError>
    where
        T: Read,
    {
        // Read packet:
        //  - header type and `0` flags
        //  - packet len
        //  - ACK flag
        //  - return code
        let mut reader = Reader::new(source);
        let _ = reader.read_header(&Self::TYPE)?;
        let _ = reader.read_constant(&Self::BODY_LEN)?;
        let [_, _, _, _, _, _, _, ack_flag] = reader.read_flags()?;
        let return_code = reader.read_u8()?;

        // Init self
        Ok(Self { ack_flag, return_code })
    }

    /// Writes `self` into the given sink
    pub fn write<T>(self, sink: T) -> Result<T, MqttError>
    where
        T: Write,
    {
        // Write packet:
        //  - header type and `0` flags
        //  - packet len
        //  - ACK flag
        //  - return code
        Writer::new(sink)
            .write_header(Self::TYPE, [false, false, false, false])?
            .write_array(Self::BODY_LEN)?
            .write_flags([false, false, false, false, false, false, false, self.ack_flag])?
            .write_u8(self.return_code)?
            .finalize()
    }
}
