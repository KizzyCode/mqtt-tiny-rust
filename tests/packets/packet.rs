#![cfg(any(feature = "std", feature = "arrayvec"))]

use core::ops::Deref;
use mqtt_tiny::{
    packets::TryFromIterator, Connack, Connect, Disconnect, Packet, Pingreq, Pingresp, Puback, Pubcomp, Publish,
    Pubrec, Pubrel, Suback, Subscribe, Unsuback, Unsubscribe,
};

// Select an appropriate vector type
#[cfg(feature = "std")]
type Vec = std::vec::Vec<u8>;
#[cfg(all(not(feature = "std"), feature = "arrayvec"))]
type Vec = arrayvec::ArrayVec<u8, 64>;

/// A test vector for encoded/decoded pairs
#[derive(Debug, Clone)]
pub struct Good {
    /// The encoded representation
    encoded: &'static [u8],
    /// The decoded representation
    decoded: Packet,
}
impl Good {
    /// Good encoded/decoded pairs
    pub fn all() -> [Self; 14] {
        [
            Self { encoded: b"\x20\x02\x00\x00", decoded: Packet::Connack(Connack::new(false, 0)) },
            Self {
                encoded: b"\x10\x10\x00\x04MQTT\x04\x00\x00\x1E\x00\x04test",
                decoded: Packet::Connect(Connect::new(30, false, b"test").expect("failed to create packet")),
            },
            Self { encoded: b"\xE0\x00", decoded: Packet::Disconnect(Disconnect::new()) },
            Self { encoded: b"\xC0\x00", decoded: Packet::Pingreq(Pingreq::new()) },
            Self { encoded: b"\xD0\x00", decoded: Packet::Pingresp(Pingresp::new()) },
            Self { encoded: b"\x40\x02\x04\x07", decoded: Packet::Puback(Puback::new(0x0407)) },
            Self { encoded: b"\x70\x02\x04\x07", decoded: Packet::Pubcomp(Pubcomp::new(0x0407)) },
            Self {
                encoded: b"\x30\x0B\x00\x04TestOlope",
                decoded: Packet::Publish(Publish::new(b"Test", b"Olope", false).expect("failed to create packet")),
            },
            Self { encoded: b"\x50\x02\x04\x07", decoded: Packet::Pubrec(Pubrec::new(0x0407)) },
            Self { encoded: b"\x60\x02\x04\x07", decoded: Packet::Pubrel(Pubrel::new(0x0407)) },
            Self { encoded: b"\x90\x02\x04\x07", decoded: Packet::Suback(Suback::new(0x0407)) },
            Self {
                encoded: b"\x82\x0E\x04\x07\x00\x09testolope\x01",
                decoded: Packet::Subscribe(
                    Subscribe::new(0x0407, [(b"testolope", 1)]).expect("failed to create packet"),
                ),
            },
            Self { encoded: b"\xB0\x02\x04\x07", decoded: Packet::Unsuback(Unsuback::new(0x0407)) },
            Self {
                encoded: b"\xA2\x0D\x04\x07\x00\x09testolope",
                decoded: Packet::Unsubscribe(
                    Unsubscribe::new(0x0407, [b"testolope"]).expect("failed to create packet"),
                ),
            },
        ]
    }
}

/// A test vector for known-bad encoded encoded fields
#[derive(Debug)]
pub struct BadEncoded {
    /// The invalid encoded representation
    encoded: &'static [u8],
}
impl BadEncoded {
    /// Good encoded/decoded pairs
    pub const fn all() -> &'static [Self] {
        &[
            // Packet with invalid packet type
            Self { encoded: b"\xF0\x00" },
        ]
    }
}

/// Tests successful decoding
#[test]
pub fn decode() {
    for test_vector in Good::all() {
        // Decode and validate
        let encoded = test_vector.encoded.iter().copied();
        let decoded = Packet::try_from_iter(encoded).expect("Failed to decode valid packet");
        assert_eq!(decoded, test_vector.decoded, "Invalid decoded packet")
    }
}

/// Tests successful encoding
#[test]
pub fn encode() {
    for test_vector in Good::all() {
        // Encode and validate
        let decoded = test_vector.decoded.clone();
        let encoded: Vec = decoded.into_iter().collect();
        assert_eq!(encoded.deref(), test_vector.encoded, "Invalid encoded packet");
    }
}

/// Tests failing decoding
#[test]
pub fn decode_invalid() {
    for test_vector in BadEncoded::all() {
        // Decode and validate
        let encoded = test_vector.encoded.iter().copied();
        let decoded = Disconnect::try_from_iter(encoded);
        assert!(decoded.is_err(), "Unexpected success when decoding invalid packet");
    }
}
