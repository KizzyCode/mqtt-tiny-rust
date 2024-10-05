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
    decoded: u16,
}
impl Good {
    /// Good encoded/decoded pairs
    pub const fn all() -> &'static [Self] {
        &[
            Self { encoded: &[0x00, 0x00], decoded: 0x00_00 },
            Self { encoded: &[0x00, 0x04], decoded: 0x00_04 },
            Self { encoded: &[0x07, 0x00], decoded: 0x07_00 },
            Self { encoded: &[0xFF, 0xFF], decoded: 0xFF_FF },
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
            // Truncated shorts
            Self { encoded: &[] },
            Self { encoded: &[0x04] },
        ]
    }
}

/// Tests successful decoding
#[test]
pub fn decode() {
    for test_vector in Good::all() {
        // Decode and validate
        let encoded = test_vector.encoded.iter().copied();
        let decoded = Decoder::new(encoded).u16().expect("Failed to decode valid short");
        assert_eq!(decoded, test_vector.decoded, "Invalid decoded short")
    }
}

/// Tests successful decoding
#[test]
pub fn decode_optional() {
    for test_vector in Good::all() {
        // Decode and validate None
        let encoded = test_vector.encoded.iter().copied();
        let decoded = Decoder::new(encoded).optional_u16(false).expect("Failed to decode valid short");
        assert!(decoded.is_none(), "Invalid decoded short");

        // Decode and validate Some
        let encoded = test_vector.encoded.iter().copied();
        let decoded = Decoder::new(encoded)
            .optional_u16(true)
            .expect("Failed to decode valid short")
            .expect("Failed to unwrap valid short");
        assert_eq!(decoded, test_vector.decoded, "Invalid decoded short")
    }
}

/// Tests successful encoding
#[test]
pub fn encode() {
    for test_vector in Good::all() {
        // Encode length
        let length: usize = Length::new().u16(&test_vector.decoded).into();

        // Encode and validate
        let decoded = Encoder::default().u16(test_vector.decoded);
        let encoded: Vec = decoded.into_iter().collect();
        assert_eq!(encoded.deref(), test_vector.encoded, "Invalid encoded short");
        assert_eq!(length, test_vector.encoded.len(), "Invalid encoded length");
    }
}

/// Tests successful encoding
#[test]
pub fn encode_optional() {
    for test_vector in Good::all() {
        // Encode and validate None
        let decoded = Encoder::default().optional_u16(None);
        let encoded: Vec = decoded.into_iter().collect();
        assert_eq!(encoded.deref(), b"", "Invalid encoded short");

        // Encode and validate Some
        let decoded = Encoder::default().optional_u16(Some(test_vector.decoded));
        let encoded: Vec = decoded.into_iter().collect();
        assert_eq!(encoded.deref(), test_vector.encoded, "Invalid encoded short");
    }
}

/// Tests failing decoding
#[test]
pub fn decode_invalid() {
    for test_vector in BadEncoded::all() {
        // Decode and validate
        let encoded = test_vector.encoded.iter().copied();
        let decoded = Decoder::new(encoded).u16();
        assert!(decoded.is_err(), "Unexpected success when decoding invalid short");
    }
}
