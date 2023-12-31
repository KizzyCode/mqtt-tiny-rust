//! MQTT [`UNSUBSCRIBE`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718072)

use crate::{
    coding::{Length, Reader, Writer},
    error::MqttError,
};
use std::{io::Read, io::Write};

/// An MQTT [`UNSUBSCRIBE` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718072)
#[derive(Debug, Clone)]
pub struct MqttUnsubscribe {
    /// The packet ID
    packet_id: u16,
    /// A list of topic filters
    topics: Vec<String>,
}
impl MqttUnsubscribe {
    /// Creates a new `TODO` packet
    pub fn new<T>(packet_id: u16, topics: Vec<T>) -> Self
    where
        T: ToString,
    {
        // Convert all topics to owned strings
        let topics = topics.into_iter().map(|topic| topic.to_string()).collect();
        Self { packet_id, topics }
    }

    /// The packet ID
    pub const fn packet_id(&self) -> u16 {
        self.packet_id
    }

    /// A list of topic filters
    pub const fn topics(&self) -> &Vec<String> {
        &self.topics
    }
}
impl MqttUnsubscribe {
    /// The packet type constant
    pub const TYPE: u8 = 10;

    /// Reads `Self` from the given source
    pub fn read<T>(source: &mut T) -> Result<Self, MqttError>
    where
        T: Read,
    {
        // Read header:
        //  - header type and `2` flags
        //  - packet len
        //  - packed ID
        //  - sequence
        //     - topic filter
        let mut reader = Reader::new(source);
        let _ = reader.read_header(&Self::TYPE)?;
        let len = reader.read_packetlen()?;
        // Limit length
        let mut reader = reader.limit(len).buffered();
        let packet_id = reader.read_u16()?;
        let topics = reader.read_topic_seq()?;

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
        #[rustfmt::skip]
        let len = Length::new()
            .add_u16(&self.packet_id)
            .add_topic_seq(&self.topics)
            .finalize();

        // Write header:
        //  - header type and `2` flags
        //  - packet len
        //  - packed ID
        //  - sequence
        //     - topic filter
        Writer::new(sink)
            .write_header(Self::TYPE, [false, false, true, false])?
            .write_packetlen(len)?
            .write_u16(self.packet_id)?
            .write_topic_seq(self.topics)?
            .finalize()
    }
}
