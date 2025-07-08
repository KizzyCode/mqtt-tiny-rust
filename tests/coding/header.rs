#![cfg(any(feature = "std", feature = "arrayvec"))]

use mqtt_tiny::coding::length::Length;
use mqtt_tiny::coding::{Decoder, Encoder};
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
    decoded: (u8, [bool; 4]),
}
impl Good {
    /// Good encoded/decoded pairs
    pub const fn all() -> &'static [Self] {
        &[
            Self { encoded: &[0b0000_0000], decoded: (0b0000, [false; 4]) },
            Self { encoded: &[0b1010_1010], decoded: (0b1010, [true, false, true, false]) },
            Self { encoded: &[0b0101_0101], decoded: (0b0101, [false, true, false, true]) },
            Self { encoded: &[0b1111_1111], decoded: (0b1111, [true; 4]) },
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
        let decoded = Decoder::new(encoded).header().expect("Failed to decode valid header");
        assert_eq!(decoded.0, test_vector.decoded.0, "Invalid decoded header");
        assert_eq!(decoded.1, test_vector.decoded.1, "Invalid decoded header");
    }
}

/// Tests successful encoding
#[test]
pub fn encode() {
    for test_vector in Good::all() {
        // Encode length
        let length: usize = Length::new().header(&test_vector.decoded.0, &test_vector.decoded.1).into();

        // Encode and validate
        let encoded = Encoder::default().header(test_vector.decoded.0, test_vector.decoded.1);
        let encoded: Vec = encoded.into_iter().collect();
        assert_eq!(encoded.deref(), test_vector.encoded, "Invalid encoded header");
        assert_eq!(length, test_vector.encoded.len(), "Invalid encoded length");
    }
}

/// Tests failing decoding
#[test]
pub fn decode_invalid() {
    for test_vector in BadEncoded::all() {
        // Decode and validate
        let encoded = test_vector.encoded.iter().copied();
        let decoded = Decoder::new(encoded).header();
        assert!(decoded.is_err(), "Unexpected success when decoding invalid header");
    }
}
