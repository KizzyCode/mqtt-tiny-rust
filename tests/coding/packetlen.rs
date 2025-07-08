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
    decoded: usize,
}
impl Good {
    /// Good encoded/decoded pairs
    #[allow(clippy::unusual_byte_groupings)]
    pub const fn all() -> &'static [Self] {
        &[
            // 1-byte lengths
            Self { encoded: &[0b0_0000000], decoded: 0b0000000 },
            Self { encoded: &[0b0_1010101], decoded: 0b1010101 },
            Self { encoded: &[0b0_1111111], decoded: 0b1111111 },
            // 2-byte lengths
            Self { encoded: &[0b1_1000000, 0b0_0000000], decoded: 0b1000000_0000000 },
            Self { encoded: &[0b1_1010101, 0b0_1010101], decoded: 0b1010101_1010101 },
            Self { encoded: &[0b1_1111111, 0b0_1111111], decoded: 0b1111111_1111111 },
            // 3-byte lengths
            Self { encoded: &[0b1_1000000, 0b1_0000000, 0b0_0000000], decoded: 0b1000000_0000000_0000000 },
            Self { encoded: &[0b1_1010101, 0b1_1010101, 0b0_1010101], decoded: 0b1010101_1010101_1010101 },
            Self { encoded: &[0b1_1111111, 0b1_1111111, 0b0_1111111], decoded: 0b1111111_1111111_1111111 },
            // 4-byte lengths
            Self {
                encoded: &[0b1_1000000, 0b1_0000000, 0b1_0000000, 0b0_0000000],
                decoded: 0b1000000_0000000_0000000_0000000,
            },
            Self {
                encoded: &[0b1_1010101, 0b1_1010101, 0b1_1010101, 0b0_1010101],
                decoded: 0b1010101_1010101_1010101_1010101,
            },
            Self {
                encoded: &[0b1_1111111, 0b1_1111111, 0b1_1111111, 0b0_1111111],
                decoded: 0b1111111_1111111_1111111_1111111,
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
    #[allow(clippy::unusual_byte_groupings)]
    pub const fn all() -> &'static [Self] {
        &[
            // Truncated length
            Self { encoded: &[0b1_1000000] },
            // Length that is too long
            Self { encoded: &[0b1_1000000, 0b1_0000000, 0b1_0000000, 0b1_0000000, 0b0_0000000] },
            // Multibyte length with leading zero byte
            Self { encoded: &[0b1_0000000, 0b0_0000000] },
        ]
    }
}

/// Tests successful decoding
#[test]
pub fn decode() {
    for test_vector in Good::all() {
        // Decode and validate
        let encoded = test_vector.encoded.iter().copied();
        let decoded = Decoder::new(encoded).packetlen().expect("Failed to decode valid flags");
        assert_eq!(decoded, test_vector.decoded, "Invalid decoded flags")
    }
}

/// Tests successful encoding
#[test]
pub fn encode() {
    for test_vector in Good::all() {
        // Encode length
        let length: usize = Length::new().packetlen(&test_vector.decoded).into();

        // Encode and validate
        let encoded = Encoder::default().packetlen(test_vector.decoded);
        let encoded: Vec = encoded.into_iter().collect();
        assert_eq!(encoded.deref(), test_vector.encoded, "Invalid encoded flags");
        assert_eq!(length, test_vector.encoded.len(), "Invalid encoded length");
    }
}

/// Tests failing decoding
#[test]
pub fn decode_invalid() {
    for test_vector in BadEncoded::all() {
        // Decode and validate
        let encoded = test_vector.encoded.iter().copied();
        let decoded = Decoder::new(encoded).packetlen();
        assert!(decoded.is_err(), "Unexpected success when decoding invalid flags");
    }
}
