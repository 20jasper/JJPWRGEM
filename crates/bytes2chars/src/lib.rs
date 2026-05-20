#![no_std]
#![allow(
    unsafe_code,
    reason = "utf-8 decoder uses char::from_u32_unchecked on validated codepoints"
)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::multiple_unsafe_ops_per_block)]
//! # bytes2chars
//!
//! lazily decodes utf-8 [`char`][char]s from bytes
//!
//! provides lazy, fallible analogs to [`str::Chars`][str-chars] ([`Utf8Chars`][utf8-chars]) and [`str::CharIndices`][str-char-indices] ([`Utf8CharIndices`][utf8-char-indices]), as well as a lower-level push-based [`Utf8Decoder`][utf8-decoder]
//!
//! [char]: https://doc.rust-lang.org/stable/std/primitive.char.html
//! [str-chars]: https://doc.rust-lang.org/stable/std/str/struct.Chars.html
//! [str-char-indices]: https://doc.rust-lang.org/stable/std/str/struct.CharIndices.html
//! [utf8-chars]: https://docs.rs/bytes2chars/latest/bytes2chars/struct.Utf8Chars.html
//! [utf8-char-indices]: https://docs.rs/bytes2chars/latest/bytes2chars/struct.Utf8CharIndices.html
//! [utf8-decoder]: https://docs.rs/bytes2chars/latest/bytes2chars/struct.Utf8Decoder.html
//! ## installation
//!
//! ```shell
//! cargo add bytes2chars
//! ```
//!
//! ## design goals
//!
//! - rich errors—what went wrong and where
//! - lazy
//! - `no-std`
//! - performance
//!
//! ## quick start
//!
//! prefer iterators like [`Utf8CharIndices`][utf8-char-indices] or [`Utf8Chars`][utf8-chars] if you have access to a byte iterator. [`Utf8Chars`][utf8-chars] still tracks bytes for error context, so it's purely a convenience wrapper
//!
//! if you receive bytes in chunks, use the push-based [`Utf8Decoder`][utf8-decoder]
//!
//! ## examples
//!
//! ### iterator api
//!
//! ```rust
//! # use bytes2chars::{Result, Utf8CharIndices, Utf8Chars};
//! # fn main() -> Result<()> {
//! let input = b"\xF0\x9F\xA6\x80 rust".iter().copied();
//!
//! // decode into an iterator of chars and their positions
//! let indexed = Utf8CharIndices::from(input.clone()).collect::<Result<Vec<_>>>()?;
//! let expected = vec![(0, '🦀'), (4, ' '), (5, 'r'), (6, 'u'), (7, 's'), (8, 't')];
//! assert_eq!(indexed, expected);
//!
//! // convenience wrapper to decode into an iterator of chars
//! let chars = Utf8Chars::from(input).collect::<Result<String>>()?;
//! assert_eq!(chars, "🦀 rust");
//! # Ok(())
//! # }
//! ```
//!
//!
//! ### push based decoder
//!
//! ```rust
//! # use bytes2chars::Utf8Decoder;
//! # use bytes2chars::ErrorKind;
//! # use bytes2chars::Error;
//! # fn main() -> bytes2chars::Result<()> {
//! let mut decoder = Utf8Decoder::new(0);
//! assert_eq!(decoder.push(0xF0), None); // accumulating
//! assert_eq!(decoder.push(0x9F), None);
//! assert_eq!(decoder.push(0xA6), None);
//! assert_eq!(decoder.push(0x80), Some(Ok((0, '🦀')))); // complete
//! assert_eq!(decoder.push(0xF0), None); // start new sequence
//! let err = Error {
//!     range: 4..5,
//!     kind: ErrorKind::UnfinishedSequence,
//! };
//! assert_eq!(decoder.finish(), Err(err)); // check for truncated sequence
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## rfc 3629 conformance
//!
//! decoding requirements are formally specified in [`spec/utf8.md`][spec],
//! derived from [RFC 3629](https://datatracker.ietf.org/doc/html/rfc3629). requirements are linked to implementation and tests using [Tracey][tracey]
//!
//! conformance is validated against the [flenniken utf-8 test suite][utf8tests]
//!
//! [spec]: ../../spec/utf8.md
//! [tracey]: https://tracey.bearcove.eu/
//! [utf8tests]: https://github.com/flenniken/utf8tests
//!
//!
//!
//! ## comparison with alternatives
//!
//! the unique benefit `bytes2chars` provides is rich error context
//!
//! see [BENCHMARKS.md](../BENCHMARKS.md) for throughput comparisons. `bytes2chars` still has a ways to go with perf!
//!
//! ### [`std::str::from_utf8`](https://doc.rust-lang.org/std/str/fn.from_utf8.html)
//!
//! eager and error context provides a range but not a particular cause
//!
//! ### [`utf8-decode`](https://docs.rs/utf8-decode/latest/utf8_decode/index.html)
//!
//! also lazy. error provides a range but not a particular cause. does not provide a push based decoder
//!
//! ### [`bstr::ByteSlice::chars`](https://docs.rs/utf8-decode/latest/utf8_decode/index.html)
//!
//! also lazy. swallows errors. does not provide a push based decoder. really fast

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

    #[expect(missing_docs, reason = "self explanatory")]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Display)]
    pub enum ErrorKind {
        /// byte 0x{_0:02X} cannot start a UTF-8 sequence
        InvalidLead(u8),
        /// byte 0x{_0:02X} would start a sequence of 5 or more bytes
        InvalidSequenceLength(u8),
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
        #[expect(missing_docs, reason = "self explanatory")]
        pub kind: ErrorKind,
    }

    impl core::error::Error for Error {}
}
