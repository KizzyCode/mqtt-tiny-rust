/// Defines an ACK-like packet (i.e. a response packet with a single 16bit packet-ID field)
#[rustfmt::skip]
macro_rules! acklike {
    ($docstr:expr, $type:ident => $typeconst:expr) => {
        #[doc = $docstr]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $type {
            /// The packet identifier
            packet_id: u16,
        }
        impl $type {
            /// The packet type constant
            pub const TYPE: u8 = $typeconst;

            /// For this packet, the body length is fixed
            const BODY_LEN: usize = 2;

            /// Creates a new packet
            pub const fn new(packet_id: u16) -> Self {
                Self { packet_id }
            }
            
            /// The packet ID
            pub const fn packet_id(&self) -> u16 {
                self.packet_id
            }
        }
        impl $crate::packets::TryFromIterator for $type {
            fn try_from_iter<T>(iter: T) -> Result<Self, &'static str>
            where
                T: IntoIterator<Item = u8>,
            {
                use crate::coding::Decoder;

                // Read packet:
                //  - header type and `0` flags
                //  - packet len
                //  - packet ID
                let mut decoder = Decoder::new(iter);
                let (Self::TYPE, _flags) = decoder.header()? else {
                    return Err("Invalid packet type");
                };
                let Self::BODY_LEN = decoder.packetlen()? else {
                    return Err("Invalid packet length");
                };
                // Read fields
                let packet_id = decoder.u16()?;
        
                // Init self
                Ok(Self { packet_id })
            }
        }
        impl IntoIterator for $type {
            type Item = u8;
            #[rustfmt::skip]
            type IntoIter = 
                // Complex iterator built out of the individual message fields
                core::iter::Chain<core::iter::Chain<core::iter::Chain<
                    // - header type and `0` flags
                    $crate::coding::encoder::Unit, $crate::coding::encoder::U8Iter>, 
                    // - packet len
                    $crate::coding::encoder::PacketLenIter>,
                    // - packet ID
                    $crate::coding::encoder::U16Iter>;
        
            fn into_iter(self) -> Self::IntoIter {
                use crate::coding::Encoder;

                // Write packet:
                //  - header type and `0` flags
                //  - packet len
                //  - packet ID
                Encoder::default()
                    .header(Self::TYPE, [false, false, false, false])
                    .packetlen(Self::BODY_LEN)
                    .u16(self.packet_id)
                    .into_iter()
            }
        }
    };
}

pub mod puback {
    //! MQTT [`PUBACK`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718043)
    acklike! {
        "An MQTT [`PUBACK` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718043)",
        Puback => 4
    }
}

pub mod pubcomp {
    //! MQTT [`PUBCOMP`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718058)
    acklike! {
        "An MQTT [`PUBCOMP` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718058)",
        Pubcomp => 7
    }
}

pub mod pubrec {
    //! MQTT [`PUBREC`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718048)
    acklike! {
        "An MQTT [`PUBREC` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718048)",
        Pubrec => 5
    }
}

pub mod pubrel {
    //! MQTT [`PUBREL`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718053)
    acklike! {
        "An MQTT [`PUBREL` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718053)",
        Pubrel => 6
    }
}

pub mod suback {
    //! MQTT [`SUBACK`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718068)
    acklike! {
        "An MQTT [`SUBACK` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718068)",
        Suback => 9
    }
}

pub mod unsuback {
    //! MQTT [`UNSUBACK`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718077)
    acklike! {
        "An MQTT [`UNSUBACK` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718077)",
        Unsuback => 11
    }
}
