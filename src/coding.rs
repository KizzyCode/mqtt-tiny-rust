//! Packet coding

use crate::error::{ErrorKind, MqttError};
use std::io::{BufRead, BufReader, Read, Take, Write};

/// A reader
#[derive(Debug)]
#[repr(transparent)]
pub struct Reader<T> {
    /// The source to read from
    source: T,
}
impl<T> Reader<T> {
    /// Creates a new reader
    pub fn new(source: T) -> Self {
        Self { source }
    }
}
impl<T> Reader<T>
where
    T: Read,
{
    /// Creates a length-limited variant of `self`
    pub fn limit(self, len: usize) -> Reader<Take<T>> {
        let reader = self.source.take(len as u64);
        Reader { source: reader }
    }
    /// Creates a buffered and subsequently peekable variant of `self`
    pub fn buffered(self) -> Reader<BufReader<T>> {
        let reader = BufReader::with_capacity(1, self.source);
        Reader { source: reader }
    }

    /// Gets a byte
    pub fn read_u8(&mut self) -> Result<u8, MqttError> {
        let mut buf = [0];
        self.source.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    /// Reads the packet header and validates the expected packet type and returns the flags
    pub fn read_header(&mut self, expected: &u8) -> Result<[bool; 4], MqttError> {
        // Get byte and type
        let byte = self.read_u8()?;
        let true = byte >> 4 == *expected else {
            return Err(ErrorKind::InvalidValue.into());
        };

        // Get flags
        #[rustfmt::skip]
        let flags = [
            byte & 0b00001000 != 0,
            byte & 0b00000100 != 0,
            byte & 0b00000010 != 0,
            byte & 0b00000001 != 0
        ];
        Ok(flags)
    }

    /// Gets a flag-octet
    pub fn read_flags(&mut self) -> Result<[bool; 8], MqttError> {
        let byte = self.read_u8()?;
        Ok([
            byte & 0b10000000 != 0,
            byte & 0b01000000 != 0,
            byte & 0b00100000 != 0,
            byte & 0b00010000 != 0,
            byte & 0b00001000 != 0,
            byte & 0b00000100 != 0,
            byte & 0b00000010 != 0,
            byte & 0b00000001 != 0,
        ])
    }

    /// Gets the packet length
    pub fn read_packetlen(&mut self) -> Result<usize, MqttError> {
        // Parse length
        let mut len = 0;
        for (pos, maybe_byte) in (&mut self.source).bytes().enumerate() {
            // Decode next length byte
            let byte = maybe_byte?;
            len <<= 7;
            len |= (byte & 0b0111_1111) as usize;

            // Validate coding
            match byte & 0b1000_0000 {
                0b1000_0000 if pos < 3 => continue,
                0b1000_0000 => return Err(ErrorKind::InvalidValue.into()),
                _ => return Ok(len),
            }
        }
        Err(ErrorKind::TruncatedData.into())
    }

    /// Gets an array
    pub fn read_array<const SIZE: usize>(&mut self) -> Result<[u8; SIZE], MqttError> {
        let mut array = [0; SIZE];
        self.source.read_exact(&mut array)?;
        Ok(array)
    }

    /// Gets an `u16`
    pub fn read_u16(&mut self) -> Result<u16, MqttError> {
        let bytes = self.read_array()?;
        Ok(u16::from_be_bytes(bytes))
    }

    /// Gets an optional `u16` field if the condition is true
    pub fn read_optional_u16(&mut self, condition: bool) -> Result<Option<u16>, MqttError> {
        match condition {
            true => self.read_u16().map(Some),
            false => Ok(None),
        }
    }

    /// Gets a length-prefixed byte field
    pub fn read_bytes(&mut self) -> Result<Vec<u8>, MqttError> {
        // Get length and allocate vector
        // Note: Since the length is implicitly limited to `u16::MAX`, this is DoS-safe
        let len = self.read_u16()? as usize;
        let mut bytes = vec![0; len];

        // Read bytes
        self.source.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    /// Gets a length-prefixed string
    pub fn read_string(&mut self) -> Result<String, MqttError> {
        let bytes = self.read_bytes()?;
        String::from_utf8(bytes).map_err(|_| ErrorKind::InvalidValue.into())
    }

    /// Gets an optional length-prefixed byte field if the condition is true
    pub fn read_optional_bytes(&mut self, condition: bool) -> Result<Option<Vec<u8>>, MqttError> {
        match condition {
            true => self.read_bytes().map(Some),
            false => Ok(None),
        }
    }

    /// Gets an optional length-prefixed string if the condition is true
    pub fn read_optional_string(&mut self, condition: bool) -> Result<Option<String>, MqttError> {
        match condition {
            true => self.read_string().map(Some),
            false => Ok(None),
        }
    }

    /// Reads an expected constant value and ensures that the input matches the expectation
    pub fn read_constant(&mut self, expected: &[u8]) -> Result<(), MqttError> {
        // Validate the bytes
        for expected in expected {
            // Read and validate the next byte
            let next = self.read_u8()?;
            let true = next == *expected else {
                return Err(ErrorKind::InvalidValue.into());
            };
        }
        Ok(())
    }

    /// Reads an expected version constant and ensures that the input matches the expectation
    ///
    /// # Note
    /// An unsupported protocol might require graceful handling, so this function returns an
    /// `MqttError::UnsupportedVersion` on mismatch
    pub fn read_version_constant(&mut self, expected: &[u8]) -> Result<(), MqttError> {
        match self.read_constant(expected) {
            Err(e) if e.kind() == &ErrorKind::InvalidValue => Err(ErrorKind::UnsupportedVersion.into()),
            Err(e) => Err(e),
            Ok(ok) => Ok(ok),
        }
    }

    /// Reads the remaining bytes
    ///
    /// # Warning
    /// Usually this function should be called on a limited reader only, otherwise it might accidentally read into the next
    /// packet. Furthermore, reading an unlimited amount of bytes into memory poses a serious DoS risk.
    pub fn read_remaining(&mut self) -> Result<Vec<u8>, MqttError> {
        let mut buf = Vec::new();
        self.source.read_to_end(&mut buf)?;
        Ok(buf)
    }
}
impl<T> Reader<BufReader<T>>
where
    T: Read,
{
    /// Peeks at the next pending byte without consuming it from the source
    pub fn peek_u8(&mut self) -> Result<u8, MqttError> {
        match self.source.fill_buf()? {
            [byte, ..] => Ok(*byte),
            _ => Err(ErrorKind::TruncatedData.into()),
        }
    }

    /// Tests whether the reader is exhausted or not
    pub fn is_empty(&mut self) -> Result<bool, MqttError> {
        let buf = self.source.fill_buf()?;
        Ok(buf.is_empty())
    }

    /// Reads concatenated `16-bit-length || topic` topic-filter-blobs **until the source is exhausted**
    ///
    /// # Warning
    /// Usually this function should be called on a limited reader only, otherwise it might accidentally read into the next
    /// packet
    pub fn read_topic_seq(&mut self) -> Result<Vec<String>, MqttError> {
        // Read all filters
        let mut filters = Vec::new();
        while !self.is_empty()? {
            // Read topic filters
            let filter = self.read_string()?;
            filters.push(filter)
        }
        Ok(filters)
    }

    /// Reads concatenated `16-bit-length || topic || qos` topic-filter-blobs **until the source is exhausted**
    /// (`(filter, qos)`)
    ///
    /// # Warning
    /// Usually this function should be called on a limited reader only, otherwise it might accidentally read into the next
    /// packet
    pub fn read_topic_qos_seq(&mut self) -> Result<Vec<(String, u8)>, MqttError> {
        // Read all filters
        let mut filters = Vec::new();
        while !self.is_empty()? {
            // Read `(filter, qos)`-tuple
            let filter = self.read_string()?;
            let qos = self.read_u8()?;
            filters.push((filter, qos))
        }
        Ok(filters)
    }
}
impl<T> Read for Reader<T>
where
    T: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.source.read(buf)
    }
}
impl<T> BufRead for Reader<T>
where
    T: BufRead,
{
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.source.fill_buf()
    }
    fn consume(&mut self, amt: usize) {
        self.source.consume(amt)
    }
}

