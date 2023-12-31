/// Unwraps an result and asserts that it is equal to a pattern
macro_rules! assert_ok {
    ($result:expr) => {
        assert_eq!(($result).unwrap(), ())
    };
    ($result:expr, $expected:expr) => {
        assert_eq!(($result).unwrap(), $expected)
    };
}

/// Asserts that an expression matches a given pattern
macro_rules! assert_err {
    ($expression:expr, $error_kind:expr) => {
        assert_eq!($expression.unwrap_err().kind(), &$error_kind)
    };
}

/// Tests `Reader`-related methods
mod reader {
    use std::io::Cursor;

    use mqtt_tiny::{coding::Reader, error::ErrorKind};

    /// Performs a reader operation
    macro_rules! reader {
        ($fn:ident: $bytes:expr, with: $($value:expr),+) => {{
            let mut reader = mqtt_tiny::coding::Reader::new(std::io::Cursor::new($bytes)).buffered();
            reader.$fn($($value),+)
        }};
        ($fn:ident: $bytes:expr) => {{
            let mut reader = mqtt_tiny::coding::Reader::new(std::io::Cursor::new($bytes)).buffered();
            reader.$fn()
        }};
    }

    /// Gets a byte
    #[test]
    fn read_u8() {
        // Success
        let read = reader!(read_u8: b"Testolope");
        assert_ok!(read, b'T');

        // Error
        let read = reader!(read_u8: b"");
        assert_err!(read, ErrorKind::TruncatedData);
    }

    /// Reads the packet header and validates the expected packet type and returns the flags
    #[test]
    fn read_header() {
        // Success
        let read = reader!(read_header: b"\x42", with: &4);
        assert_ok!(read, [false, false, true, false]);

        // Error (invalid type)
        let read = reader!(read_header: b"\x42", with: &7);
        assert_err!(read, ErrorKind::InvalidValue);

        // Error (truncated)
        let read = reader!(read_header: b"", with: &4);
        assert_err!(read, ErrorKind::TruncatedData);
    }

    /// Gets a flag-octet
    #[test]
    fn read_flags() {
        // Success
        let read = reader!(read_flags: b"\x74");
        assert_ok!(read, [false, true, true, true, false, true, false, false]);

        // Error
        let read = reader!(read_flags: b"");
        assert_err!(read, ErrorKind::TruncatedData);
    }

    /// Gets the packet length
    #[test]
    fn read_packetlen() {
        // Success
        let read = reader!(read_packetlen: b"\x07");
        assert_ok!(read, 7);

        // Success
        let read = reader!(read_packetlen: b"\xff\xff\xff\x5f");
        assert_ok!(read, 268435423);

        // Error (too long)
        let read = reader!(read_packetlen: b"\xff\xff\xff\xff");
        assert_err!(read, ErrorKind::InvalidValue);

        // Error (truncated)
        let read = reader!(read_packetlen: b"");
        assert_err!(read, ErrorKind::TruncatedData);
    }

    /// Gets an array
    #[test]
    fn read_array() {
        // Success
        let read = reader!(read_array: b"\xde\xad\xbe\xef");
        assert_ok!(read, [0xde, 0xad, 0xbe, 0xef]);

        // Error
        let read: Result<[u8; 1], _> = reader!(read_array: b"");
        assert_err!(read, ErrorKind::TruncatedData);
    }

    /// Gets an `u16`
    #[test]
    fn read_u16() {
        // Success
        let read = reader!(read_u16: b"\xbe\xef");
        assert_ok!(read, 0xbeef);

        // Error
        let read = reader!(read_u16: b"\xde");
        assert_err!(read, ErrorKind::TruncatedData);
    }

    /// Gets an optional `u16` field if the condition is true
    #[test]
    pub fn read_optional_u16() {
        // Some
        let read = reader!(read_optional_u16: b"\xbe\xef", with: true);
        assert_ok!(read, Some(0xbeef));

        // None
        let read = reader!(read_optional_u16: b"\xbe\xef", with: false);
        assert_ok!(read, None);
    }

    /// Gets a length-prefixed byte field
    #[test]
    fn read_bytes() {
        // Success
        let read = reader!(read_bytes: b"\x00\x04\xde\xad\xbe\xef");
        assert_ok!(read, vec![0xde, 0xad, 0xbe, 0xef]);

        // Error (truncated length)
        let read = reader!(read_bytes: b"\x00");
        assert_err!(read, ErrorKind::TruncatedData);

        // Error (truncated)
        let read = reader!(read_bytes: b"\x00\x04\xde\xad\xbe");
        assert_err!(read, ErrorKind::TruncatedData);
    }

