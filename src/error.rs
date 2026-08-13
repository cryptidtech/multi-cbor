//! When serializing or deserializing CBOR goes wrong.
use core::fmt;
use core::result;
use serde::de;
use serde::ser;
#[cfg(feature = "std")]
use std::error;
#[cfg(feature = "std")]
use std::io;

/// This type represents all possible errors that can occur when serializing or deserializing CBOR
/// data.
///
/// # Backtrace Support
///
/// When the `std` feature is enabled and running on Rust 1.65+, errors automatically capture
/// a backtrace when created. Access it via the `std::error::Error::backtrace()` method.
pub struct Error(ErrorImpl);

/// Alias for a `Result` with the error type `multi_cbor::Error`.
pub type Result<T> = result::Result<T, Error>;

/// Categorizes the cause of a `multi_cbor::Error`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Category {
    /// The error was caused by a failure to read or write bytes on an IO stream.
    Io,
    /// The error was caused by input that was not syntactically valid CBOR.
    Syntax,
    /// The error was caused by input data that was semantically incorrect.
    Data,
    /// The error was caused by prematurely reaching the end of the input data.
    Eof,
}

impl Error {
    /// The byte offset at which the error occurred.
    #[must_use]
    pub const fn offset(&self) -> u64 {
        self.0.offset
    }

    pub(crate) fn syntax(code: ErrorCode, offset: u64) -> Self {
        Self(ErrorImpl {
            code,
            offset,
            #[cfg(feature = "std")]
            _backtrace: std::backtrace::Backtrace::capture(),
        })
    }

    #[cfg(feature = "std")]
    pub(crate) fn io(error: io::Error) -> Self {
        Self(ErrorImpl {
            code: ErrorCode::Io(error),
            offset: 0,
            _backtrace: std::backtrace::Backtrace::capture(),
        })
    }

    #[cfg(all(not(feature = "std"), feature = "unsealed_read_write"))]
    /// Creates an error signalling that the underlying `Read` encountered an I/O error.
    #[must_use]
    pub fn io() -> Self {
        Self(ErrorImpl {
            code: ErrorCode::Io,
            offset: 0,
        })
    }

    #[cfg(feature = "unsealed_read_write")]
    /// Creates an error signalling that the scratch buffer was too small to fit the data.
    #[must_use]
    pub fn scratch_too_small(offset: u64) -> Self {
        Self(ErrorImpl {
            code: ErrorCode::ScratchTooSmall,
            offset,
            #[cfg(feature = "std")]
            _backtrace: std::backtrace::Backtrace::capture(),
        })
    }

    #[cfg(not(feature = "unsealed_read_write"))]
    pub(crate) fn scratch_too_small(offset: u64) -> Self {
        Self(ErrorImpl {
            code: ErrorCode::ScratchTooSmall,
            offset,
            #[cfg(feature = "std")]
            _backtrace: std::backtrace::Backtrace::capture(),
        })
    }

    #[cfg(feature = "unsealed_read_write")]
    /// Creates an error with a custom message.
    ///
    /// **Note**: When the "std" feature is disabled, the message will be discarded.
    pub fn message<T: fmt::Display>(msg: T) -> Self {
        #[cfg(not(feature = "std"))]
        {
            let _ = msg;
            Self(ErrorImpl {
                code: ErrorCode::Message,
                offset: 0,
            })
        }
        #[cfg(feature = "std")]
        {
            Self(ErrorImpl {
                code: ErrorCode::Message(msg.to_string()),
                offset: 0,
                _backtrace: std::backtrace::Backtrace::capture(),
            })
        }
    }

    #[cfg(not(feature = "unsealed_read_write"))]
    pub(crate) fn message<T: fmt::Display>(msg: T) -> Self {
        #[cfg(not(feature = "std"))]
        {
            let _ = msg;
            Self(ErrorImpl {
                code: ErrorCode::Message,
                offset: 0,
            })
        }
        #[cfg(feature = "std")]
        {
            Self(ErrorImpl {
                code: ErrorCode::Message(msg.to_string()),
                offset: 0,
                _backtrace: std::backtrace::Backtrace::capture(),
            })
        }
    }

    #[cfg(feature = "unsealed_read_write")]
    /// Creates an error signalling that the underlying read
    /// encountered an end of input.
    #[must_use]
    pub fn eof(offset: u64) -> Self {
        Self(ErrorImpl {
            code: ErrorCode::EofWhileParsingValue,
            offset,
            #[cfg(feature = "std")]
            _backtrace: std::backtrace::Backtrace::capture(),
        })
    }

    /// Categorizes the cause of this error.
    #[must_use]
    pub const fn classify(&self) -> Category {
        match self.0.code {
            #[cfg(feature = "std")]
            ErrorCode::Message(_) => Category::Data,
            #[cfg(not(feature = "std"))]
            ErrorCode::Message => Category::Data,
            #[cfg(feature = "std")]
            ErrorCode::Io(_) => Category::Io,
            #[cfg(not(feature = "std"))]
            ErrorCode::Io => Category::Io,
            ErrorCode::ScratchTooSmall => Category::Io,
            ErrorCode::EofWhileParsingValue
            | ErrorCode::EofWhileParsingArray
            | ErrorCode::EofWhileParsingMap => Category::Eof,
            ErrorCode::LengthOutOfRange
            | ErrorCode::InvalidUtf8
            | ErrorCode::UnassignedCode
            | ErrorCode::UnexpectedCode
            | ErrorCode::TrailingData
            | ErrorCode::ArrayTooShort
            | ErrorCode::ArrayTooLong
            | ErrorCode::RecursionLimitExceeded
            | ErrorCode::WrongEnumFormat
            | ErrorCode::WrongStructFormat
            | ErrorCode::ArraySizeLimitExceeded
            | ErrorCode::MapSizeLimitExceeded
            | ErrorCode::IndefiniteIterationLimitExceeded => Category::Syntax,
        }
    }