/// A writer
#[derive(Debug)]
#[repr(transparent)]
pub struct Writer<T> {
    /// The sink to write to
    sink: T,
}
impl<T> Writer<T> {
    /// Creates a new writer
    pub const fn new(sink: T) -> Self {
        Self { sink }
    }
}
impl<T> Writer<T>
where
    T: Write,
{
    /// Finalizes the writer, yielding the underlying sink
    pub fn finalize(mut self) -> Result<T, MqttError> {
        self.sink.flush()?;
        Ok(self.sink)
    }

    /// Writes a raw buffer directly as-is to the underlying sink
    pub fn write_raw<A>(mut self, raw: A) -> Result<Self, MqttError>
    where
        A: AsRef<[u8]>,
    {
        self.sink.write_all(raw.as_ref())?;
        Ok(self)
    }

    /// Writes a byte
    pub fn write_u8(self, byte: u8) -> Result<Self, MqttError> {
        self.write_raw([byte])
    }

    /// Writes a packet header
    pub fn write_header(self, type_: u8, bits: [bool; 4]) -> Result<Self, MqttError> {
        #[rustfmt::skip]
        let byte = type_ << 4
            | ((bits[0] as u8) << 3)
            | ((bits[1] as u8) << 2)
            | ((bits[2] as u8) << 1)
            | (bits[3] as u8);
        self.write_u8(byte)
    }

    /// Writes a flag-octet
    pub fn write_flags(self, bits: [bool; 8]) -> Result<Self, MqttError> {
        let byte = ((bits[0] as u8) << 7)
            | ((bits[1] as u8) << 6)
            | ((bits[2] as u8) << 5)
            | ((bits[3] as u8) << 4)
            | ((bits[4] as u8) << 3)
            | ((bits[5] as u8) << 2)
            | ((bits[6] as u8) << 1)
            | (bits[7] as u8);
        self.write_u8(byte)
    }

    /// Writes a packet lengrh
    ///
    /// # Panics
    /// This function panics if `len` is greater than `2^28 - 1`
    pub fn write_packetlen(self, mut len: usize) -> Result<Self, MqttError> {
        // Encode the length in 7-bit nibbles
        let mut bytes = [0; 4];
        let mut bytes_len = 0;
        while len > 0 {
            // Push the next more-significant 7 bits to the **front**
            bytes.rotate_right(1);
            bytes[0] = (len as u8) & 0b0111_1111;

            // Mark the current byte as non-last if it is not the least-significant (i.e. trailing/firstly written) byte
            if bytes_len > 0 {
                bytes[0] |= 0b1000_0000;
            }

            // Decrement length and increment byte counter
            len >>= 7;
            // Note: This is safe since due to the left-shift-assign by `7`, the maximum amount of iterations is
            // `(64 / 7) + 1` on 64-bit systems, which always by magnitudes smaller than `usize::MAX`
            //
            // Note: The length might become larger than `bytes.len()` in case the number overflows, this is detected
            // later
            #[allow(clippy::arithmetic_side_effects)]
            (bytes_len += 1);
        }

        // Validate that we don't have an overflow
        // Note: We trust the caller here
        #[allow(clippy::expect_used)]
        let used = bytes.get(..bytes_len).expect("length larger than `2^28 - 1`?!");
        self.write_raw(used)
    }

    /// Writes an array
    pub fn write_array<const SIZE: usize>(self, array: [u8; SIZE]) -> Result<Self, MqttError> {
        self.write_raw(array)
    }

    /// Writes a `u16`
    pub fn write_u16(self, value: u16) -> Result<Self, MqttError> {
        self.write_raw(value.to_be_bytes())
    }

    /// Writes an optional `u16`
    pub fn write_optional_u16(self, value: Option<u16>) -> Result<Self, MqttError> {
        match value {
            Some(value) => self.write_u16(value),
            None => Ok(self),
        }
    }

    /// Writes a length-prefixed byte field
    ///
    /// # Panics
    /// This function panics if `bytes.len()` is greater than `u16::MAX`
    pub fn write_bytes(mut self, bytes: Vec<u8>) -> Result<Self, MqttError> {
        // Convert the length into a 16 bit length field
        // Note: We trust the caller here
        #[allow(clippy::expect_used)]
        let len = u16::try_from(bytes.len()).expect("byte field is too large?!");

        // Write the length prefix and bytes
        self = self.write_u16(len)?;
        self.write_raw(&bytes)
    }

    /// Writes a length-prefixed string
    ///
    /// # Panics
    /// This function panics if `string.len()` is greater than `u16::MAX`
    pub fn write_string(self, string: String) -> Result<Self, MqttError> {
        self.write_bytes(string.into_bytes())
    }

    /// Writes an optional length-prefixed byte field
    ///
    /// # Panics
    /// This function panics if `bytes.len()` is greater than `u16::MAX`
    pub fn write_optional_bytes(self, bytes: Option<Vec<u8>>) -> Result<Self, MqttError> {
        match bytes {
            Some(bytes) => self.write_bytes(bytes),
            None => Ok(self),
        }
    }

    /// Writes an optional length-prefixed string
    ///
    /// # Panics
    /// This function panics if `string.len()` is greater than `u16::MAX`
    pub fn write_optional_string(self, string: Option<String>) -> Result<Self, MqttError> {
        match string {
            Some(string) => self.write_string(string),
            None => Ok(self),
        }
    }

    /// Writes a topic list as concatenated `16-bit-length || topic`-blobs
    ///
    /// # Panics
    /// This function panics if a topic length is greater than `u16::MAX`
    pub fn write_topic_seq(mut self, topics: Vec<String>) -> Result<Self, MqttError> {
        for topic in topics {
            // Directly append topic varstring
            self = self.write_string(topic)?;
        }
        Ok(self)
    }

    /// Writes a topic list as concatenated `16-bit-length || topic || qos`-blobs
    ///
    /// # Panics
    /// This function panics if a topic length is greater than `u16::MAX`
    pub fn write_topic_qos_seq(mut self, topics: Vec<(String, u8)>) -> Result<Self, MqttError> {
        for (topic, qos) in topics {
            // Directly append topic varstring and QoS
            self = self.write_string(topic)?;
            self = self.write_u8(qos)?;
        }
        Ok(self)
    }
}
impl<T> Write for Writer<T>
where
    T: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.sink.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.sink.flush()
    }
}

