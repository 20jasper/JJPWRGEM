#![no_std]
#![doc = include_str!("../README.md")]

mod decoder;
mod iter;

pub use decoder::Utf8Decoder;
pub use error::{Error, ErrorKind, Result};
pub use iter::{Utf8CharIndices, Utf8Chars};

mod error {
    use core::ops::Range;

    use displaydoc::Display;

    /// a specialized [`core::result::Result`] for utf8 decoding
    pub type Result<T> = core::result::Result<T, Error>;

    #[allow(missing_docs)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Display)]
    pub enum ErrorKind {
        /// byte 0x{_0:02X} cannot start a UTF-8 sequence
        InvalidLead(u8),
        /// expected a continuation byte, found 0x{_0:02X}
        ExpectedContinuation(u8),
        /// invalid surrogate code point U+{_0:04X}
        InvalidSurrogate(u32),
        /// overlong encoding of U+{_0:04X}
        Overlong(u32),
        /// code point U+{_0:04X} exceeds maximum U+10FFFF
        OutOfRange(u32),
        /// unfinished multi-byte sequence
        UnfinishedSequence,
    }

    /// invalid utf-8 at bytes {range:?}: {kind}
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Display)]
    pub struct Error {
        /// byte range of the invalid sequence
        pub range: Range<usize>,
        #[allow(missing_docs)]
        pub kind: ErrorKind,
    }

    impl core::error::Error for Error {}
}
