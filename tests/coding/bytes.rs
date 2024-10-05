#![cfg(any(feature = "std", feature = "arrayvec"))]

use mqtt_tiny::{
    anyvec::AnyVec,
    coding::{length::Length, Decoder, Encoder},
};
use std::ops::Deref;

// Select an appropriate vector type
#[cfg(feature = "std")]
type Vec = std::vec::Vec<u8>;
#[cfg(all(not(feature = "std"), feature = "arrayvec"))]
type Vec = arrayvec::ArrayVec<u8, { 65 * 1024 }>;

/// A test vector for known-good encoded/decoded pairs
#[derive(Debug, Clone)]
pub struct Good {
    /// The encoded representation
    encoded: Vec,
    /// The decoded representation
    decoded: Vec,
}
impl Good {
    /// Good encoded/decoded pairs
    pub fn all() -> [Self; 5] {
        [
            // An empty byte field
            Self::new(&[0x00, 0x00], &[]),
            // An 1 byte field
            Self::new(&[0x00, 0x01, 0x01], &[0x01]),
            // An example field
            Self::new(b"\x00\x09Testolope", b"Testolope"),
            // An 255 byte field
            Self::combine(&[0x00, 0xFF], &[0x04; 255]),
            // A max-length byte field
            Self::combine(&[0xFF, 0xFF], &[0x07; 65_535]),
        ]
    }

    /// Creates a new test vector
    fn new(encoded: &[u8], decoded: &[u8]) -> Self {
        let encoded = AnyVec::new(encoded).expect("Failed to create test vector");
        let decoded = AnyVec::new(decoded).expect("Failed to create test vector");
        Self { encoded, decoded }
    }
    /// Creates a test vector by combining prefix and suffix
    fn combine(prefix: &[u8; 2], suffix: &[u8]) -> Self {
        // Create suffix data
        let plain: Vec = AnyVec::new(suffix).expect("Failed to create test vector");

        // Create prefix
        let mut encoded = AnyVec::new(prefix).expect("Failed to create test vector");
        AnyVec::extend(&mut encoded, &plain).expect("Failed to create test vector");
        Self { encoded, decoded: plain }
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
            // A truncated header
            Self { encoded: &[0x00] },
            // A truncated data field
            Self { encoded: &[0x00, 0x02, 0x01] },
        ]
    }
}

/// Tests successful decoding
#[test]
pub fn decode() {
    for test_vector in Good::all() {
        // Decode and validate
        let encoded = test_vector.encoded.iter().copied();
        let decoded: Vec = Decoder::new(encoded).bytes().expect("Failed to decode valid byte field");
        assert_eq!(decoded.deref(), test_vector.decoded.as_slice(), "Invalid decoded byte field")
    }
}

/// Tests successful decoding
#[test]
pub fn decode_optional() {
    for test_vector in Good::all() {
        // Decode and validate None
        let encoded = test_vector.encoded.iter().copied();
        let decoded: Option<Vec> =
            Decoder::new(encoded).optional_bytes(false).expect("Failed to decode valid byte field");
        assert!(decoded.is_none(), "Invalid decoded byte field");

        // Decode and validate Some
        let encoded = test_vector.encoded.iter().copied();
        let decoded: Vec = Decoder::new(encoded)
            .optional_bytes(true)
            .expect("Failed to decode valid byte field")
            .expect("Failed to unwrap valid byte field");
        assert_eq!(decoded.deref(), test_vector.decoded.as_slice(), "Invalid decoded byte field")
    }
}

/// Tests successful encoding
#[test]
pub fn encode() {
    for test_vector in Good::all() {
        // Encode length
        let length: usize = Length::new().bytes(&test_vector.decoded).into();

        // Encode and validate
        let encoded = Encoder::default().bytes(test_vector.decoded);
        let encoded: Vec = encoded.into_iter().collect();
        assert_eq!(encoded.deref(), test_vector.encoded.as_slice(), "Invalid encoded byte field");
        assert_eq!(length, test_vector.encoded.len(), "Invalid encoded length");
    }
}

/// Tests successful encoding
#[test]
pub fn encode_optional() {
    for test_vector in Good::all() {
        // Encode and validate None
        let encoded = Encoder::default().optional_bytes(Option::<Vec>::None);
        let encoded: Vec = encoded.into_iter().collect();
        assert_eq!(encoded.deref(), b"", "Invalid encoded byte field");

        // Encode and validate Some
        let encoded = Encoder::default().optional_bytes(Some(test_vector.decoded));
        let encoded: Vec = encoded.into_iter().collect();
        assert_eq!(encoded.deref(), test_vector.encoded.as_slice(), "Invalid encoded byte field");
    }
}

/// Tests failing decoding
#[test]
pub fn decode_invalid() {
    for test_vector in BadEncoded::all() {
        // Decode and validate
        let encoded = test_vector.encoded.iter().copied();
        let decoded: Result<Vec, _> = Decoder::new(encoded).bytes();
        assert!(decoded.is_err(), "Unexpected success when decoding invalid byte field");
    }
}
