#![cfg(any(feature = "std", feature = "arrayvec"))]

use mqtt_tiny::anyvec::AnyVec;
use mqtt_tiny::coding::length::Length;
use mqtt_tiny::coding::{Decoder, Encoder};
use std::ops::Deref;

// Select an appropriate vector type
#[cfg(feature = "std")]
type Vec<T> = std::vec::Vec<T>;
#[cfg(all(not(feature = "std"), feature = "arrayvec"))]
type Vec<T> = arrayvec::ArrayVec<T, 64>;

/// A test vector for known-good encoded/decoded pairs
#[derive(Debug, Clone)]
pub struct Good {
    /// The encoded representation
    encoded: Vec<u8>,
    /// The decoded representation
    decoded: Vec<(Vec<u8>, u8)>,
}
impl Good {
    /// Good encoded/decoded pairs
    pub fn all() -> [Self; 4] {
        [
            // An empty topics
            Self::new(&[0x00, 0x00, 0x00], [(b"", 0)]),
            // Two empty topics
            Self::new(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00], [(b"", 0), (b"", 0)]),
            // An example field
            Self::new(b"\x00\x09Testolope\x07", [(b"Testolope", 7)]),
            // Two example fields
            Self::new(b"\x00\x04Test\x04\x00\x05olope\x07", [(b"Test", 4), (b"olope", 7)]),
        ]
    }

    /// Creates a new test vector
    fn new<const SIZE: usize>(encoded: &[u8], decoded: [(&[u8], u8); SIZE]) -> Self {
        // Create vecs from test pairs
        let encoded = AnyVec::new(encoded).expect("Failed to create test vector");
        let mut decoded_: Vec<(Vec<u8>, u8)> = Default::default();
        for (topic, qos) in decoded {
            // Create vec from each decoded topic
            let topic = AnyVec::new(topic).expect("Failed to create test vector");
            decoded_.push((topic, qos));
        }

        // Init self
        Self { encoded, decoded: decoded_ }
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
            // A truncated data field sequence
            Self { encoded: &[0x00, 0x01, 0xFF, 0x00, 0x04, 0xFF] },
        ]
    }
}

/// Tests successful decoding
#[test]
pub fn decode() {
    for test_vector in Good::all() {
        // Decode and validate
        let encoded = test_vector.encoded.iter().copied();
        let decoded: Vec<(Vec<u8>, u8)> =
            Decoder::new(encoded).peekable().topics_qos().expect("Failed to decode valid topics+QoS sequence");
        assert_eq!(decoded.deref(), test_vector.decoded.as_slice(), "Invalid decoded topics+QoS sequence")
    }
}

/// Tests successful encoding
#[test]
pub fn encode() {
    for test_vector in Good::all() {
        // Encode length
        let length: usize = Length::new().topics_qos(&test_vector.decoded).into();

        // Encode and validate
        let encoded = Encoder::default().topics_qos(test_vector.decoded);
        let encoded: Vec<u8> = encoded.into_iter().collect();
        assert_eq!(encoded.deref(), test_vector.encoded.as_slice(), "Invalid encoded topics+QoS sequence");
        assert_eq!(length, test_vector.encoded.len(), "Invalid encoded length");
    }
}

/// Tests failing decoding
#[test]
pub fn decode_invalid() {
    for test_vector in BadEncoded::all() {
        // Decode and validate
        let encoded = test_vector.encoded.iter().copied();
        let decoded: Result<Vec<Vec<u8>>, _> = Decoder::new(encoded).peekable().topics();
        assert!(decoded.is_err(), "Unexpected success when decoding invalid topics+QoS sequence");
    }
}
