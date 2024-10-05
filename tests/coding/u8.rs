#![cfg(any(feature = "std", feature = "arrayvec"))]

use mqtt_tiny::coding::{length::Length, Decoder, Encoder};
use std::ops::Deref;

// Select an appropriate vector type
#[cfg(feature = "std")]
type Vec = std::vec::Vec<u8>;
#[cfg(all(not(feature = "std"), feature = "arrayvec"))]
type Vec = arrayvec::ArrayVec<u8, 64>;

/// A test vector for encoded/decoded pairs
#[derive(Debug, Clone, Copy)]
pub struct Good {
    /// The encoded representation
    encoded: &'static [u8],
    /// The decoded representation
    decoded: u8,
}
impl Good {
    /// Good encoded/decoded pairs
    pub const fn all() -> &'static [Self] {
        &[
            Self { encoded: &[0x00], decoded: 0x00 },
            Self { encoded: &[0x04], decoded: 0x04 },
            Self { encoded: &[0x07], decoded: 0x07 },
            Self { encoded: &[0xFF], decoded: 0xFF },
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
        &[Self { encoded: &[] }]
    }
}

/// Tests successful decoding
#[test]
pub fn decode() {
    for test_vector in Good::all() {
        // Decode and validate
        let encoded = test_vector.encoded.iter().copied();
        let decoded = Decoder::new(encoded).u8().expect("Failed to decode valid byte");
        assert_eq!(decoded, test_vector.decoded, "Invalid decoded byte")
    }
}

/// Tests successful encoding
#[test]
pub fn encode() {
    for test_vector in Good::all() {
        // Encode length
        let length: usize = Length::new().u8(&test_vector.decoded).into();

        // Encode and validate
        let encoded = Encoder::default().u8(test_vector.decoded);
        let encoded: Vec = encoded.into_iter().collect();
        assert_eq!(encoded.deref(), test_vector.encoded, "Invalid encoded byte");
        assert_eq!(length, test_vector.encoded.len(), "Invalid encoded length");
    }
}

/// Tests failing decoding
#[test]
pub fn decode_invalid() {
    for test_vector in BadEncoded::all() {
        // Decode and validate
        let encoded = test_vector.encoded.iter().copied();
        let decoded = Decoder::new(encoded).u8();
        assert!(decoded.is_err(), "Unexpected success when decoding invalid byte");
    }
}
