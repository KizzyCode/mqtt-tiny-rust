/// Defines an ACK-like packet (i.e. a response packet with a single 16bit packet-ID field)
#[rustfmt::skip]
macro_rules! emptylike {
    ($docstr:expr, $type:ident => $typeconst:expr) => {
        #[doc = $docstr]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $type {
            _private: ()
        }
        impl $type {
            /// The packet type constant
            pub const TYPE: u8 = $typeconst;

            /// For this packet, the body length is fixed
            const BODY_LEN: usize = 0;

            /// Creates a new packet
            #[allow(clippy::new_without_default, reason = "packets should not be constructed via `Default`")]
            pub const fn new() -> Self {
                Self { _private: () }
            }
        }
        impl $crate::packets::TryFromIterator for $type {
            fn try_from_iter<T>(iter: T) -> Result<Self, crate::error::DecoderError>
            where
                T: IntoIterator<Item = u8>,
            {
                use crate::err;
                use crate::coding::Decoder;
                use crate::error::Data;

                // Read packet:
                //  - header type and `0` flags
                //  - packet len
                //  - packet I
                let mut decoder = Decoder::new(iter);
                let (Self::TYPE, _flags) = decoder.header()? else {
                    return Err(err!(Data::SpecViolation, "invalid packet type"))?;
                };
                let Self::BODY_LEN = decoder.packetlen()? else {
                    return Err(err!(Data::SpecViolation, "invalid packet length"))?;
                };
        
                // Init self
                Ok(Self { _private: () })
            }
        }
        impl IntoIterator for $type {
            type Item = u8;
            #[rustfmt::skip]
            type IntoIter = 
                // Complex iterator built out of the individual message fields
                core::iter::Chain<core::iter::Chain<
                    // - header type and `0` flags
                    $crate::coding::encoder::Unit, $crate::coding::encoder::U8Iter>, 
                    // - packet len
                    $crate::coding::encoder::PacketLenIter>;
        
            fn into_iter(self) -> Self::IntoIter {
                use crate::coding::Encoder;

                // Write packet:
                //  - header type and `0` flags
                //  - packet len
                Encoder::default()
                    .header(Self::TYPE, [false, false, false, false])
                    .packetlen(Self::BODY_LEN)
                    .into_iter()
            }
        }
    };
}

pub mod disconnect {
    //! MQTT [`DISCONNECT`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718090)
    emptylike! {
        "An MQTT [`DISCONNECT` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718090)",
        Disconnect => 14
    }
}

pub mod pingreq {
    //! MQTT [`PINGREQ`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718081)
    emptylike! {
        "An MQTT [`PINGREQ` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718081)",
        Pingreq => 12
    }
}

pub mod pingresp {
    //! MQTT [`PINGRESP`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718086)
    emptylike! {
        "An MQTT [`PINGRESP` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718086)",
        Pingresp => 13
    }
}
