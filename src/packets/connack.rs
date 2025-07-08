//! MQTT [`CONNACK`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718033)

use crate::coding::encoder::{PacketLenIter, U8Iter, Unit};
use crate::coding::{Decoder, Encoder};
use crate::err;
use crate::error::{Data, DecoderError};
use crate::packets::TryFromIterator;
use core::iter::Chain;

/// An MQTT [`CONNACK` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718033)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connack {
    /// Whether a previous session is present or not
    session_present: bool,
    /// The return code
    return_code: u8,
}
impl Connack {
    /// The packet type constant
    pub const TYPE: u8 = 2;

    /// The expected body length
    const BODY_LEN: usize = 2;

    /// Creates a new packet
    pub const fn new(session_present: bool, return_code: u8) -> Self {
        Self { session_present, return_code }
    }

    /// Whether a previous session is present or not
    pub const fn session_present(&self) -> bool {
        self.session_present
    }
    /// The return code
    pub const fn return_code(&self) -> u8 {
        self.return_code
    }
}
impl TryFromIterator for Connack {
    fn try_from_iter<T>(iter: T) -> Result<Self, DecoderError>
    where
        T: IntoIterator<Item = u8>,
    {
        // Read packet:
        //  - header type and `0` flags
        //  - packet len
        //  - ACK flags
        //  - return code
        let mut decoder = Decoder::new(iter);
        let (Self::TYPE, _flags) = decoder.header()? else {
            return Err(err!(Data::SpecViolation, "invalid packet type"))?;
        };
        let Self::BODY_LEN = decoder.packetlen()? else {
            return Err(err!(Data::SpecViolation, "invalid packet length"))?;
        };

        // Read fields
        let [_, _, _, _, _, _, _, session_present] = decoder.bitmap()?;
        let return_code = decoder.u8()?;

        // Init self
        Ok(Self { session_present, return_code })
    }
}
impl IntoIterator for Connack {
    type Item = u8;
    #[rustfmt::skip]
    type IntoIter =
        // Complex iterator built out of the individual message fields
        Chain<Chain<Chain<Chain<
            // - header type and `0` flags
            Unit, U8Iter>,
            // - packet len
            PacketLenIter>,
            // - ACK flags
            U8Iter>,
            // - return code
            U8Iter>;

    fn into_iter(self) -> Self::IntoIter {
        // Write packet:
        //  - header type and `0` flags
        //  - packet len
        //  - ACK flags
        //  - return code
        Encoder::default()
            .header(Self::TYPE, [false, false, false, false])
            .packetlen(Self::BODY_LEN)
            .bitmap([false, false, false, false, false, false, false, self.session_present])
            .u8(self.return_code)
            .into_iter()
    }
}
