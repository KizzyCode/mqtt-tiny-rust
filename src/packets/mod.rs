//! Typed MQTT packets

pub mod connack;
pub mod connect;
pub mod disconnect;
pub mod pingreq;
pub mod pingresp;
pub mod publish;
pub mod subscribe;
pub mod unsubscribe;
include!("_acklike.rs");

use crate::{
    coding::Reader,
    error::{ErrorKind, MqttError},
    packets::{
        connack::MqttConnack, connect::MqttConnect, disconnect::MqttDisconnect, pingreq::MqttPingreq,
        pingresp::MqttPingresp, puback::MqttPuback, pubcomp::MqttPubcomp, publish::MqttPublish, pubrec::MqttPubrec,
        pubrel::MqttPubrel, suback::MqttSuback, subscribe::MqttSubscribe, unsuback::MqttUnsuback,
        unsubscribe::MqttUnsubscribe,
    },
};
use std::io::{Read, Write};

/// A type-erased MQTT packet
#[derive(Debug, Clone)]
pub enum MqttPacket {
    /// An [MqttConnack] packet
    CONNACK(MqttConnack),
    /// An [MqttConnect] packet
    CONNECT(MqttConnect),
    /// An [MqttDisconnect] packet
    DISCONNECT(MqttDisconnect),
    /// An [MqttPingreq] packet
    PINGREQ(MqttPingreq),
    /// An [MqttPingresp] packet
    PINGRESP(MqttPingresp),
    /// An [MqttPuback] packet
    PUBACK(MqttPuback),
    /// An [MqttPubcomp] packet
    PUBCOMP(MqttPubcomp),
    /// An [MqttPublish] packet
    PUBLISH(MqttPublish),
    /// An [MqttPubrec] packet
    PUBREC(MqttPubrec),
    /// An [MqttPubrel] packet
    PUBREL(MqttPubrel),
    /// An [MqttSuback] packet
    SUBACK(MqttSuback),
    /// An [MqttSubscribe] packet
    SUBSCRIBE(MqttSubscribe),
    /// An [MqttUnsuback] packet
    UNSUBACK(MqttUnsuback),
    /// An [MqttUnsubscribe] packet
    UNSUBSCRIBE(MqttUnsubscribe),
}
impl MqttPacket {
    /// Reads `Self` from the given source
    pub fn read<T>(source: &mut T) -> Result<Self, MqttError>
    where
        T: Read,
    {
        // We have to peek at the header to determine the type
        let mut source = Reader::new(source).buffered();
        let header = source.peek_u8()?;

        // Select the appropriate packet depending on the type
        match header >> 4 {
            MqttConnack::TYPE => MqttConnack::read(&mut source).map(Self::CONNACK),
            MqttConnect::TYPE => MqttConnect::read(&mut source).map(Self::CONNECT),
            MqttDisconnect::TYPE => MqttDisconnect::read(&mut source).map(Self::DISCONNECT),
            MqttPingreq::TYPE => MqttPingreq::read(&mut source).map(Self::PINGREQ),
            MqttPingresp::TYPE => MqttPingresp::read(&mut source).map(Self::PINGRESP),
            MqttPuback::TYPE => MqttPuback::read(&mut source).map(Self::PUBACK),
            MqttPubcomp::TYPE => MqttPubcomp::read(&mut source).map(Self::PUBCOMP),
            MqttPublish::TYPE => MqttPublish::read(&mut source).map(Self::PUBLISH),
            MqttPubrec::TYPE => MqttPubrec::read(&mut source).map(Self::PUBREC),
            MqttPubrel::TYPE => MqttPubrel::read(&mut source).map(Self::PUBREL),
            MqttSuback::TYPE => MqttSuback::read(&mut source).map(Self::SUBACK),
            MqttSubscribe::TYPE => MqttSubscribe::read(&mut source).map(Self::SUBSCRIBE),
            MqttUnsuback::TYPE => MqttUnsuback::read(&mut source).map(Self::UNSUBACK),
            MqttUnsubscribe::TYPE => MqttUnsubscribe::read(&mut source).map(Self::UNSUBSCRIBE),
            _ => Err(ErrorKind::InvalidValue.into()),
        }
    }

    /// Writes `self` into the given sink
    pub fn write<T>(self, sink: T) -> Result<T, MqttError>
    where
        T: Write,
    {
        match self {
            MqttPacket::CONNACK(this) => this.write(sink),
            MqttPacket::CONNECT(this) => this.write(sink),
            MqttPacket::DISCONNECT(this) => this.write(sink),
            MqttPacket::PINGREQ(this) => this.write(sink),
            MqttPacket::PINGRESP(this) => this.write(sink),
            MqttPacket::PUBACK(this) => this.write(sink),
            MqttPacket::PUBCOMP(this) => this.write(sink),
            MqttPacket::PUBLISH(this) => this.write(sink),
            MqttPacket::PUBREC(this) => this.write(sink),
            MqttPacket::PUBREL(this) => this.write(sink),
            MqttPacket::SUBACK(this) => this.write(sink),
            MqttPacket::SUBSCRIBE(this) => this.write(sink),
            MqttPacket::UNSUBACK(this) => this.write(sink),
            MqttPacket::UNSUBSCRIBE(this) => this.write(sink),
        }
    }
}
