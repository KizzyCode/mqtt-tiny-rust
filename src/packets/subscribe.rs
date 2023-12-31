//! MQTT [`SUBSCRIBE`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718063)

use crate::{
    coding::{Length, Reader, Writer},
    error::MqttError,
};
use std::io::{Read, Write};

/// An MQTT [`SUBSCRIBE` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718063)
#[derive(Debug, Clone)]
pub struct MqttSubscribe {
    /// The packet ID
    packet_id: u16,
    /// A list of `(topic, qos)`-tuples
    topics: Vec<(String, u8)>,
}
impl MqttSubscribe {
    /// Creates a new packet
    pub fn new<T>(packet_id: u16, topics: Vec<(T, u8)>) -> Self
    where
        T: ToString,
    {
        // Convert all topics to owned strings
        let topics = topics.into_iter().map(|(topic, qos)| (topic.to_string(), qos)).collect();
        Self { packet_id, topics }
    }

    /// The packet ID
    pub const fn packet_id(&self) -> u16 {
        self.packet_id
    }
    /// A list of `(filter, qos)`-tuples
    pub const fn filters(&self) -> &Vec<(String, u8)> {
        &self.topics
    }
}
impl MqttSubscribe {
    /// The packet type constant
    pub const TYPE: u8 = 8;

    /// Reads `Self` from the given source
    pub fn read<T>(source: &mut T) -> Result<Self, MqttError>
    where
        T: Read,
    {
        // Read header:
        //  - header type and `2` flags
        //  - packet len
        //  - packet ID
        //  - sequence
        //     - topic filter
        //     - qos
        let mut reader = Reader::new(source);
        let _ = reader.read_header(&Self::TYPE)?;
        let len = reader.read_packetlen()?;
        // Limit length
        let mut reader = reader.limit(len).buffered();
        let packet_id = reader.read_u16()?;
        let topics = reader.read_topic_qos_seq()?;

        // Init self
        Ok(Self { packet_id, topics })
    }

    /// Writes `self` into the given sink
    pub fn write<T>(self, sink: T) -> Result<T, MqttError>
    where
        T: Write,
    {
        // Precompute body length:
        //  - packet ID
        //  - sequence
        //     - topic filter
        //     - qos
        #[rustfmt::skip]
        let len = Length::new()
            .add_u16(&self.packet_id)
            .add_topic_qos_seq(&self.topics)
            .finalize();

        // Write header:
        //  - header type and `2` flags
        //  - packet len
        //  - packet ID
        //  - sequence
        //     - topic filter
        //     - qos
        Writer::new(sink)
            .write_header(Self::TYPE, [false, false, true, false])?
            .write_packetlen(len)?
            .write_u16(self.packet_id)?
            .write_topic_qos_seq(self.topics)?
            .finalize()
    }
}