/// A length counter
#[derive(Debug)]
#[repr(transparent)]
pub struct Length {
    /// The length
    len: usize,
}
impl Length {
    /// Creates a new length counter
    pub const fn new() -> Self {
        Self { len: 0 }
    }

    /// Finalizes the length computation
    pub fn finalize(self) -> usize {
        self.len
    }

    /// Adds a raw buffer directly as-is
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    pub fn add_raw<A>(self, raw: &A) -> Self
    where
        A: AsRef<[u8]>,
    {
        // Note: We trust the caller here
        #[allow(clippy::expect_used)]
        let len = self.len.checked_add(raw.as_ref().len()).expect("length is larger than `usize::MAX`?!");
        Self { len }
    }

    /// Adds a `u8`
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    pub fn add_u8(self, _byte: &u8) -> Self {
        // Note: We trust the caller here
        #[allow(clippy::expect_used)]
        let len = self.len.checked_add(1).expect("length is larger than `usize::MAX`?!");
        Self { len }
    }

    /// Adds a flag field
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    pub fn add_flags(self, _flags: &[bool; 8]) -> Self {
        // Note: We trust the caller here
        #[allow(clippy::expect_used)]
        let len = self.len.checked_add(1).expect("length is larger than `usize::MAX`?!");
        Self { len }
    }

