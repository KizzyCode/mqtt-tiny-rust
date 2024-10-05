//! MQTT [`UNSUBSCRIBE`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718072)

use crate::{
    anyvec::AnyVec,
    coding::{
        encoder::{PacketLenIter, TopicsIter, U16Iter, U8Iter, Unit},
        length::Length,
        Decoder, Encoder,
    },
    packets::TryFromIterator,
};
use core::{iter::Chain, marker::PhantomData};

/// An MQTT [`UNSUBSCRIBE` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718072)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unsubscribe<Seq, Bytes> {
    /// The packet ID
    packet_id: u16,
    /// A list of topic filters
    topics: Seq,
    /// The byte vector type
    _vec: PhantomData<Bytes>,
}
impl<Seq, Bytes> Unsubscribe<Seq, Bytes>
where
    Seq: AnyVec<Bytes>,
    Bytes: AnyVec<u8>,
{
    /// The packet type constant
    pub const TYPE: u8 = 10;

    /// Creates a new packet
    pub fn new<S, T>(packet_id: u16, topics: S) -> Result<Self, &'static str>
    where
        S: IntoIterator<Item = T>,
        T: AsRef<[u8]>,
    {
        // Collect all topic-qos pairs
        let mut topics_ = Seq::default();
        for topic in topics {
            // Copy topic and append pair
            let topic = Bytes::new(topic.as_ref())?;
            topics_.push(topic)?;
        }

        // Init self
        Ok(Self { packet_id, topics: topics_, _vec: PhantomData })
    }

    /// The packet ID
    pub fn packet_id(&self) -> u16 {
        self.packet_id
    }

    /// A list of topic filters
    pub fn topics(&self) -> &Seq {
        &self.topics
    }
}
impl<Seq, Bytes> TryFromIterator for Unsubscribe<Seq, Bytes>
where
    Seq: AnyVec<Bytes>,
    Bytes: AnyVec<u8>,
{
    fn try_from_iter<T>(iter: T) -> Result<Self, &'static str>
    where
        T: IntoIterator<Item = u8>,
    {
        // Read packet:
        //  - header type and `2` flags
        //  - packet len
        //  - packed ID
        //  - sequence
        //     - topic filter
        let mut decoder = Decoder::new(iter);
        let (Self::TYPE, [false, false, true, false]) = decoder.header()? else {
            return Err("Invalid packet type/header");
        };
        // Limit length and make decoder peekable
        let len = decoder.packetlen()?;
        let mut decoder = decoder.limit(len).peekable();
        // Read fields
        let packet_id = decoder.u16()?;
        let topics = decoder.topics()?;

        // Init self
        Ok(Self { packet_id, topics, _vec: PhantomData })
    }
}
impl<Seq, Bytes> IntoIterator for Unsubscribe<Seq, Bytes>
where
    Seq: AnyVec<Bytes>,
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
            // - packed ID
            U16Iter>,
            // - sequence
            //    - topic filter
            TopicsIter<Seq, Bytes>>;

    fn into_iter(self) -> Self::IntoIter {
        // Precompute body length:
        //  - packet ID
        //  - sequence
        //     - topic filter
        #[rustfmt::skip]
        let len = Length::new()
            .u16(&self.packet_id)
            .topics(&self.topics)
            .into();

        // Write packet:
        //  - header type and `2` flags
        //  - packet len
        //  - packed ID
        //  - sequence
        //     - topic filter
        Encoder::default()
            .header(Self::TYPE, [false, false, true, false])
            .packetlen(len)
            .u16(self.packet_id)
            .topics(self.topics)
            .into_iter()
    }
}
