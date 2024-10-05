#![cfg(any(feature = "std", feature = "arrayvec"))]

use core::ops::Deref;
use mqtt_tiny::{
    anyvec::AnyVec,
    coding::{length::Length, Decoder, Encoder},
};

// Select an appropriate vector type
#[cfg(feature = "std")]
type Vec = std::vec::Vec<u8>;
#[cfg(all(not(feature = "std"), feature = "arrayvec"))]
type Vec = arrayvec::ArrayVec<u8, { 65 * 1024 }>;

/// A test vector for known-good encoded/decoded pairs
#[derive(Debug, Clone)]
pub struct Good {
    /// The raw representation (encoded and decoded is the same)
    raw: Vec,
}
impl Good {
    /// Good encoded/decoded pairs
    pub fn all() -> [Self; 5] {
        [
            // An empty blob
            Self::new(&[]),
            // An 1 blob
            Self::new(&[0x01]),
            // An example blob
            Self::new(b"Testolope"),
            // An 255 blob
            Self::new(&[0x04; 255]),
            // A very long blob
            Self::new(&[0x07; 65_535]),
        ]
    }

    /// Creates a new test vector
    fn new(raw: &[u8]) -> Self {
        let raw = AnyVec::new(raw).expect("Failed to create test vector");
        Self { raw }
    }
}

/// Tests successful decoding
#[test]
pub fn decode() {
    for test_vector in Good::all() {
        // Encode length
        let length: usize = Length::new().raw(&test_vector.raw).into();

        // Decode and validate
        let encoded = test_vector.raw.iter().copied();
        let decoded: Vec = Decoder::new(encoded).raw_remainder().expect("Failed to decode valid raw data");
        assert_eq!(decoded.deref(), test_vector.raw.as_slice(), "Invalid decoded raw data");
        assert_eq!(length, test_vector.raw.len(), "Invalid encoded length");
    }
}

/// Tests successful encoding
#[test]
pub fn encode() {
    for test_vector in Good::all() {
        // Encode and validate
        let encoded = Encoder::default().raw(test_vector.raw.clone());
        let encoded: Vec = encoded.into_iter().collect();
        assert_eq!(encoded.deref(), test_vector.raw.as_slice(), "Invalid encoded raw data")
    }
}
