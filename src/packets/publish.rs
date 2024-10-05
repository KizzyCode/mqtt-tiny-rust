//! MQTT [`PUBLISH`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718037)

use crate::{
    anyvec::AnyVec,
    coding::{
        encoder::{BytesIter, OptionalU16Iter, PacketLenIter, U8Iter, Unit},
        length::Length,
        Decoder, Encoder,
    },
    packets::TryFromIterator,
};
use core::iter::Chain;

/// An MQTT [`PUBLISH` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718037)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Publish<Bytes> {
    /// Whether this packet is a redelivery or not
    dup: bool,
    /// The packet QoS
    ///
    /// # QoS Levels
    /// Valid QoS levels are:
    ///  - `0`: At most one delivery
    ///  - `1`: At least one delivery
    ///  - `2`: Exactly one delivery
    qos: u8,
    /// Whether the message should be retained
    retain: bool,
    /// The message topic
    topic: Bytes,
    /// The packet ID
    packet_id: Option<u16>,
    /// The payload
    payload: Bytes,
}
impl<Bytes> Publish<Bytes>
where
    Bytes: AnyVec<u8>,
{
    /// The packet type constant
    pub const TYPE: u8 = 3;

    /// Creates a new packet
    pub fn new<T, P>(topic: T, payload: P, retain: bool) -> Result<Self, &'static str>
    where
        T: AsRef<[u8]>,
        P: AsRef<[u8]>,
    {
        let topic = Bytes::new(topic.as_ref())?;
        let payload = Bytes::new(payload.as_ref())?;
        Ok(Self { dup: false, qos: 0, retain, topic, packet_id: None, payload })
    }
    /// Configures the packet quality-of-service level and specifies whether this packet is a duplicate transmission
    /// (aka retry) or not
    ///
    /// # QoS Levels
    /// Valid QoS levels are:
    ///  - `0`: At most one delivery
    ///  - `1`: At least one delivery
    ///  - `2`: Exactly one delivery
    pub fn with_qos(mut self, qos: u8, packet_id: u16, dup: bool) -> Self {
        self.dup = dup;
        self.qos = qos;
        self.packet_id = Some(packet_id);
        self
    }

    /// The message topic
    pub fn topic(&self) -> &[u8] {
        self.topic.as_ref()
    }

    /// The payload
    pub fn payload(&self) -> &[u8] {
        self.payload.as_ref()
    }

    /// Whether the message should be retained
    pub fn retain(&self) -> bool {
        self.retain
    }

    /// Whether this packet is a redelivery or not
    pub fn dup(&self) -> bool {
        self.dup
    }
    /// The packet QoS
    pub fn qos(&self) -> u8 {
        self.qos
    }
    /// The packet ID
    pub fn packet_id(&self) -> Option<u16> {
        self.packet_id
    }
}
impl<Bytes> TryFromIterator for Publish<Bytes>
where
    Bytes: AnyVec<u8>,
{
    fn try_from_iter<T>(iter: T) -> Result<Self, &'static str>
    where
        T: IntoIterator<Item = u8>,
    {
        // Read packet:
        //  - header type and flags
        //  - packet len
        //  - topic
        //  - packet ID
        let mut decoder = Decoder::new(iter);
        let (Self::TYPE, [dup, qos0, qos1, retain]) = decoder.header()? else {
            return Err("Invalid packet type");
        };
        // Limit length
        let len = decoder.packetlen()?;
        let mut decoder = decoder.limit(len);
        // Read fields
        let topic = decoder.bytes()?;
        let packet_id = decoder.optional_u16(qos0 || qos1)?;
        let payload = decoder.raw_remainder()?;

        // Init self
        let qos = ((qos0 as u8) << 1) | (qos1 as u8);
        Ok(Self { dup, qos, retain, topic, packet_id, payload })
    }
}
impl<Bytes> IntoIterator for Publish<Bytes>
where
    Bytes: AnyVec<u8>,
{
    type Item = u8;
    #[rustfmt::skip]
    type IntoIter =
        // Complex iterator built out of the individual message fields
        Chain<Chain<Chain<Chain<Chain<
            // - header type and flags
            Unit, U8Iter>,
            // - packet len
            PacketLenIter>,
            // - topic
            BytesIter<Bytes>>,
            // - packet ID
            OptionalU16Iter>,
            //  - payload
            <Bytes as IntoIterator>::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
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
        //  - payload
        #[rustfmt::skip]
        let len = Length::new()
            .bytes(&self.topic)
            .optional_u16(&self.packet_id)
            .raw(&self.payload)
            .into();

        // Write packet:
        //  - header type and flags
        //  - packet len
        //  - topic
        //  - packet ID
        //  - payload
        Encoder::default()
            .header(Self::TYPE, flags)
            .packetlen(len)
            .bytes(self.topic)
            .optional_u16(self.packet_id)
            .raw(self.payload)
            .into_iter()
    }
}