    /// Returns true if this error was caused by a failure to read or write bytes on an IO stream.
    #[must_use]
    pub const fn is_io(&self) -> bool {
        matches!(self.classify(), Category::Io)
    }

    /// Returns true if this error was caused by input that was not syntactically valid CBOR.
    #[must_use]
    pub const fn is_syntax(&self) -> bool {
        matches!(self.classify(), Category::Syntax)
    }

    /// Returns true if this error was caused by data that was semantically incorrect.
    #[must_use]
    pub const fn is_data(&self) -> bool {
        matches!(self.classify(), Category::Data)
    }

    /// Returns true if this error was caused by prematurely reaching the end of the input data.
    #[must_use]
    pub const fn is_eof(&self) -> bool {
        matches!(self.classify(), Category::Eof)
    }

    /// Returns true if this error was caused by the scratch buffer being too small.
    ///
    /// Note this being `true` implies that `is_io()` is also `true`.
    #[must_use]
    pub const fn is_scratch_too_small(&self) -> bool {
        matches!(self.0.code, ErrorCode::ScratchTooSmall)
    }
}

#[cfg(feature = "std")]
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.0.code {
            ErrorCode::Io(ref err) => Some(err),
            _ => None,
        }
    }

    // Note: Once std::error::Error::provide() is stabilized, we can expose the backtrace
    // For now, the backtrace is captured and stored but not exposed via the Error trait
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.offset == 0 {
            fmt::Display::fmt(&self.0.code, f)
        } else {
            write!(f, "{} at offset {}", self.0.code, self.0.offset)
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, fmt)
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::message(msg)
    }

    fn invalid_type(unexp: de::Unexpected<'_>, exp: &dyn de::Expected) -> Self {
        if unexp == de::Unexpected::Unit {
            Self::custom(format_args!("invalid type: null, expected {exp}"))
        } else {
            Self::custom(format_args!("invalid type: {unexp}, expected {exp}"))
        }
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::message(msg)
    }
}

#[cfg(feature = "std")]
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::io(e)
    }
}

#[cfg(not(feature = "std"))]
impl From<core::fmt::Error> for Error {
    fn from(_: core::fmt::Error) -> Self {
        Self(ErrorImpl {
            code: ErrorCode::Message,
            offset: 0,
        })
    }
}

#[derive(Debug)]
struct ErrorImpl {
    code: ErrorCode,
    offset: u64,
    #[cfg(feature = "std")]
    _backtrace: std::backtrace::Backtrace,
}

#[derive(Debug)]
pub(crate) enum ErrorCode {
    #[cfg(feature = "std")]
    Message(String),
    #[cfg(not(feature = "std"))]
    Message,
    #[cfg(feature = "std")]
    Io(io::Error),
    #[allow(unused)]
    #[cfg(not(feature = "std"))]
    Io,
    ScratchTooSmall,
    EofWhileParsingValue,
    EofWhileParsingArray,
    EofWhileParsingMap,
    LengthOutOfRange,
    InvalidUtf8,
    UnassignedCode,
    UnexpectedCode,
    TrailingData,
    ArrayTooShort,
    ArrayTooLong,
    RecursionLimitExceeded,
    WrongEnumFormat,
    WrongStructFormat,
    ArraySizeLimitExceeded,
    MapSizeLimitExceeded,
    IndefiniteIterationLimitExceeded,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            #[cfg(feature = "std")]
            Self::Message(ref msg) => f.write_str(msg),
            #[cfg(not(feature = "std"))]
            Self::Message => f.write_str("Unknown error"),
            #[cfg(feature = "std")]
            Self::Io(ref err) => fmt::Display::fmt(err, f),
            #[cfg(not(feature = "std"))]
            Self::Io => f.write_str("Unknown I/O error"),
            Self::ScratchTooSmall => f.write_str("Scratch buffer too small"),
            Self::EofWhileParsingValue => f.write_str("EOF while parsing a value"),
            Self::EofWhileParsingArray => f.write_str("EOF while parsing an array"),
            Self::EofWhileParsingMap => f.write_str("EOF while parsing a map"),
            Self::LengthOutOfRange => f.write_str("length out of range"),
            Self::InvalidUtf8 => f.write_str("invalid UTF-8"),
            Self::UnassignedCode => f.write_str("unassigned type"),
            Self::UnexpectedCode => f.write_str("unexpected code"),
            Self::TrailingData => f.write_str("trailing data"),
            Self::ArrayTooShort => f.write_str("array too short"),
            Self::ArrayTooLong => f.write_str("array too long"),
            Self::RecursionLimitExceeded => f.write_str("recursion limit exceeded"),
            Self::WrongEnumFormat => f.write_str("wrong enum format"),
            Self::WrongStructFormat => f.write_str("wrong struct format"),
            Self::ArraySizeLimitExceeded => f.write_str("array size limit exceeded"),
            Self::MapSizeLimitExceeded => f.write_str("map size limit exceeded"),
            Self::IndefiniteIterationLimitExceeded => {
                f.write_str("indefinite-length iteration limit exceeded")
            }
        }
    }
}
