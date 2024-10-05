//! MQTT [`SUBSCRIBE`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718063)

use crate::{
    anyvec::AnyVec,
    coding::{
        encoder::{PacketLenIter, TopicsQosIter, U16Iter, U8Iter, Unit},
        length::Length,
        Decoder, Encoder,
    },
    packets::TryFromIterator,
};
use core::{iter::Chain, marker::PhantomData};

/// An MQTT [`SUBSCRIBE` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718063)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subscribe<Seq, Bytes> {
    /// The packet ID
    packet_id: u16,
    /// A list of `(topic, qos)`-tuples
    ///
    /// # QoS Levels
    /// Valid QoS levels are:
    ///  - `0`: At most one delivery
    ///  - `1`: At least one delivery
    ///  - `2`: Exactly one delivery
    topics_qos: Seq,
    /// The byte vector type
    _vec: PhantomData<Bytes>,
}
impl<Seq, Bytes> Subscribe<Seq, Bytes>
where
    Seq: AnyVec<(Bytes, u8)>,
    Bytes: AnyVec<u8>,
{
    /// The packet type constant
    pub const TYPE: u8 = 8;

    /// Creates a new packet
    ///
    /// # QoS Levels
    /// Valid QoS levels are:
    ///  - `0`: At most one delivery
    ///  - `1`: At least one delivery
    ///  - `2`: Exactly one delivery
    pub fn new<S, T>(packet_id: u16, topics: S) -> Result<Self, &'static str>
    where
        S: IntoIterator<Item = (T, u8)>,
        T: AsRef<[u8]>,
    {
        // Collect all topic-qos pairs
        let mut topics_qos = Seq::default();
        for (topic, qos) in topics {
            // Copy topic and append pair
            let topic = Bytes::new(topic.as_ref())?;
            topics_qos.push((topic, qos))?;
        }

        // Init self
        Ok(Self { packet_id, topics_qos, _vec: PhantomData })
    }

    /// The packet ID
    pub fn packet_id(&self) -> u16 {
        self.packet_id
    }

    /// A list of `(topic, qos)`-tuples
    pub fn topics_qos(&self) -> &Seq {
        &self.topics_qos
    }
}
impl<Seq, Bytes> TryFromIterator for Subscribe<Seq, Bytes>
where
    Seq: AnyVec<(Bytes, u8)>,
    Bytes: AnyVec<u8>,
{
    fn try_from_iter<T>(iter: T) -> Result<Self, &'static str>
    where
        T: IntoIterator<Item = u8>,
    {
        // Read packet:
        //  - header type and `2` flags
        //  - packet len
        //  - packet ID
        //  - sequence
        //     - topic filter
        //     - qos
        let mut decoder = Decoder::new(iter);
        let (Self::TYPE, [false, false, true, false]) = decoder.header()? else {
            return Err("Invalid packet type/header");
        };
        // Limit length and make decoder peekable
        let len = decoder.packetlen()?;
        let mut decoder = decoder.limit(len).peekable();
        // Read fields
        let packet_id = decoder.u16()?;
        let topics_qos = decoder.topics_qos()?;

        // Init self
        Ok(Self { packet_id, topics_qos, _vec: PhantomData })
    }
}
impl<Seq, Bytes> IntoIterator for Subscribe<Seq, Bytes>
where
    Seq: AnyVec<(Bytes, u8)>,
    Bytes: AnyVec<u8>,
{
    type Item = u8;
    #[rustfmt::skip]
    type IntoIter =
        // Complex iterator built out of the individual message fields
        Chain<Chain<Chain<Chain<
            // - header type and `2` flags
            Unit, U8Iter>,
            // - packet len
            PacketLenIter>,
            // - packet ID
            U16Iter>,
            // - sequence
            //    - topic filter
            //    - qos
            TopicsQosIter<Seq, Bytes>>;

    fn into_iter(self) -> Self::IntoIter {
        // Precompute body length:
        //  - packet ID
        //  - sequence
        //     - topic filter
        //     - qos
        #[rustfmt::skip]
        let len = Length::new()
            .u16(&self.packet_id)
            .topics_qos(&self.topics_qos)
            .into();

        // Write packet:
        //  - header type and `2` flags
        //  - packet len
        //  - packet ID
        //  - sequence
        //     - topic filter
        //     - qos
        Encoder::default()
            .header(Self::TYPE, [false, false, true, false])
            .packetlen(len)
            .u16(self.packet_id)
            .topics_qos(self.topics_qos)
            .into_iter()
    }
}