    /// Gets a length-prefixed string
    #[test]
    fn read_string() {
        // Success
        let read = reader!(read_string: b"\x00\x09Testolope");
        assert_ok!(read, "Testolope".to_string());

        // Error (truncated length)
        let read = reader!(read_string: b"\x00");
        assert_err!(read, ErrorKind::TruncatedData);

        // Error (truncated)
        let read = reader!(read_string: b"\x00\x09Testolop");
        assert_err!(read, ErrorKind::TruncatedData);
    }

    /// Gets an optional length-prefixed byte field if the condition is true
    #[test]
    fn read_optional_bytes() {
        // Some
        let read = reader!(read_optional_bytes: b"\x00\x04\xde\xad\xbe\xef", with: true);
        assert_ok!(read, Some(vec![0xde, 0xad, 0xbe, 0xef]));

        // None
        let read = reader!(read_optional_bytes: b"\x00\x04\xde\xad\xbe\xef", with: false);
        assert_ok!(read, None);
    }

    /// Gets an optional length-prefixed string if the condition is true
    #[test]
    fn read_optional_string() {
        // Some
        let read = reader!(read_optional_string: b"\x00\x09Testolope", with: true);
        assert_ok!(read, Some("Testolope".to_string()));

        // None
        let read = reader!(read_optional_string: b"\x00\x09Testolope", with: false);
        assert_ok!(read, None);
    }

    /// Reads an expected constant value and ensures that the input matches the expectation
    #[test]
    fn read_constant() {
        // Success
        let read = reader!(read_constant: b"\x07", with: b"\x07");
        assert_ok!(read);

        // Error
        let read = reader!(read_constant: b"\x07", with: b"\x04");
        assert_err!(read, ErrorKind::InvalidValue);
    }

    /// Reads an expected version constant and ensures that the input matches the expectation
    ///
    /// # Note
    /// An unsupported protocol might require graceful handling, so this function returns an
    /// `ErrorKind::UnsupportedVersion` on mismatch
    #[test]
    fn read_version_constant() {
        // Success
        let read = reader!(read_version_constant: b"\x07", with: b"\x07");
        assert_ok!(read);

        // Error
        let read = reader!(read_version_constant: b"\x07", with: b"\x04");
        assert_err!(read, ErrorKind::UnsupportedVersion);
    }

    /// Reads the remaining bytes
    ///
    /// # Warning
    /// Usually this function should be called on a limited reader only, otherwise it might accidentally read into the next
    /// packet. Furthermore, reading an unlimited amount of bytes into memory poses a serious DoS risk.
    #[test]
    pub fn read_remaining() {
        let read = reader!(read_remaining: b"Testolope");
        assert_ok!(read, b"Testolope".to_vec());
    }

    /// Peeks at the next pending byte without consuming it from the source
    #[test]
    pub fn peek_u8() {
        // Create reader
        let mut reader = Reader::new(Cursor::new(b"Testolope")).buffered();
        let read = reader.peek_u8();
        assert_ok!(read, b'T');

        // Ensure the reader still contains the entire message
        let read = reader.read_remaining();
        assert_ok!(read, b"Testolope".to_vec());
    }

    /// Tests whether the reader is exhausted or not
    #[test]
    pub fn is_empty() {
        // Non-empty
        let status = reader!(is_empty: b"Testolope");
        assert_ok!(status, false);

        // Empty
        let status = reader!(is_empty: b"");
        assert_ok!(status, true);
    }

    /// Reads concatenated `16-bit-length || topic` topic-filter-blobs **until the source is exhausted**
    ///
    /// # Warning
    /// Usually this function should be called on a limited reader only, otherwise it might accidentally read into the next
    /// packet
    #[test]
    fn read_topic_seq() {
        // Success
        let read = reader!(read_topic_seq: b"\x00\x04Test\x00\x05olope");
        assert_ok!(read, vec!["Test".to_string(), "olope".to_string()]);

        // Error (truncated length)
        let invalid = reader!(read_topic_seq: b"\x00\x04Test\x00");
        assert_err!(invalid, ErrorKind::TruncatedData);

        // Error (truncated)
        let invalid = reader!(read_topic_seq: b"\x00\x04Test\x00\x05olop");
        assert_err!(invalid, ErrorKind::TruncatedData);
    }

    /// Reads concatenated `16-bit-length || topic || qos` topic-filter-blobs **until the source is exhausted**
    /// (`(filter, qos)`)
    ///
    /// # Warning
    /// Usually this function should be called on a limited reader only, otherwise it might accidentally read into the next
    /// packet
    #[test]
    fn read_topic_qos_seq() {
        // Success
        let read = reader!(read_topic_qos_seq: b"\x00\x04Test\x01\x00\x05olope\x02");
        #[rustfmt::skip]
        assert_ok!(read, vec![
            ("Test".to_string(), 1),
            ("olope".to_string(), 2)
        ]);

        // Error (truncated tuple)
        let invalid = reader!(read_topic_qos_seq: b"\x00\x04Test\x01\x00\x05olope");
        assert_err!(invalid, ErrorKind::TruncatedData);

        // Error (truncated length)
        let invalid = reader!(read_topic_qos_seq: b"\x00\x04Test\x01\x00");
        assert_err!(invalid, ErrorKind::TruncatedData);

        // Error (truncated)
        let invalid = reader!(read_topic_qos_seq: b"\x00\x04Test\x01\x00\x05olop");
        assert_err!(invalid, ErrorKind::TruncatedData);
    }
}

