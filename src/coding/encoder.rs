//! An iterator-based encoder

use crate::anyvec::AnyVec;
use core::iter::{self, Chain, Empty, FlatMap, Once, Take};

/// An empty iterator
pub type Unit = Empty<u8>;
/// A result iterator when encoding a `u8`
pub type U8Iter = Once<u8>;
/// A result iterator when encoding a `u16`
pub type U16Iter = <[u8; 2] as IntoIterator>::IntoIter;
/// A result iterator when encoding a length-prefixed byte field
pub type BytesIter<Bytes> = Chain<U16Iter, <Bytes as IntoIterator>::IntoIter>;
/// A result iterator when encoding a packet length
pub type PacketLenIter = Take<<[u8; 4] as IntoIterator>::IntoIter>;
/// A result iterator when encoding an optional `u16`
pub type OptionalU16Iter = Take<U16Iter>;
/// A result iterator when encoding an optional length-prefixed byte field
pub type OptionalBytesIter<Bytes> = Chain<OptionalU16Iter, <Bytes as IntoIterator>::IntoIter>;
/// A result iterator when encoding a sequence of topic+quality-of-service tuples
pub type TopicsIter<Sequence, Bytes> =
    FlatMap<<Sequence as IntoIterator>::IntoIter, BytesIter<Bytes>, fn(Bytes) -> BytesIter<Bytes>>;
/// A result iterator when encoding a sequence of topic+quality-of-service tuples
pub type TopicsQosIter<Sequence, Bytes> = FlatMap<
    <Sequence as IntoIterator>::IntoIter,
    Chain<BytesIter<Bytes>, U8Iter>,
    fn((Bytes, u8)) -> Chain<BytesIter<Bytes>, U8Iter>,
>;

