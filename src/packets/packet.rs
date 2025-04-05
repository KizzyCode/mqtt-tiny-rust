//! A type-erased MQTT packet

use crate::{
    anyvec::AnyVec,
    packets::{
        connack::Connack, connect::Connect, disconnect::Disconnect, pingreq::Pingreq, pingresp::Pingresp,
        puback::Puback, pubcomp::Pubcomp, publish::Publish, pubrec::Pubrec, pubrel::Pubrel, suback::Suback,
        subscribe::Subscribe, unsuback::Unsuback, unsubscribe::Unsubscribe, TryFromIterator,
    },
};

/// A type-erased MQTT packet
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Packet<TopicsSeq, TopicsQosSeq, Bytes> {
    /// An [`Connack`] packet
    Connack(Connack),
    /// An [`Connect`] packet
    Connect(Connect<Bytes>),
    /// An [`Disconnect`] packet
    Disconnect(Disconnect),
    /// An [`Pingreq`] packet
    Pingreq(Pingreq),
    /// An [`Pingresp`] packet
    Pingresp(Pingresp),
    /// An [`Puback`] packet
    Puback(Puback),
    /// An [`Pubcomp`] packet
    Pubcomp(Pubcomp),
    /// An [`Publish`] packet
    Publish(Publish<Bytes>),
    /// An [`Pubrec`] packet
    Pubrec(Pubrec),
    /// An [`Pubrel`] packet
    Pubrel(Pubrel),
    /// An [`Suback`] packet
    Suback(Suback),
    /// An [`Subscribe`] packet
    Subscribe(Subscribe<TopicsQosSeq, Bytes>),
    /// An [`Unsuback`] packet
    Unsuback(Unsuback),
    /// An [`Unsubscribe`] packet
    Unsubscribe(Unsubscribe<TopicsSeq, Bytes>),
}
impl<TopicsSeq, TopicsQosSeq, Bytes> TryFromIterator for Packet<TopicsSeq, TopicsQosSeq, Bytes>
where
    TopicsSeq: AnyVec<Bytes>,
    TopicsQosSeq: AnyVec<(Bytes, u8)>,
    Bytes: AnyVec<u8>,
{
    fn try_from_iter<T>(iter: T) -> Result<Self, &'static str>
    where
        T: IntoIterator<Item = u8>,
    {
        // We have to peek at the header to determine the type
        let mut decoder = iter.into_iter().peekable();
        let header = decoder.next().ok_or("Empty packet")?;

        // Select the appropriate packet depending on the type
        match header >> 4 {
            Connack::TYPE => Connack::try_from_iter(&mut decoder).map(Self::Connack),
            Connect::<Bytes>::TYPE => Connect::try_from_iter(&mut decoder).map(Self::Connect),
            Disconnect::TYPE => Disconnect::try_from_iter(&mut decoder).map(Self::Disconnect),
            Pingreq::TYPE => Pingreq::try_from_iter(&mut decoder).map(Self::Pingreq),
            Pingresp::TYPE => Pingresp::try_from_iter(&mut decoder).map(Self::Pingresp),
            Puback::TYPE => Puback::try_from_iter(&mut decoder).map(Self::Puback),
            Pubcomp::TYPE => Pubcomp::try_from_iter(&mut decoder).map(Self::Pubcomp),
            Publish::<Bytes>::TYPE => Publish::try_from_iter(&mut decoder).map(Self::Publish),
            Pubrec::TYPE => Pubrec::try_from_iter(&mut decoder).map(Self::Pubrec),
            Pubrel::TYPE => Pubrel::try_from_iter(&mut decoder).map(Self::Pubrel),
            Suback::TYPE => Suback::try_from_iter(&mut decoder).map(Self::Suback),
            Subscribe::<TopicsQosSeq, Bytes>::TYPE => Subscribe::try_from_iter(&mut decoder).map(Self::Subscribe),
            Unsuback::TYPE => Unsuback::try_from_iter(&mut decoder).map(Self::Unsuback),
            Unsubscribe::<TopicsSeq, Bytes>::TYPE => Unsubscribe::try_from_iter(&mut decoder).map(Self::Unsubscribe),
            _ => Err("Unknown packet type"),
        }
    }
}
impl<TopicsSeq, TopicsQosSeq, Bytes> IntoIterator for Packet<TopicsSeq, TopicsQosSeq, Bytes>
where
    TopicsSeq: AnyVec<Bytes>,
    TopicsQosSeq: AnyVec<(Bytes, u8)>,
    Bytes: AnyVec<u8>,
{
    type Item = u8;
    type IntoIter = PacketIter<TopicsSeq, TopicsQosSeq, Bytes>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Connack(this) => PacketIter::Connack(this.into_iter()),
            Self::Connect(this) => PacketIter::Connect(this.into_iter()),
            Self::Disconnect(this) => PacketIter::Disconnect(this.into_iter()),
            Self::Pingreq(this) => PacketIter::Pingreq(this.into_iter()),
            Self::Pingresp(this) => PacketIter::Pingresp(this.into_iter()),
            Self::Puback(this) => PacketIter::Puback(this.into_iter()),
            Self::Pubcomp(this) => PacketIter::Pubcomp(this.into_iter()),
            Self::Publish(this) => PacketIter::Publish(this.into_iter()),
            Self::Pubrec(this) => PacketIter::Pubreq(this.into_iter()),
            Self::Pubrel(this) => PacketIter::Pubrel(this.into_iter()),
            Self::Suback(this) => PacketIter::Suback(this.into_iter()),
            Self::Subscribe(this) => PacketIter::Subscribe(this.into_iter()),
            Self::Unsuback(this) => PacketIter::Unsuback(this.into_iter()),
            Self::Unsubscribe(this) => PacketIter::Unsubscribe(this.into_iter()),
        }
    }
}