/// Tests `Writer`-related methods
mod writer {
    /// Performs a writer operation
    macro_rules! writer {
        ($fn:ident: $($value:expr),+) => {{
            let writer = mqtt_tiny::coding::Writer::new(Vec::new());
            writer.$fn($($value),+).and_then(|w| w.finalize())
        }};
    }

    /// Writes a raw buffer directly as-is to the underlying sink
    #[test]
    pub fn write_raw() {
        let written = writer!(write_raw: b"Testolope");
        assert_ok!(written, b"Testolope".to_vec())
    }

    /// Writes a byte
    #[test]
    fn write_u8() {
        let written = writer!(write_u8: 0x07);
        assert_ok!(written, vec![0x07])
    }

    /// Writes a packet header
    #[test]
    fn write_header() {
        let written = writer!(write_header: 0x04, [false, false, true, false]);
        assert_ok!(written, vec![0x42])
    }

    /// Writes a flag-octet
    #[test]
    fn write_flags() {
        let written = writer!(write_flags: [false, true, true, true, false, true, false, false]);
        assert_ok!(written, vec![0x74])
    }

    /// Writes a packet lengrh
    #[test]
    fn write_packetlen() {
        // Short length
        let written = writer!(write_packetlen: 7);
        assert_ok!(written, vec![0x07]);

        // Long length
        let written = writer!(write_packetlen: 268435423);
        assert_ok!(written, vec![0xff, 0xff, 0xff, 0x5f]);
    }

    /// Writes a packet lengrh
    #[test]
    #[should_panic]
    fn write_packetlen_panic() {
        const _2_POW_28: usize = 268435456;
        let _ = writer!(write_packetlen: _2_POW_28);
    }

    /// Writes an array
    #[test]
    fn write_array() {
        let written = writer!(write_array: *b"\xde\xad\xbe\xef");
        assert_ok!(written, vec![0xde, 0xad, 0xbe, 0xef]);
    }

    /// Writes a `u16`
    #[test]
    fn write_u16() {
        let written = writer!(write_u16: 0xbeef);
        assert_ok!(written, vec![0xbe, 0xef]);
    }

    /// Writes an optional `u16`
    #[test]
    pub fn write_optional_u16() {
        // Some
        let written = writer!(write_optional_u16: Some(0xbeef));
        assert_ok!(written, vec![0xbe, 0xef]);

        // None
        let written = writer!(write_optional_u16: None);
        assert_ok!(written, vec![]);
    }

    /// Writes a length-prefixed byte field
    ///
    /// # Panics
    /// This function panics if `bytes.len()` is greater than `u16::MAX`
    #[test]
    fn write_bytes() {
        let written = writer!(write_bytes: vec![0xde, 0xad, 0xbe, 0xef]);
        assert_ok!(written, vec![0x00, 0x04, 0xde, 0xad, 0xbe, 0xef]);
    }

    /// Writes a length-prefixed byte field
    ///
    /// # Panics
    /// This function panics if `bytes.len()` is greater than `u16::MAX`
    #[test]
    #[should_panic]
    fn write_bytes_panic() {
        let very_long_vec = vec![0x07; (u16::MAX as usize) + 1];
        let _ = writer!(write_bytes: very_long_vec);
    }

    /// Writes a length-prefixed string
    ///
    /// # Panics
    /// This function panics if `string.len()` is greater than `u16::MAX`
    #[test]
    fn write_string() {
        let written = writer!(write_string: "Testolope".to_string());
        assert_ok!(written, b"\x00\x09Testolope".to_vec());
    }

    /// Writes a length-prefixed string
    ///
    /// # Panics
    /// This function panics if `string.len()` is greater than `u16::MAX`
    #[test]
    #[should_panic]
    fn write_string_panic() {
        let very_long_vec = vec![b'x'; (u16::MAX as usize) + 1];
        let very_long_string = String::from_utf8(very_long_vec).unwrap();
        let _ = writer!(write_string: very_long_string);
    }

    /// Writes an optional length-prefixed byte field
    ///
    /// # Panics
    /// This function panics if `bytes.len()` is greater than `u16::MAX`
    #[test]
    fn write_optional_bytes() {
        // Some
        let written = writer!(write_optional_bytes: Some(vec![0xde, 0xad, 0xbe, 0xef]));
        assert_ok!(written, vec![0x00, 0x04, 0xde, 0xad, 0xbe, 0xef]);

        // None
        let written = writer!(write_optional_bytes: None);
        assert_ok!(written, vec![]);
    }

