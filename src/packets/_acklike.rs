/// Defines an ACK-like packet (i.e. a response packet with a single 16bit packet-ID field)
#[rustfmt::skip]
macro_rules! acklike {
    ($docstr:expr, $type:ident => $const:expr) => {
        #[doc = $docstr]
        #[derive(Debug, Clone)]
        pub struct $type {
            /// The packet identifier
            packet_id: u16,
        }
        impl $type {
            /// Creates a new packet
            pub const fn new(packet_id: u16) -> Self {
                Self { packet_id }
            }
            
            /// The packet ID
            pub const fn packet_id(&self) -> u16 {
                self.packet_id
            }
        }
        impl $type {
            /// The packet type constant
            pub const TYPE: u8 = $const;

            /// For this packet, the body length is fixed
            const BODY_LEN: [u8; 1] = [2];

            /// Reads `Self` from the given source
            pub fn read<T>(source: &mut T) -> Result<Self, $crate::error::MqttError>
            where
                T: std::io::Read,
            {
                // Read header:
                //  - header type and `0` flags
                //  - packet len
                //  - packet ID
                let mut reader = $crate::coding::Reader::new(source);
                let _ = reader.read_header(&Self::TYPE)?;
                let _ = reader.read_constant(&Self::BODY_LEN)?;
                let packet_id = reader.read_u16()?;

                // Init self
                Ok(Self { packet_id })
            }

            /// Writes `self` into the given sink
            pub fn write<T>(self, sink: T) -> Result<T, $crate::error::MqttError>
            where
                T: std::io::Write,
            {
                // Write header:
                //  - header type and `0` flags
                //  - packet len
                //  - packet ID
                $crate::coding::Writer::new(sink)
                    .write_header(Self::TYPE, [false, false, false, false])?
                    .write_array(Self::BODY_LEN)?
                    .write_u16(self.packet_id)?
                    .finalize()
            }
        }
    };
}

pub mod puback {
    //! MQTT [`PUBACK`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718043)
    acklike! {
        "An MQTT [`PUBACK` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718043)",
        MqttPuback => 4
    }
}

pub mod pubcomp {
    //! MQTT [`PUBCOMP`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718058)
    acklike! {
        "An MQTT [`PUBCOMP` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718058)",
        MqttPubcomp => 7
    }
}

pub mod pubrec {
    //! MQTT [`PUBREC`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718048)
    acklike! {
        "An MQTT [`PUBREC` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718048)",
        MqttPubrec => 5
    }
}

pub mod pubrel {
    //! MQTT [`PUBREL`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718053)
    acklike! {
        "An MQTT [`PUBREL` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718053)",
        MqttPubrel => 6
    }
}

pub mod suback {
    //! MQTT [`SUBACK`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718068)
    acklike! {
        "An MQTT [`SUBACK` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718068)",
        MqttSuback => 9
    }
}

pub mod unsuback {
    //! MQTT [`UNSUBACK`](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718077)
    acklike! {
        "An MQTT [`UNSUBACK` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718077)",
        MqttUnsuback => 11
    }
}
