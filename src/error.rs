//! Error types and aggregate types

use core::fmt::{Debug, Display, Formatter};

/// Creates a new error with the given variant and description
#[macro_export]
macro_rules! err {
    ($variant:expr, $desc:expr) => {{
        $crate::error::Error::new($variant, $desc, file!(), line!())
    }};
}

/// A slim, abstract error type
#[derive(Debug, Clone)]
pub struct Error<Variant> {
    /// The error variant
    pub variant: Variant,
    /// A human readable error description
    #[cfg(feature = "backtrace")]
    pub description: &'static str,
    /// The file where the error was created
    #[cfg(feature = "backtrace")]
    pub file: &'static str,
    /// The line at which the error was created
    #[cfg(feature = "backtrace")]
    pub line: u32,
    /// An informative error backtrace for debugging
    #[cfg(all(feature = "backtrace", feature = "std"))]
    pub backtrace: std::sync::Arc<std::backtrace::Backtrace>,
}
impl<Variant> Error<Variant> {
    /// Creates a new error variant
    #[doc(hidden)]
    #[allow(unused, reason = "some args are unused, depending on feature flags")]
    pub fn new(variant: Variant, description: &'static str, file: &'static str, line: u32) -> Self {
        Self {
            variant,
            #[cfg(feature = "backtrace")]
            description,
            #[cfg(feature = "backtrace")]
            file,
            #[cfg(feature = "backtrace")]
            line,
            #[cfg(all(feature = "backtrace", feature = "std"))]
            backtrace: std::sync::Arc::new(std::backtrace::Backtrace::capture()),
        }
    }

    /// Converts `self` into a new error variant while retaining all other metadata
    pub fn into_variant<NewVariant>(self, new_variant: NewVariant) -> Error<NewVariant> {
        Error {
            variant: new_variant,
            #[cfg(feature = "backtrace")]
            description: self.description,
            #[cfg(feature = "backtrace")]
            file: self.file,
            #[cfg(feature = "backtrace")]
            line: self.line,
            #[cfg(all(feature = "backtrace", feature = "std"))]
            backtrace: self.backtrace,
        }
    }
}
impl<T> Display for Error<T>
where
    T: Debug,
{
    #[cfg(not(feature = "backtrace"))]
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(f, "MQTT error: {:#?}", self.variant)
    }

    #[cfg(all(feature = "backtrace", not(feature = "std")))]
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(f, "MQTT error: {:#?} at {}:{}", self.variant, self.file, self.line)
    }

    #[cfg(all(feature = "backtrace", feature = "std"))]
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        use std::backtrace::BacktraceStatus;

        write!(f, "MQTT error: {:#?} at", self.variant)?;
        if self.backtrace.status() == BacktraceStatus::Captured {
            // Write backtrace if present
            writeln!(f, "{}", self.backtrace)?;
        }

        Ok(())
    }
}
#[cfg(feature = "std")]
impl<T> std::error::Error for Error<T>
where
    T: Debug,
{
    // No members to implement yet
}

/// Some buffer is out of memory
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Memory;

/// Some MQTT data is invalid
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Data {
    /// The data is truncated
    Truncated,
    /// The data violates the specifications
    SpecViolation,
}

/// A general decoding-related error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decoding {
    /// [`Data::Truncated`]
    Truncated,
    /// [`Data::SpecViolation`]
    SpecViolation,
    /// [`Memory`]
    Memory,
}
impl From<Error<Memory>> for Error<Decoding> {
    fn from(error: Error<Memory>) -> Self {
        error.into_variant(Decoding::Memory)
    }
}
impl From<Error<Data>> for Error<Decoding> {
    fn from(error: Error<Data>) -> Self {
        match error.variant {
            Data::Truncated => error.into_variant(Decoding::Truncated),
            Data::SpecViolation => error.into_variant(Decoding::SpecViolation),
        }
    }
}

/// [`Error<Memory>`]
pub type MemoryError = Error<Memory>;

/// [`Error<Data>`]
pub type DataError = Error<Data>;

/// [`Error<Decoding>`]
pub type DecoderError = Error<Decoding>;
