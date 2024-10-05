//! Iterator based en-/decoding
#![doc(hidden)]

pub mod decoder;
pub mod encoder;
pub mod length;

/// An blank encoder
pub type Encoder = encoder::Encoder;
/// A decoder
pub type Decoder<T> = decoder::Decoder<T>;
