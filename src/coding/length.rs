//! A helper to predict the encoded length

use crate::anyvec::AnyVec;

/// A helper to predict the encoded length
#[derive(Debug, Clone, Copy, Default)]
pub struct Length {
    /// The accumulated length
    len: usize,
}
impl Length {
    /// Creates a new, all-zero length
    pub const fn new() -> Self {
        Self { len: 0 }
    }

    /// Writes a `u8`
    ///
    /// # Panics
    /// This function panics if the total accumulated length is greater than `usize::MAX`.
    pub fn u8(mut self, _u8: &u8) -> Self {
        #[allow(clippy::expect_used, reason = "Serious API misuse")]
        (self.len = self.len.checked_add(1).expect("Accumulated length is too large"));
        self
    }

    /// Writes a `u16`
    ///
    /// # Panics
    /// This function panics if the total accumulated length is greater than `usize::MAX`.
    pub fn u16(mut self, _u16: &u16) -> Self {
        #[allow(clippy::expect_used, reason = "Serious API misuse")]
        (self.len = self.len.checked_add(2).expect("Accumulated length is too large"));
        self
    }

    /// Writes some raw data as-is
    ///
    /// # Panics
    /// This function panics if the total accumulated length is greater than `usize::MAX`.
    pub fn raw<T>(mut self, raw: &T) -> Self
    where
        T: AsRef<[u8]> + IntoIterator<Item = u8>,
    {
        #[allow(clippy::expect_used, reason = "Serious API misuse")]
        (self.len = self.len.checked_add(raw.as_ref().len()).expect("Accumulated length is too large"));
        self
    }

    /// Writes a length-prefixed byte field
    ///
    /// # Panics
    /// This function panics if the length of the byte field is greater than `u16::MAX`. This function also panics if
    /// the total accumulated length is greater than `usize::MAX`.
    pub fn bytes<T>(mut self, bytes: &T) -> Self
    where
        T: AsRef<[u8]> + IntoIterator<Item = u8>,
    {
        #[allow(clippy::expect_used, reason = "Serious API misuse")]
        (self.len = (self.len.checked_add(2))
            .and_then(|len| len.checked_add(bytes.as_ref().len()))
            .expect("Accumulated length is too large"));
        self
    }

    /// Writes a bitmap as byte
    ///
    /// # Panics
    /// This function panics if the total accumulated length is greater than `usize::MAX`.
    pub fn bitmap(mut self, _bits: &[bool; 8]) -> Self {
        #[allow(clippy::expect_used, reason = "Serious API misuse")]
        (self.len = self.len.checked_add(1).expect("Accumulated length is too large"));
        self
    }

    /// Writes a packet type and associated flags (as bitmap) as header byte
    ///
    /// # Panics
    /// This function panics if the packet type is greater than `15` (`2^4 - 1`). This function also panics if the total
    /// accumulated length is greater than `usize::MAX`.
    pub fn header(mut self, type_: &u8, _flags: &[bool; 4]) -> Self {
        // Validate type value
        #[allow(clippy::panic, reason = "Serious API misuse")]
        (assert!(*type_ <= 15, "Packet type is too large"));

        // Accumulate length
        #[allow(clippy::expect_used, reason = "Serious API misuse")]
        (self.len = self.len.checked_add(1).expect("Accumulated length is too large"));
        self
    }

    /// Writes a packet length field
    ///
    /// # Panics
    /// This function panics if the packet length is greater than `2^28 - 1`. This function also panics if the total
    /// accumulated length is greater than `usize::MAX`.
    pub fn packetlen(mut self, len: &usize) -> Self {
        // Validate and compute packet length size
        #[allow(clippy::panic, reason = "Packet length must be encoded in 4 or less heptets")]
        #[allow(clippy::unusual_byte_groupings, reason = "Length bytes are encoded in heptets")]
        let len_size = match len {
            0b1_0000000_0000000_0000000_0000000.. => panic!("Packet length is too large"),
            0b1_0000000_0000000_0000000.. => 4,
            0b1_0000000_0000000.. => 3,
            0b1_0000000.. => 2,
            _ => 1,
        };

        // Accumulate length
        #[allow(clippy::expect_used, reason = "Serious API misuse")]
        (self.len = self.len.checked_add(len_size).expect("Accumulated length is too large"));
        self
    }

    /// Writes an optional `u16`
    ///
    /// # Panics
    /// This function panics if the total accumulated length is greater than `usize::MAX`.
    pub fn optional_u16(self, u16_: &Option<u16>) -> Self {
        match u16_ {
            Some(u16_) => self.u16(u16_),
            None => self,
        }
    }

    /// Writes an optional length-prefixed byte field
    ///
    /// # Panics
    /// This function panics if the length of the byte field is greater than `u16::MAX`. This function also panics if
    /// the total accumulated length is greater than `usize::MAX`.
    pub fn optional_bytes<T>(self, bytes: &Option<T>) -> Self
    where
        T: AnyVec<u8>,
    {
        match bytes {
            Some(bytes) => self.bytes(bytes),
            None => self,
        }
    }

    /// Writes a sequence of topic+quality-of-service tuples
    ///
    /// # Panics
    /// This function panics if the length of a topic is greater than `u16::MAX`. This function also panics if the total
    /// accumulated length is greater than `usize::MAX`.
    pub fn topics<S, T>(mut self, topics: &S) -> Self
    where
        S: AsRef<[T]>,
        T: AsRef<[u8]> + IntoIterator<Item = u8>,
    {
        // Sum-up all topics
        for topic in topics.as_ref() {
            // Topics are just concatenated
            self = self.bytes(topic);
        }
        self
    }

    /// Writes a sequence of topic+quality-of-service tuples
    ///
    /// # Panics
    /// This function panics if the length of a topic is greater than `u16::MAX`. This function also panics if the total
    /// accumulated length is greater than `usize::MAX`.
    pub fn topics_qos<S, T>(mut self, topics_qos: &S) -> Self
    where
        S: AsRef<[(T, u8)]>,
        T: AsRef<[u8]> + IntoIterator<Item = u8>,
    {
        // Sum-up all tuples
        for (topic, qos) in topics_qos.as_ref() {
            // Topic+QoS tubles are just concatenated
            self = self.bytes(topic);
            self = self.u8(qos);
        }
        self
    }
}
impl From<Length> for usize {
    fn from(value: Length) -> Self {
        value.len
    }
}
