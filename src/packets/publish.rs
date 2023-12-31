//! MQTT [`PUBLISH`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718037)

use crate::{
    coding::{Length, Reader, Writer},
    error::MqttError,
};
use std::io::{Read, Write};

/// An MQTT [`PUBLISH` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718037)
#[derive(Debug, Clone)]
pub struct MqttPublish {
    /// Whether this packet is a redelivery or not
    dup: bool,
    /// The packet QoS
    qos: u8,
    /// Whether the message should be retained
    retain: bool,
    /// The message topic
    topic: String,
    /// The packet ID
    packet_id: Option<u16>,
    /// The payload
    payload: Vec<u8>,
}
impl MqttPublish {
    /// Creates a new packet
    pub fn new<T>(topic: T, retain: bool) -> Self
    where
        T: ToString,
    {
        Self { dup: false, qos: 0, retain, topic: topic.to_string(), packet_id: None, payload: Vec::new() }
    }
    /// Extends `self` to contain a non-zero QoS
    pub fn with_qos(mut self, qos: u8, packet_id: u16, dup: bool) -> Self {
        self.dup = dup;
        self.qos = qos;
        self.packet_id = Some(packet_id);
        self
    }
    /// Extends `self` to contain a non-empty payload
    pub fn with_payload<T>(mut self, payload: T) -> Self
    where
        T: Into<Vec<u8>>,
    {
        self.payload = payload.into();
        self
    }

    /// Whether this packet is a redelivery or not
    pub const fn dup(&self) -> bool {
        self.dup
    }
    /// The packet QoS
    pub const fn qos(&self) -> u8 {
        self.qos
    }
    /// Whether the message should be retained
    pub const fn retain(&self) -> bool {
        self.retain
    }
    /// The message topic
    pub fn topic(&self) -> &str {
        self.topic.as_ref()
    }
    /// The packet ID
    pub const fn packet_id(&self) -> Option<u16> {
        self.packet_id
    }
}
impl MqttPublish {
    /// The packet type constant
    pub const TYPE: u8 = 3;

    /// Reads `Self` from the given source
    pub fn read<T>(source: &mut T) -> Result<Self, MqttError>
    where
        T: Read,
    {
        // Read header:
        //  - header type and flags
        //  - packet len
        //  - topic
        //  - packet ID
        let mut reader = Reader::new(source);
        let [dup, qos0, qos1, retain] = reader.read_header(&Self::TYPE)?;
        let len = reader.read_packetlen()?;
        // Limit length
        let mut reader = reader.limit(len);
        let topic = reader.read_string()?;
        let packet_id = reader.read_optional_u16(qos0 || qos1)?;
        let payload = reader.read_remaining()?;

        // Init self
        let qos = ((qos0 as u8) << 1) | (qos1 as u8);
        Ok(Self { dup, qos, retain, topic, packet_id, payload })
    }

    /// Writes `self` into the given sink
    pub fn write<T>(self, sink: T) -> Result<T, MqttError>
    where
        T: Write,
    {
        // Assemble flags
        #[rustfmt::skip]
        let flags = [
            self.dup,
            (self.qos >> 1) != 0,
            (self.qos & 1) != 0,
            self.retain
        ];

        // Precompute body length:
        //  - header type and flags
        //  - packet len
        #[rustfmt::skip]
        let len = Length::new()
            .add_string(&self.topic)
            .add_optional_u16(&self.packet_id)
            .add_raw(&self.payload)
            .finalize();

        // Write header:
        //  - header type and flags
        //  - packet len
        //  - topic
        //  - packet ID
        Writer::new(sink)
            .write_header(Self::TYPE, flags)?
            .write_packetlen(len)?
            .write_string(self.topic)?
            .write_optional_u16(self.packet_id)?
            .write_raw(self.payload)?
            .finalize()
    }
}
