//! MQTT packet types

use crate::error::DecoderError;

pub mod connack;
pub mod connect;
pub mod packet;
pub mod publish;
pub mod subscribe;
pub mod unsubscribe;
include!("_ack.rs");
include!("_signal.rs");

/// Traits for elements that can be build from a byte iterator
pub trait TryFromIterator
where
    Self: Sized,
{
    /// Tries to build `Self` from the given byte iterator
    fn try_from_iter<T>(iter: T) -> Result<Self, DecoderError>
    where
        T: IntoIterator<Item = u8>;
}

/// Traits for elements that can be built from a byte reader
#[cfg(feature = "std")]
pub trait TryFromReader
where
    Self: Sized,
{
    /// Tries to build `Self` from the given byte iterator
    fn try_read<T>(reader: T) -> Result<Self, std::io::Error>
    where
        T: std::io::Read;
}
#[cfg(feature = "std")]
impl<T> TryFromReader for T
where
    T: TryFromIterator,
{
    #[allow(clippy::unbuffered_bytes, reason = "implementors may pass a buffered reader if appropriate")]
    fn try_read<R>(reader: R) -> Result<Self, std::io::Error>
    where
        R: std::io::Read,
    {
        use crate::error::Decoding;
        use std::io::{Error, ErrorKind};

        // Create a byte iterator from the reader
        let mut last_error = None;
        let iter = reader.bytes()
            // Retain an I/O error if any
            .map(|result| result.map_err(|e| last_error = Some(e)))
            // Yield bytes as long as there is not an error
            .map_while(|result| result.ok());

        // Try to build `Self` from iterator
        match (Self::try_from_iter(iter), last_error) {
            (Ok(value), _) => Ok(value),
            (Err(_), Some(e)) => Err(e),
            (Err(e), _) => match e.variant {
                // Map error to an appropriate I/O error kind
                Decoding::Truncated => Err(Error::new(ErrorKind::UnexpectedEof, e)),
                Decoding::SpecViolation => Err(Error::new(ErrorKind::InvalidData, e)),
                Decoding::Memory => Err(Error::new(ErrorKind::OutOfMemory, e)),
            },
        }
    }
}

/// Traits for elements that can be written to a byte writer
#[cfg(feature = "std")]
pub trait ToWriter {
    /// Writes `self` to the given byte writer
    fn write<T>(self, writer: T) -> Result<(), std::io::Error>
    where
        T: std::io::Write;
}
#[cfg(feature = "std")]
impl<T> ToWriter for T
where
    T: IntoIterator<Item = u8>,
{
    fn write<W>(self, writer: W) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        use std::io::{BufWriter, Write};

        // Write each byte in a buffered way for performance
        let mut writer = BufWriter::new(writer);
        for byte in self {
            // Write byte
            writer.write_all(&[byte])?;
        }

        // Flush buffer
        writer.flush()
    }
}