/// An iterator-based encoder
#[derive(Debug, Default)]
pub struct Encoder<Iter = Unit> {
    /// The underlying iterator
    sink: Iter,
}
impl<Iter> Encoder<Iter>
where
    Iter: Iterator<Item = u8>,
{
    /// Writes some raw data as-is
    pub fn raw<T>(self, raw: T) -> Encoder<Chain<Iter, <T as IntoIterator>::IntoIter>>
    where
        T: IntoIterator<Item = u8>,
    {
        Encoder { sink: self.sink.chain(raw) }
    }

    /// Writes a `u8`
    pub fn u8(self, u8_: u8) -> Encoder<Chain<Iter, U8Iter>> {
        let iter = iter::once(u8_);
        Encoder { sink: self.sink.chain(iter) }
    }

    /// Writes a `u16`
    pub fn u16(self, u16_: u16) -> Encoder<Chain<Iter, U16Iter>> {
        let iter = u16_.to_be_bytes().into_iter();
        Encoder { sink: self.sink.chain(iter) }
    }

    /// Writes a length-prefixed byte field
    ///
    /// # Panics
    /// This function panics if the length of the byte field is greater than `u16::MAX`.
    pub fn bytes<T>(self, bytes: T) -> Encoder<Chain<Iter, BytesIter<T>>>
    where
        T: AsRef<[u8]> + IntoIterator<Item = u8>,
    {
        // Encode length
        #[allow(clippy::expect_used, reason = "serious API misuse")]
        let len_iter = u16::try_from(bytes.as_ref().len()).expect("byte field is too long")
            // Create iterator
            .to_be_bytes().into_iter();

        // Chain length and bytes and yield new encoder
        let iter = len_iter.chain(bytes);
        Encoder { sink: self.sink.chain(iter) }
    }

    /// Writes a bitmap as byte
    pub fn bitmap(self, bits: [bool; 8]) -> Encoder<Chain<Iter, U8Iter>> {
        let byte = ((bits[0] as u8) << 7)
            | ((bits[1] as u8) << 6)
            | ((bits[2] as u8) << 5)
            | ((bits[3] as u8) << 4)
            | ((bits[4] as u8) << 3)
            | ((bits[5] as u8) << 2)
            | ((bits[6] as u8) << 1)
            | (bits[7] as u8);
        let iter = iter::once(byte);
        Encoder { sink: self.sink.chain(iter) }
    }

    /// Writes a packet type and associated flags (as bitmap) as header byte
    ///
    /// # Panics
    /// This function panics if the packet type is greater than `15` (`2^4 - 1`).
    pub fn header(self, type_: u8, flags: [bool; 4]) -> Encoder<Chain<Iter, U8Iter>> {
        // Validate type value
        #[allow(clippy::panic, reason = "serious API misuse")]
        (assert!(type_ <= 15, "packet type is too large"));

        // Assemble byte
        let byte = (type_ << 4)
            | ((flags[0] as u8) << 3)
            | ((flags[1] as u8) << 2)
            | ((flags[2] as u8) << 1)
            | (flags[3] as u8);
        let iter = iter::once(byte);
        Encoder { sink: self.sink.chain(iter) }
    }

    /// Writes a packet length field
    ///
    /// # Panics
    /// This function panics if the packet length is greater than `2^28 - 1`.
    pub fn packetlen(self, mut len: usize) -> Encoder<Chain<Iter, PacketLenIter>> {
        // Validate and compute packet length size
        #[allow(clippy::panic, reason = "packet length must be encoded in 4 or less heptets")]
        #[allow(clippy::unusual_byte_groupings, reason = "length bytes are encoded in heptets")]
        let len_size = match len {
            0b1_0000000_0000000_0000000_0000000.. => panic!("packet length is too large"),
            0b1_0000000_0000000_0000000.. => 4,
            0b1_0000000_0000000.. => 3,
            0b1_0000000.. => 2,
            _ => 1,
        };

        // Encode the length in 7-bit nibbles
        let mut bytes = [0; 4];
        for index in 0..len_size {
            // Push the next remaining least-significant 7 bits to the **front** of the encoded length
            bytes.rotate_right(1);
            bytes[0] = (len as u8) & 0b0111_1111;
            len >>= 7;

            // Insert the marker if the byte is not at the end-of-array
            if index > 0 {
                bytes[0] |= 0b1000_0000;
            }
        }

        // Truncate the length field accordingly
        let iter = bytes.into_iter().take(len_size);
        Encoder { sink: self.sink.chain(iter) }
    }

    /// Writes a `u16`
    pub fn optional_u16(self, u16_: Option<u16>) -> Encoder<Chain<Iter, OptionalU16Iter>> {
        // Map the `u16` iterator into a type representation that works for both cases
        let iter = match u16_ {
            Some(u16_) => u16_.to_be_bytes().into_iter().take(2),
            None => 0u16.to_ne_bytes().into_iter().take(0),
        };
        Encoder { sink: self.sink.chain(iter) }
    }

    /// Writes an optional length-prefixed byte field
    ///
    /// # Panics
    /// This function panics if the length of the byte field is greater than `u16::MAX`.
    pub fn optional_bytes<T>(self, bytes: Option<T>) -> Encoder<Chain<Iter, OptionalBytesIter<T>>>
    where
        T: AnyVec<u8>,
    {
        // Find an iterator representation that works for both cases
        if let Some(bytes) = bytes {
            // Encode length
            #[allow(clippy::expect_used, reason = "serious API misuse")]
            let len_iter = u16::try_from(bytes.as_ref().len()).expect("byte field is too long")
                // Create iterator
                .to_be_bytes().into_iter()
                // This allows us to mock the None-case
                .take(2);

            // Chain length and bytes and yield new encoder
            let iter = len_iter.chain(bytes);
            Encoder { sink: self.sink.chain(iter) }
        } else {
            // Create two empty iterators that yield the same type signature as the Some-case
            let len_iter = [0u8; 2].into_iter().take(0);
            let iter = len_iter.chain(T::default());
            Encoder { sink: self.sink.chain(iter) }
        }
    }

    /// Writes a sequence of topic+quality-of-service tuples
    ///
    /// # Panics
    /// This function panics if the length of a topic is greater than `u16::MAX`.
    pub fn topics<S, T>(self, topics: S) -> Encoder<Chain<Iter, TopicsIter<S, T>>>
    where
        S: IntoIterator<Item = T>,
        T: AsRef<[u8]> + IntoIterator<Item = u8>,
    {
        /// Static helper function for `flat_map` sp that the iterator doesn't capture state
        fn topics_flatmap<T>(topic: T) -> BytesIter<T>
        where
            T: AsRef<[u8]> + IntoIterator<Item = u8>,
        {
            // Encode topic length
            #[allow(clippy::expect_used, reason = "serious API misuse")]
            let len_iter = u16::try_from(topic.as_ref().len()).expect("topic is too long")
                // Create iterator
                .to_be_bytes().into_iter();

            // Chain length and bytes and yield new encoder
            len_iter.chain(topic)
        }

        // Create iterator
        let flat_map_fn: fn(T) -> BytesIter<T> = topics_flatmap::<T>;
        let topics = topics.into_iter().flat_map(flat_map_fn);
        Encoder { sink: self.sink.chain(topics) }
    }

    /// Writes a sequence of topic+quality-of-service tuples
    ///
    /// # Panics
    /// This function panics if the length of a topic is greater than `u16::MAX`.
    pub fn topics_qos<S, T>(self, topics_qos: S) -> Encoder<Chain<Iter, TopicsQosIter<S, T>>>
    where
        S: IntoIterator<Item = (T, u8)>,
        T: AsRef<[u8]> + IntoIterator<Item = u8>,
    {
        /// Static helper function for `flat_map` sp that the iterator doesn't capture state
        fn topics_qos_flatmap<T>((topic, qos): (T, u8)) -> Chain<BytesIter<T>, U8Iter>
        where
            T: AsRef<[u8]> + IntoIterator<Item = u8>,
        {
            // Encode topic length
            #[allow(clippy::expect_used, reason = "serious API misuse")]
            let len_iter = u16::try_from(topic.as_ref().len()).expect("topic is too long")
                // Create iterator
                .to_be_bytes().into_iter();

            // Chain length and bytes and yield new encoder
            len_iter.chain(topic).chain(iter::once(qos))
        }

        // Create iterator
        let flat_map_fn: fn((T, u8)) -> Chain<BytesIter<T>, U8Iter> = topics_qos_flatmap::<T>;
        let topics_qos = topics_qos.into_iter().flat_map(flat_map_fn);
        Encoder { sink: self.sink.chain(topics_qos) }
    }
}
impl<Iter> IntoIterator for Encoder<Iter>
where
    Iter: Iterator,
{
    type Item = Iter::Item;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        self.sink
    }
}
