use core::ops::Deref;
use mqtt_tiny::{packets::TryFromIterator, Subscribe};

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
    decoded: Subscribe,
}
impl Good {
    /// Good encoded/decoded pairs
    pub fn all() -> [Self; 2] {
        [
            // Single topic subscription
            Self {
                encoded: b"\x82\x0E\x04\x07\x00\x09testolope\x01",
                decoded: Subscribe::new(0x0407, [(b"testolope", 1)]).expect("failed to create packet"),
            },
            // Multiple topic subscription
            Self {
                encoded: b"\x82\x11\x04\x07\x00\x04test\x01\x00\x05olope\x02",
                decoded: Subscribe::new(0x0407, [("test", 1), ("olope", 2)]).expect("failed to create packet"),
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
            Self { encoded: b"\x92\x0E\x04\x07\x00\x09testolope\x01" },
            // Packet with invalid header flags
            Self { encoded: b"\x80\x0E\x04\x07\x00\x09testolope\x01" },
        ]
    }
}

/// Tests successful decoding
#[test]
pub fn decode() {
    for test_vector in Good::all() {
        // Decode and validate
        let encoded = test_vector.encoded.iter().copied();
        let decoded = Subscribe::try_from_iter(encoded).expect("Failed to decode valid packet");
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
        let decoded = Subscribe::try_from_iter(encoded);
        assert!(decoded.is_err(), "Unexpected success when decoding invalid packet");
    }
}
