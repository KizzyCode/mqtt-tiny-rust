use core::ops::Deref;
use mqtt_tiny::{packets::TryFromIterator, Connect};

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
    decoded: Connect,
}
impl Good {
    /// Good encoded/decoded pairs
    pub fn all() -> [Self; 4] {
        [
            // A basic packet
            Self {
                encoded: b"\x10\x10\x00\x04MQTT\x04\x00\x00\x1E\x00\x04test",
                decoded: Connect::new(30, false, b"test").expect("failed to create packet"),
            },
            // A packet with a last-will
            Self {
                encoded: b"\x10\x25\x00\x04MQTT\x04\x04\x00\x1E\x00\x04test\x00\x08lastwill\x00\x09testolope",
                decoded: Connect::new(30, false, b"test").expect("failed to create packet")
                    // Set last will
                    .with_will(b"lastwill", b"testolope", 0x00, false).expect("failed to configure last will"),
            },
            // A packet with login data
            Self {
                encoded: b"\x10\x24\x00\x04MQTT\x04\xC0\x00\x1E\x00\x04test\x00\x08username\x00\x08password",
                decoded: Connect::new(30, false, b"test").expect("failed to create packet")
                    // Set login data
                    .with_username_password(b"username", b"password").expect("failed to configure login data"),
            },
            // A packet with everything and clean session
            Self {
                encoded: b"\x10\x3D\x00\x04MQTT\x04\xEE\xFF\xFF\x00\x08clientid\x00\x08lastwill\x00\x09testolope\x00\x08username\x00\x08password",
                decoded: Connect::new(65535, true, b"clientid").expect("failed to create packet")
                    // Set last will
                    .with_will(b"lastwill", b"testolope", 0x01, true).expect("failed to configure last will")
                    // Set login data
                    .with_username_password(b"username", b"password").expect("failed to configure login data"),
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
            Self { encoded: b"\x20\x10\x00\x04MQTT\x04\x00\x00\x1E\x00\x04test" },
            // Packet with invalid protocol name
            Self { encoded: b"\x10\x10\x00\x04MQTP\x04\x00\x00\x1E\x00\x04test" },
            // Packet with invalid protocol version
            Self { encoded: b"\x10\x10\x00\x04MQTT\x05\x00\x00\x1E\x00\x04test" },
            // Packet with indicated last will but missing topic/message
            Self { encoded: b"\x10\x1A\x00\x04MQTT\x04\x04\x00\x1E\x00\x04test\x00\x08lastwill" },
            // Packet with indicated but missing username
            Self { encoded: b"\x10\x10\x00\x04MQTT\x04\xC0\x00\x1E\x00\x04test" },
            // Packet with indicated but missing password
            Self { encoded: b"\x10\x10\x00\x04MQTT\x04\xC0\x00\x1E\x00\x04test" },
        ]
    }
}

/// Tests successful decoding
#[test]
pub fn decode() {
    for test_vector in Good::all() {
        // Decode and validate
        let encoded = test_vector.encoded.iter().copied();
        let decoded = Connect::try_from_iter(encoded).expect("Failed to decode valid packet");
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
        let decoded = Connect::try_from_iter(encoded);
        assert!(decoded.is_err(), "Unexpected success when decoding invalid packet");
    }
}
