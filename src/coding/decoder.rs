//! An iterator-based decoder

use crate::anyvec::AnyVec;
use core::iter::{Peekable, Take};

/// An iterator-based decoder
#[derive(Debug)]
pub struct Decoder<Iter> {
    /// The underlying iterator
    source: Iter,
}
impl<Iter> Decoder<Iter> {
    /// Create a new decoder over an iterator
    pub fn new<T>(source: T) -> Self
    where
        T: IntoIterator<IntoIter = Iter>,
    {
        Self { source: source.into_iter() }
    }
}
impl<Iter> Decoder<Iter>
where
    Iter: Iterator<Item = u8>,
{
    /// Limits the decoder to the given amount of bytes
    pub fn peekable(self) -> Decoder<Peekable<Iter>> {
        Decoder { source: self.source.peekable() }
    }

    /// Limits the decoder to the given amount of bytes
    pub fn limit(self, limit: usize) -> Decoder<Take<Iter>> {
        Decoder { source: self.source.take(limit) }
    }

    /// Reads the remaining data as-is
    ///
    /// # Note
    /// This function is greedy. As raw read is unbounded by definition, this function will simply read as much data as
    /// possible until the underlying source is exhausted. Limit the source using [`Self::limit`] if necessary.
    pub fn raw_remainder<T>(&mut self) -> Result<T, &'static str>
    where
        T: AnyVec<u8>,
    {
        // Read all remaining bytes
        let mut raw = T::default();
        for byte in &mut self.source {
            // Try to append byte
            raw.push(byte)?;
        }
        Ok(raw)
    }

    /// Reads a `u8`
    pub fn u8(&mut self) -> Result<u8, &'static str> {
        self.source.next().ok_or("Truncated input")
    }

    /// Reads some raw bytes as-is into a fixed-size array
    pub fn raw<const SIZE: usize>(&mut self) -> Result<[u8; SIZE], &'static str> {
        // Fill an entire array of the requested bytes
        let mut array = [0; SIZE];
        for slot in array.iter_mut() {
            // Require next byte
            *slot = self.u8()?;
        }
        Ok(array)
    }

    /// Reads a `u16`
    pub fn u16(&mut self) -> Result<u16, &'static str> {
        let bytes = self.raw()?;
        Ok(u16::from_be_bytes(bytes))
    }

    /// Reads a length-prefixed byte field
    pub fn bytes<T>(&mut self) -> Result<T, &'static str>
    where
        T: AnyVec<u8>,
    {
        // Copy the exact amount of bytes from the source iterator
        let length = self.u16()? as usize;
        let mut bytes = T::default();
        for _ in 0..length {
            // Copy each byte
            let byte = self.u8()?;
            bytes.push(byte)?;
        }
        Ok(bytes)
    }

    /// Reads a byte as bitmap
    pub fn bitmap(&mut self) -> Result<[bool; 8], &'static str> {
        let byte = self.u8()?;
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

    /// Reads a header byte and decodes it into packet type and associated flags (as bitmap)
    pub fn header(&mut self) -> Result<(u8, [bool; 4]), &'static str> {
        let byte = self.u8()?;
        Ok((byte >> 4, [byte & 0b1000 != 0, byte & 0b0100 != 0, byte & 0b0010 != 0, byte & 0b0001 != 0]))
    }

    /// Reads a packet length field
    pub fn packetlen(&mut self) -> Result<usize, &'static str> {
        // Parse length
        let mut value = 0;
        for (pos, byte) in (&mut self.source).enumerate() {
            // Decode next length byte
            value <<= 7;
            value |= (byte & 0b0111_1111) as usize;

            // Check for end-of-length
            match byte & 0b1000_0000 {
                // Multi-byte length with a leading zero heptet
                0b1000_0000 if byte == 0b1000_0000 && value == 0 => return Err("Invalid packet length"),
                // Not the last byte but further length bytes are invalid
                0b1000_0000 if pos > 2 => return Err("Packet length is too large"),
                // Not the last byte and further length bytes are allowed
                0b1000_0000 => continue,
                // Length byte is the last byte
                _ => return Ok(value),
            }
        }

        // The packet length is truncated
        Err("Truncated input")
    }

    /// Reads an optional `u16`
    pub fn optional_u16(&mut self, condition: bool) -> Result<Option<u16>, &'static str> {
        match condition {
            true => self.u16().map(Some),
            false => Ok(None),
        }
    }

    /// Reads an optional length-prefixed byte field
    pub fn optional_bytes<T>(&mut self, condition: bool) -> Result<Option<T>, &'static str>
    where
        T: AnyVec<u8>,
    {
        match condition {
            true => self.bytes().map(Some),
            false => Ok(None),
        }
    }
}
impl<Iter> Decoder<Peekable<Iter>>
where
    Iter: Iterator<Item = u8>,
{
    /// Peeks at the next byte from the underlying source without consuming it
    pub fn peek_u8(&mut self) -> Option<u8> {
        self.source.peek().copied()
    }

    /// Checks if the underlying source is empty
    #[must_use]
    pub fn is_empty(&mut self) -> bool {
        self.source.peek().is_none()
    }

    /// Reads a sequence of topics
    ///
    /// # Note
    /// This function is greedy. As there is no way to know how much topics to read, this function will simply read as
    /// much bytes as possible until the underlying source is exhausted. Limit the source using [`Self::limit`] if
    /// necessary.
    pub fn topics<S, T>(&mut self) -> Result<S, &'static str>
    where
        S: AnyVec<T>,
        T: AnyVec<u8>,
    {
        // Read tuples
        let mut topics = S::default();
        while !self.is_empty() {
            // Read topic and associated QoS
            let topic = self.bytes()?;
            topics.push(topic)?;
        }
        Ok(topics)
    }

    /// Reads a sequence of topic+quality-of-service tuples
    ///
    /// # Note
    /// This function is greedy. As there is no way to know how much tuples to read, this function will simply read as
    /// much bytes as possible until the underlying source is exhausted. Limit the source using [`Self::limit`] if
    /// necessary.
    pub fn topics_qos<S, T>(&mut self) -> Result<S, &'static str>
    where
        S: AnyVec<(T, u8)>,
        T: AnyVec<u8>,
    {
        // Read tuples
        let mut topics_qos = S::default();
        while !self.is_empty() {
            // Read topic and associated QoS
            let topic = self.bytes()?;
            let qos = self.u8()?;
            topics_qos.push((topic, qos))?;
        }
        Ok(topics_qos)
    }
}
impl<Iter> IntoIterator for Decoder<Iter>
where
    Iter: Iterator,
{
    type Item = Iter::Item;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        self.source
    }
}
