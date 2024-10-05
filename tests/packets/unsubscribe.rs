#![cfg(any(feature = "std", feature = "arrayvec"))]

use core::ops::Deref;
use mqtt_tiny::{packets::TryFromIterator, Unsubscribe};

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
    decoded: Unsubscribe,
}
impl Good {
    /// Good encoded/decoded pairs
    pub fn all() -> [Self; 2] {
        [
            // Single topic unsubscription
            Self {
                encoded: b"\xA2\x0D\x04\x07\x00\x09testolope",
                decoded: Unsubscribe::new(0x0407, [b"testolope"]).expect("failed to create packet"),
            },
            // Multiple topic unsubscription
            Self {
                encoded: b"\xA2\x0F\x04\x07\x00\x04test\x00\x05olope",
                decoded: Unsubscribe::new(0x0407, ["test", "olope"]).expect("failed to create packet"),
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
            Self { encoded: b"\xB2\x0D\x04\x07\x00\x09testolope" },
            // Packet with invalid header flags
            Self { encoded: b"\xA0\x0D\x04\x07\x00\x09testolope" },
        ]
    }
}

/// Tests successful decoding
#[test]
pub fn decode() {
    for test_vector in Good::all() {
        // Decode and validate
        let encoded = test_vector.encoded.iter().copied();
        let decoded = Unsubscribe::try_from_iter(encoded).expect("Failed to decode valid packet");
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
        let decoded = Unsubscribe::try_from_iter(encoded);
        assert!(decoded.is_err(), "Unexpected success when decoding invalid packet");
    }
}
