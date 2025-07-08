//! MQTT [`CONNECT`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718033)

use crate::anyvec::AnyVec;
use crate::coding::encoder::{BytesIter, OptionalBytesIter, PacketLenIter, U16Iter, U8Iter, Unit};
use crate::coding::length::Length;
use crate::coding::{Decoder, Encoder};
use crate::err;
use crate::error::{Data, DecoderError, MemoryError};
use crate::packets::TryFromIterator;
use core::iter::Chain;

/// An MQTT [`CONNECT` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718033)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connect<Bytes> {
    /// The seconds to keep the connection alive
    keep_alive_secs: u16,
    /// When set to `true` the client and server need not process the deletion of state atomically
    clean_session: bool,
    /// This bit specifies if the will message is to be Retained when it is published
    will_retain: bool,
    /// The QoS level to be used when publishing the will message
    ///
    /// # QoS Levels
    /// Valid QoS levels are:
    ///  - `0`: At most one delivery
    ///  - `1`: At least one delivery
    ///  - `2`: Exactly one delivery
    will_qos: u8,
    /// The client identifier
    ///
    /// # Important
    /// MQTT allows only the characters `0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ`, and the
    /// identifier should not be longer than 23 bytes.
    client_id: Bytes,
    /// The will topic
    will_topic: Option<Bytes>,
    /// The will message
    will_message: Option<Bytes>,
    /// The username
    username: Option<Bytes>,
    /// The password
    password: Option<Bytes>,
}
impl<Bytes> Connect<Bytes>
where
    Bytes: AnyVec<u8>,
{
    /// The packet type constant
    pub const TYPE: u8 = 1;

    /// The protocol name
    const PROTOCOL_NAME: [u8; 6] = *b"\x00\x04MQTT";
    /// The protocol constant for MQTT 3.1.1
    const PROTOCOL_LEVEL_MQTT_3_1_1: u8 = 0x04;

    /// Creates a new packet
    pub fn new<T>(keep_alive_secs: u16, clean_session: bool, client_id: T) -> Result<Self, MemoryError>
    where
        T: AsRef<[u8]>,
    {
        let client_id = Bytes::new(client_id.as_ref())?;
        Ok(Self {
            keep_alive_secs,
            clean_session,
            will_retain: false,
            will_qos: 0,
            client_id,
            will_topic: None,
            will_message: None,
            username: None,
            password: None,
        })
    }
    /// Configures a last-will topic and message
    ///
    /// # QoS Levels
    /// Valid QoS levels are:
    ///  - `0`: At most one delivery
    ///  - `1`: At least one delivery
    ///  - `2`: Exactly one delivery
    pub fn with_will<T, M>(mut self, topic: T, message: M, qos: u8, retain: bool) -> Result<Self, MemoryError>
    where
        T: AsRef<[u8]>,
        M: AsRef<[u8]>,
    {
        self.will_topic = Bytes::new(topic.as_ref()).map(Some)?;
        self.will_message = Bytes::new(message.as_ref()).map(Some)?;
        self.will_retain = retain;
        self.will_qos = qos;
        Ok(self)
    }
    /// Configures a username and password
    pub fn with_username_password<U, P>(mut self, username: U, password: P) -> Result<Self, MemoryError>
    where
        U: AsRef<[u8]>,
        P: AsRef<[u8]>,
    {
        self.username = Bytes::new(username.as_ref()).map(Some)?;
        self.password = Bytes::new(password.as_ref()).map(Some)?;
        Ok(self)
    }

    /// Gets the seconds to keep the connection alive
    pub const fn keep_alive_secs(&self) -> u16 {
        self.keep_alive_secs
    }

    /// Gets the clean session bit which indicate if the client and server do not need to process the deletion of state
    /// atomically
    pub const fn clean_session(&self) -> bool {
        self.clean_session
    }

    /// Gets the client identifier
    ///
    /// # Important
    /// MQTT allows only the characters `0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ`, and the
    /// identifier should not be longer than 23 bytes.
    pub fn client_id(&self) -> &[u8] {
        self.client_id.as_ref()
    }

    /// Gets the will-retain bit to indicate if the will message is to be Retained when it is published
    pub const fn will_retain(&self) -> bool {
        self.will_retain
    }
    /// Gets the QoS level to be used when publishing the will message
    pub const fn will_qos(&self) -> u8 {
        self.will_qos
    }
    /// Gets the will topic
    pub fn will_topic(&self) -> Option<&[u8]> {
        self.will_topic.as_ref().map(|bytes| bytes.as_ref())
    }
    /// Gets the will message
    pub fn will_message(&self) -> Option<&[u8]> {
        self.will_message.as_ref().map(|bytes| bytes.as_ref())
    }

    /// Gets the username
    pub fn username(&self) -> Option<&[u8]> {
        self.username.as_ref().map(|bytes| bytes.as_ref())
    }
    /// Gets the password
    pub fn password(&self) -> Option<&[u8]> {
        self.password.as_ref().map(|bytes| bytes.as_ref())
    }
}
impl<Bytes> TryFromIterator for Connect<Bytes>
where
    Bytes: AnyVec<u8>,
{
    fn try_from_iter<T>(iter: T) -> Result<Self, DecoderError>
    where
        T: IntoIterator<Item = u8>,
    {
        // Read packet:
        //  - header type and `0` flags
        //  - packet len
        //  - protocol name
        //  - protocol level
        //  - connect flags
        //  - keep-alive
        //  - client id
        //  - will topic
        //  - will message
        //  - username
        //  - password
        let mut decoder = Decoder::new(iter);
        let (Self::TYPE, _flags) = decoder.header()? else {
            return Err(err!(Data::SpecViolation, "invalid packet type"))?;
        };

        // Limit length
        let len = decoder.packetlen()?;
        let mut decoder = decoder.limit(len);

        // Read protocol name byte-by-byte and version
        let Self::PROTOCOL_NAME = decoder.raw()? else {
            return Err(err!(Data::SpecViolation, "invalid protocol name"))?;
        };
        let Self::PROTOCOL_LEVEL_MQTT_3_1_1 = decoder.u8()? else {
            return Err(err!(Data::SpecViolation, "invalid protocol version"))?;
        };

        // Read fields
        let [f_user, f_pass, will_retain, will_qos0, will_qos1, f_will, clean_session, _] = decoder.bitmap()?;
        let keep_alive_secs = decoder.u16()?;
        let client_id = decoder.bytes()?;
        let will_topic = decoder.optional_bytes(f_will)?;
        let will_message = decoder.optional_bytes(f_will)?;
        let username = decoder.optional_bytes(f_user)?;
        let password = decoder.optional_bytes(f_pass)?;

        // Init self
        let will_qos = ((will_qos0 as u8) << 1) | (will_qos1 as u8);
        Ok(Self {
            keep_alive_secs,
            clean_session,
            will_retain,
            will_qos,
            client_id,
            will_topic,
            will_message,
            username,
            password,
        })
    }
}
impl<Bytes> IntoIterator for Connect<Bytes>
where
    Bytes: AnyVec<u8>,
{
    type Item = u8;
    #[rustfmt::skip]
    type IntoIter =
        // Complex iterator built out of the individual message fields
        Chain<Chain<Chain<Chain<Chain<Chain<Chain<Chain<Chain<Chain<Chain<
            // - header type and `0` flags
            Unit, U8Iter>,
            // - packet len
            PacketLenIter>,
            // - protocol name
            <[u8; 6] as IntoIterator>::IntoIter>,
            // - protocol level
            U8Iter>,
            // - connect flags
            U8Iter>,
            // - keep-alive
            U16Iter>,
            // - client id
            BytesIter<Bytes>>,
            // - will topic
            OptionalBytesIter<Bytes>>,
            // - will message
            OptionalBytesIter<Bytes>>,
            // - username
            OptionalBytesIter<Bytes>>,
            // - password
            OptionalBytesIter<Bytes>>;

    fn into_iter(self) -> Self::IntoIter {
        // Assemble protocol name and flags
        let flags = [
            self.username.is_some(),
            self.password.is_some(),
            self.will_retain,
            (self.will_qos >> 1) != 0,
            (self.will_qos & 1) != 0,
            self.will_topic.is_some(),
            self.clean_session,
            false,
        ];

        // Precompute body length:
        //  - protocol name
        //  - protocol level
        //  - connect flags
        //  - keep-alive
        //  - client id
        //  - will topic
        //  - will message
        //  - username
        //  - password
        let len = Length::new()
            .raw(&Self::PROTOCOL_NAME)
            .u8(&Self::PROTOCOL_LEVEL_MQTT_3_1_1)
            .bitmap(&flags)
            .u16(&self.keep_alive_secs)
            .bytes(&self.client_id)
            .optional_bytes(&self.will_topic)
            .optional_bytes(&self.will_message)
            .optional_bytes(&self.username)
            .optional_bytes(&self.password)
            .into();

        // Write header:
        //  - header type and `0` flags
        //  - packet len
        //  - protocol name
        //  - protocol level
        //  - connect flags
        //  - keep-alive
        //  - client id
        //  - will topic
        //  - will message
        //  - username
        //  - password
        Encoder::default()
            .header(Self::TYPE, [false, false, false, false])
            .packetlen(len)
            .raw(Self::PROTOCOL_NAME)
            .u8(Self::PROTOCOL_LEVEL_MQTT_3_1_1)
            .bitmap(flags)
            .u16(self.keep_alive_secs)
            .bytes(self.client_id)
            .optional_bytes(self.will_topic)
            .optional_bytes(self.will_message)
            .optional_bytes(self.username)
            .optional_bytes(self.password)
            .into_iter()
    }
}