    /// Adds an array
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    pub fn add_array<const SIZE: usize>(self, slice: &[u8; SIZE]) -> Self {
        // Note: We trust the caller here
        #[allow(clippy::expect_used)]
        let len = self.len.checked_add(slice.len()).expect("length is larger than `usize::MAX`?!");
        Self { len }
    }

    /// Adds a `u16`
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    pub fn add_u16(self, _value: &u16) -> Self {
        // Note: We trust the caller here
        #[allow(clippy::expect_used)]
        let len = self.len.checked_add(2).expect("length is larger than `usize::MAX`?!");
        Self { len }
    }

    /// Adds an optional `u16`
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    pub fn add_optional_u16(self, value: &Option<u16>) -> Self {
        match value {
            Some(value) => self.add_u16(value),
            None => self,
        }
    }

    /// Adds a length-prefixed byte field
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    pub fn add_bytes(self, bytes: &Vec<u8>) -> Self {
        // Note: We trust the caller here
        #[allow(clippy::expect_used)]
        let len = (self.len)
            .checked_add(2)
            .and_then(|len| len.checked_add(bytes.len()))
            .expect("length is larger than `usize::MAX`?!");
        Self { len }
    }

    /// Adds a length-prefixed string
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    pub fn add_string(self, string: &String) -> Self {
        // Note: We trust the caller here
        #[allow(clippy::expect_used)]
        let len = (self.len)
            .checked_add(2)
            .and_then(|len| len.checked_add(string.len()))
            .expect("length is larger than `usize::MAX`?!");
        Self { len }
    }

    /// Adds an optional length-prefixed byte field
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    pub fn add_optional_bytes(self, bytes: &Option<Vec<u8>>) -> Self {
        match bytes {
            Some(bytes) => self.add_bytes(bytes),
            None => self,
        }
    }

    /// Adds an optional length-prefixed string
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    pub fn add_optional_string(self, string: &Option<String>) -> Self {
        match string {
            Some(string) => self.add_string(string),
            None => self,
        }
    }

    /// Adds a topic list as concatenated `16-bit-length || topic`-blobs
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    pub fn add_topic_seq(mut self, topics: &Vec<String>) -> Self {
        for topic in topics {
            // Add string
            self = self.add_string(topic);
        }
        self
    }

    /// Adds a topic list as concatenated `16-bit-length || topic || qos`-blobs
    ///
    /// # Panics
    /// This function panics if the length would exceed `usize::MAX`
    pub fn add_topic_qos_seq(mut self, topics: &Vec<(String, u8)>) -> Self {
        for (topic, qos) in topics {
            // Add string and qos
            self = self.add_string(topic);
            self = self.add_u8(qos);
        }
        self
    }
}