/// A packet-type-erased iterator over the encoded representation
pub enum PacketIter<TopicsSeq, TopicsQosSeq, Bytes>
where
    TopicsSeq: AnyVec<Bytes>,
    TopicsQosSeq: AnyVec<(Bytes, u8)>,
    Bytes: AnyVec<u8>,
{
    /// An [`Connack`] packet iterator
    Connack(<Connack as IntoIterator>::IntoIter),
    /// An [`Connect`] packet iterator
    Connect(<Connect<Bytes> as IntoIterator>::IntoIter),
    /// An [`Disconnect`] packet iterator
    Disconnect(<Disconnect as IntoIterator>::IntoIter),
    /// An [`Pingreq`] packet iterator
    Pingreq(<Pingreq as IntoIterator>::IntoIter),
    /// An [`Pingresp`] packet iterator
    Pingresp(<Pingresp as IntoIterator>::IntoIter),
    /// An [`Puback`] packet iterator
    Puback(<Puback as IntoIterator>::IntoIter),
    /// An [`Pubcomp`] packet iterator
    Pubcomp(<Pubcomp as IntoIterator>::IntoIter),
    /// An [`Publish`] packet iterator
    Publish(<Publish<Bytes> as IntoIterator>::IntoIter),
    /// An [`Pubrec`] packet iterator
    Pubreq(<Pubrec as IntoIterator>::IntoIter),
    /// An [`Pubrel`] packet iterator
    Pubrel(<Pubrel as IntoIterator>::IntoIter),
    /// An [`Suback`] packet iterator
    Suback(<Suback as IntoIterator>::IntoIter),
    /// An [`Subscribe`] packet iterator
    Subscribe(<Subscribe<TopicsQosSeq, Bytes> as IntoIterator>::IntoIter),
    /// An [`Unsuback`] packet iterator
    Unsuback(<Unsuback as IntoIterator>::IntoIter),
    /// An [`Unsubscribe`] packet iterator
    Unsubscribe(<Unsubscribe<TopicsSeq, Bytes> as IntoIterator>::IntoIter),
}
impl<TopicsSeq, TopicsQosSeq, Bytes> Iterator for PacketIter<TopicsSeq, TopicsQosSeq, Bytes>
where
    TopicsSeq: AnyVec<Bytes>,
    TopicsQosSeq: AnyVec<(Bytes, u8)>,
    Bytes: AnyVec<u8>,
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Connack(iter) => iter.next(),
            Self::Connect(iter) => iter.next(),
            Self::Disconnect(iter) => iter.next(),
            Self::Pingreq(iter) => iter.next(),
            Self::Pingresp(iter) => iter.next(),
            Self::Puback(iter) => iter.next(),
            Self::Pubcomp(iter) => iter.next(),
            Self::Publish(iter) => iter.next(),
            Self::Pubreq(iter) => iter.next(),
            Self::Pubrel(iter) => iter.next(),
            Self::Suback(iter) => iter.next(),
            Self::Subscribe(iter) => iter.next(),
            Self::Unsuback(iter) => iter.next(),
            Self::Unsubscribe(iter) => iter.next(),
        }
    }
}
