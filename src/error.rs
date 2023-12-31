//! An MQTT error

use core::{
    fmt::{Display, Formatter, Result},
    mem,
};
use std::backtrace::{Backtrace, BacktraceStatus};

/// An error
#[derive(Debug)]
pub enum ErrorKind {
    /// If the MQTT version is unsupported
    ///
    /// # Note
    /// This error is distinct from `UnsupportedValue` because a protocol mismatch must be handled gracefully by a server
    UnsupportedVersion,
    /// Some data is truncated
    TruncatedData,
    /// A value is invalid
    InvalidValue,
    /// An in-out error occurred
    InOutError(std::io::Error),
}
impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::UnsupportedVersion => write!(f, "unsupported protocol"),
            Self::TruncatedData => write!(f, "truncated input"),
            Self::InvalidValue => write!(f, "invalid value"),
            Self::InOutError(e) => write!(f, "in-out error: {e}"),
        }
    }
}
impl PartialEq for ErrorKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InOutError(_), Self::InOutError(_)) => false,
            _ => mem::discriminant(self) == mem::discriminant(other),
        }
    }
}
impl From<std::io::Error> for ErrorKind {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            // Unexpected EOFs always indicate that we expected more data
            std::io::ErrorKind::UnexpectedEof => Self::TruncatedData,
            _ => Self::InOutError(error),
        }
    }
}

/// An MQTT error
#[derive(Debug)]
pub struct MqttError {
    /// The error kind
    kind: ErrorKind,
    /// The backtrace if captured
    backtrace: Backtrace,
}
impl MqttError {
    /// Creates a new error
    pub fn new(kind: ErrorKind) -> Self {
        let backtrace = Backtrace::capture();
        Self { kind, backtrace }
    }

    /// The error kind
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
    /// Whether the error has a backtrace
    pub fn has_backtrace(&self) -> bool {
        self.backtrace.status() == BacktraceStatus::Captured
    }
    /// The backtrace (maybe empty; to see if a backtrace has been captured, use `self.has_backtrace()`)
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}
impl Display for MqttError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        self.kind.fmt(f)
    }
}
impl std::error::Error for MqttError {
    // Nothing to override
}
impl From<ErrorKind> for MqttError {
    fn from(kind: ErrorKind) -> Self {
        Self::new(kind)
    }
}
impl From<std::io::Error> for MqttError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.into())
    }
}
