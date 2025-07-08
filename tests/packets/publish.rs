#![cfg(any(feature = "std", feature = "arrayvec"))]

use core::ops::Deref;
use mqtt_tiny::packets::TryFromIterator;
use mqtt_tiny::Publish;

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
    decoded: Publish,
}
impl Good {
    /// Good encoded/decoded pairs
    pub fn all() -> [Self; 3] {
        [
            // A basic packet
            Self {
                encoded: b"\x30\x0B\x00\x04TestOlope",
                decoded: Publish::new(b"Test", b"Olope", false).expect("failed to create packet"),
            },
            // A packet with QoS
            Self {
                encoded: b"\x34\x0D\x00\x04Test\x04\x07Olope",
                decoded: Publish::new(b"Test", b"Olope", false).expect("failed to create packet")
                    // Set QoS
                    .with_qos(2, 0x0407, false),
            },
            // A packet with everything enabled
            Self {
                encoded: b"\x3B\x0D\x00\x04Test\x04\x07Olope",
                decoded: Publish::new(b"Test", b"Olope", true).expect("failed to create packet")
                    // Set QoS
                    .with_qos(1, 0x0407, true),
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
            Self { encoded: b"\x40\x0B\x00\x04TestOlope" },
            // Packet with non-zero QoS but missing/truncated packet ID
            Self { encoded: b"\x34\x07\x00\x04TestO" },
        ]
    }
}

/// Tests successful decoding
#[test]
pub fn decode() {
    for test_vector in Good::all() {
        // Decode and validate
        let encoded = test_vector.encoded.iter().copied();
        let decoded = Publish::try_from_iter(encoded).expect("Failed to decode valid packet");
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
        let decoded = Publish::try_from_iter(encoded);
        assert!(decoded.is_err(), "Unexpected success when decoding invalid packet");
    }
}
