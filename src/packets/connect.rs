//! MQTT [`CONNECT`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718033)

use crate::{
    coding::{Length, Reader, Writer},
    error::MqttError,
};
use std::io::{Read, Write};

/// An MQTT [`CONNECT` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718028)
#[derive(Debug, Clone)]
pub struct MqttConnect {
    /// The seconds to keep the connection alive
    keep_alive_secs: u16,
    /// When set to `true` the client and server need not process the deletion of state atomically
    clean_session: bool,
    /// This bit specifies if the will message is to be Retained when it is published
    will_retain: bool,
    /// The QoS level to be used when publishing the will message
    will_qos: u8,
    /// The client identifier
    ///
    /// # Possible allowed characters
    /// The only possible allowed characters are `0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ`, and
    /// should not be longer than 23 bytes.
    client_id: String,
    /// The will topic
    will_topic: Option<String>,
    /// The will message
    will_message: Option<String>,
    /// The username
    username: Option<String>,
    /// The password
    password: Option<Vec<u8>>,
}
impl MqttConnect {
    /// Creates a new packet
    pub fn new<T>(keep_alive_secs: u16, clean_session: bool, client_id: T) -> Self
    where
        T: ToString,
    {
        Self {
            keep_alive_secs,
            clean_session,
            will_retain: false,
            will_qos: 0,
            client_id: client_id.to_string(),
            will_topic: None,
            will_message: None,
            username: None,
            password: None,
        }
    }
    /// Extends `self` with a last-will topic and message
    pub fn with_will<A, B>(mut self, topic: A, message: B, qos: u8, retain: bool) -> Self
    where
        A: ToString,
        B: ToString,
    {
        self.will_topic = Some(topic.to_string());
        self.will_message = Some(message.to_string());
        self.will_retain = retain;
        self.will_qos = qos;
        self
    }
    /// Extends `self` to authenticate with a username
    pub fn with_username<T>(mut self, username: T) -> Self
    where
        T: ToString,
    {
        self.username = Some(username.to_string());
        self
    }
    /// Extends `self` to authenticate with a password
    pub fn with_password<T>(mut self, password: T) -> Self
    where
        T: Into<Vec<u8>>,
    {
        self.password = Some(password.into());
        self
    }

    /// The seconds to keep the connection alive
    pub const fn keep_alive_secs(&self) -> u16 {
        self.keep_alive_secs
    }
    /// When set to `true` the client and server need not process the deletion of state atomically
    pub const fn clean_session(&self) -> bool {
        self.clean_session
    }
    /// This bit specifies if the will message is to be Retained when it is published
    pub const fn will_retain(&self) -> bool {
        self.will_retain
    }
    /// The QoS level to be used when publishing the will message
    pub const fn will_qos(&self) -> u8 {
        self.will_qos
    }
    /// The client identifier
    ///
    /// # Possible allowed characters
    /// The only possible allowed characters are `0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ`, and
    /// should not be longer than 23 bytes.
    pub fn client_id(&self) -> &str {
        &self.client_id
    }
    /// The will topic
    pub fn will_topic(&self) -> Option<&str> {
        self.will_topic.as_deref()
    }
    /// The will message
    pub fn will_message(&self) -> Option<&str> {
        self.will_message.as_deref()
    }
    /// The username
    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }
    /// The password
    pub fn password(&self) -> Option<&[u8]> {
        self.password.as_deref()
    }
}
impl MqttConnect {
    /// The packet type constant
    pub const TYPE: u8 = 1;

    /// The protocol name
    const PROTOCOL_NAME: [u8; 6] = *b"\x00\x04MQTT";
    /// The protocol constant for MQTT 3.1.1
    const PROTOCOL_LEVEL_MQTT_3_1_1: [u8; 1] = *b"\x04";

    /// Reads `Self` from the given source
    pub fn read<T>(source: &mut T) -> Result<Self, MqttError>
    where
        T: Read,
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
        let mut reader = Reader::new(source);
        let _ = reader.read_header(&Self::TYPE)?;
        let len = reader.read_packetlen()?;
        // Limit length
        let mut reader = reader.limit(len);
        let _ = reader.read_constant(&Self::PROTOCOL_NAME)?;
        let _ = reader.read_version_constant(&Self::PROTOCOL_LEVEL_MQTT_3_1_1)?;
        let [f_user, f_pass, will_retain, will_qos0, will_qos1, f_will_flag, clean_session, _] = reader.read_flags()?;
        let keep_alive_secs = reader.read_u16()?;
        let client_id = reader.read_string()?;
        let will_topic = reader.read_optional_string(f_will_flag)?;
        let will_message = reader.read_optional_string(f_will_flag)?;
        let username = reader.read_optional_string(f_user)?;
        let password = reader.read_optional_bytes(f_pass)?;

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

    /// Writes `self` into the given sink
    pub fn write<T>(self, sink: T) -> Result<T, MqttError>
    where
        T: Write,
    {
        // Assemble flags
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
            .add_array(&Self::PROTOCOL_NAME)
            .add_array(&Self::PROTOCOL_LEVEL_MQTT_3_1_1)
            .add_flags(&flags)
            .add_u16(&self.keep_alive_secs)
            .add_string(&self.client_id)
            .add_optional_string(&self.will_topic)
            .add_optional_string(&self.will_message)
            .add_optional_string(&self.username)
            .add_optional_bytes(&self.password)
            .finalize();

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
        Writer::new(sink)
            .write_header(Self::TYPE, [false, false, false, false])?
            .write_packetlen(len)?
            .write_array(Self::PROTOCOL_NAME)?
            .write_array(Self::PROTOCOL_LEVEL_MQTT_3_1_1)?
            .write_flags(flags)?
            .write_u16(self.keep_alive_secs)?
            .write_string(self.client_id)?
            .write_optional_string(self.will_topic)?
            .write_optional_string(self.will_message)?
            .write_optional_string(self.username)?
            .write_optional_bytes(self.password)?
            .finalize()
    }
}