    /// Writes an optional length-prefixed string
    ///
    /// # Panics
    /// This function panics if `string.len()` is greater than `u16::MAX`
    #[test]
    fn write_optional_string() {
        // Some
        let written = writer!(write_optional_string: Some("Testolope".to_string()));
        assert_ok!(written, vec![0x00, 0x09, b'T', b'e', b's', b't', b'o', b'l', b'o', b'p', b'e']);

        // None
        let written = writer!(write_optional_string: None);
        assert_ok!(written, vec![]);
    }

    /// Writes a topic list as concatenated `16-bit-length || topic`-blobs
    ///
    /// # Panics
    /// This function panics if a topic length is greater than `u16::MAX`
    #[test]
    fn write_topic_seq() {
        let seq = vec!["Test".to_string(), "olope".to_string()];
        let written = writer!(write_topic_seq: seq);
        assert_ok!(written, b"\x00\x04Test\x00\x05olope".to_vec())
    }

    /// Writes a topic list as concatenated `16-bit-length || topic || qos`-blobs
    ///
    /// # Panics
    /// This function panics if a topic length is greater than `u16::MAX`
    #[test]
    fn write_topic_qos_seq() {
        #[rustfmt::skip]
        let seq = vec![
            ("Test".to_string(), 1),
            ("olope".to_string(), 2)
        ];
        let written = writer!(write_topic_qos_seq: seq);
        assert_ok!(written, b"\x00\x04Test\x01\x00\x05olope\x02".to_vec())
    }
}

/// Tests `Length`-related methods
mod length {
    /// Performs a length operation
    macro_rules! length {
        ($fn:ident: $($value:expr),+) => {{
            let length = mqtt_tiny::coding::Length::new();
            length.$fn($($value),+).finalize()
        }};
    }

    /// Adds a raw buffer directly as-is
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    #[test]
    pub fn add_raw() {
        let len = length!(add_raw: b"Testolope");
        assert_eq!(len, 9);
    }

    /// Adds a `u8`
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    #[test]
    fn add_u8() {
        let len = length!(add_u8: &0x07);
        assert_eq!(len, 1);
    }

    /// Adds a flag field
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    #[test]
    fn add_flags() {
        let len = length!(add_flags: &[false, true, true, true, false, true, false, false]);
        assert_eq!(len, 1);
    }

    /// Adds an array
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    #[test]
    fn add_array() {
        let len = length!(add_array: b"Testolope");
        assert_eq!(len, 9);
    }

    /// Adds a `u16`
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    #[test]
    fn add_u16() {
        let len = length!(add_u16: &0xbeef);
        assert_eq!(len, 2);
    }

    /// Adds an optional `u16`
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    #[test]
    pub fn add_optional_u16() {
        // Some
        let len = length!(add_optional_u16: &Some(0xbeef));
        assert_eq!(len, 2);

        // None
        let len = length!(add_optional_u16: &None);
        assert_eq!(len, 0);
    }

    /// Adds a length-prefixed byte field
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    #[test]
    fn add_bytes() {
        let len = length!(add_bytes: &vec![0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(len, 6);
    }

    /// Adds a length-prefixed string
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    #[test]
    fn add_string() {
        let len = length!(add_string: &"Testolope".to_string());
        assert_eq!(len, 11);
    }

    /// Adds an optional length-prefixed byte field
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    #[test]
    fn add_optional_bytes() {
        // Some
        let len = length!(add_optional_bytes: &Some(vec![0xde, 0xad, 0xbe, 0xef]));
        assert_eq!(len, 6);

        // None
        let len = length!(add_optional_bytes: &None);
        assert_eq!(len, 0);
    }

    /// Adds an optional length-prefixed string
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    #[test]
    fn add_optional_string() {
        // Some
        let len = length!(add_optional_string: &Some("Testolope".to_string()));
        assert_eq!(len, 11);

        // None
        let len = length!(add_optional_string: &None);
        assert_eq!(len, 0);
    }

    /// Adds a topic list as concatenated `16-bit-length || topic`-blobs
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    #[test]
    fn add_topic_seq() {
        let seq = vec!["Test".to_string(), "olope".to_string()];
        let len = length!(add_topic_seq: &seq);
        assert_eq!(len, 13);
    }

    /// Adds a topic list as concatenated `16-bit-length || topic || qos`-blobs
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    #[test]
    fn add_topic_qos_seq() {
        #[rustfmt::skip]
        let seq = vec![
            ("Test".to_string(), 1),
            ("olope".to_string(), 2)
        ];
        let len = length!(add_topic_qos_seq: &seq);
        assert_eq!(len, 15)
    }
}
