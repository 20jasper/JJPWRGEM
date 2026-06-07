use core::mem;

use inner::Utf8State;

use crate::{Error, ErrorKind, Result};

mod inner {
    use core::ops::RangeInclusive;

    use crate::ErrorKind;

    type Utf8Result<T> = core::result::Result<T, ErrorKind>;

    // utf8[depends validate.expected-continuation]
    const CONTINUATION_MASK: u8 = 0b1100_0000;
    const CONTINUATION_PREFIX: u8 = 0b1000_0000;
    const CONTINUATION_PAYLOAD_MASK: u8 = 0b0011_1111;
    // utf8[depends encoding.two-byte]
    const TWO_BYTE_LEAD_PAYLOAD_MASK: u8 = 0b0001_1111;
    // utf8[depends encoding.three-byte]
    const THREE_BYTE_LEAD_PAYLOAD_MASK: u8 = 0b0000_1111;
    // utf8[depends encoding.four-byte]
    const FOUR_BYTE_LEAD_PAYLOAD_MASK: u8 = 0b0000_0111;

    // utf8[depends validate.no-surrogates]
    const SURROGATE_RANGE: RangeInclusive<u32> = 0xD800..=0xDFFF;

    // utf8[depends validate.max-codepoint]
    const MAX_CODEPOINT: u32 = 0x0010_FFFF;

    trait Utf8ByteExt: Copy {
        fn is_utf8_continuation(self) -> bool;
        fn utf8_seq_len(self) -> Utf8Result<u8>;
    }

    impl Utf8ByteExt for u8 {
        // utf8[depends validate.expected-continuation]
        fn is_utf8_continuation(self) -> bool {
            self & CONTINUATION_MASK == CONTINUATION_PREFIX
        }

        fn utf8_seq_len(self) -> Utf8Result<u8> {
            match self {
                0x00..=0x7F => unreachable!(
                    "ASCII bytes are handled in Utf8Decoder::push before utf8_seq_len is called"
                ),
                0x80..=0xBF => Err(ErrorKind::InvalidLead(self)), // utf8[impl validate.invalid-lead]
                0xC0..=0xC1 => Err(ErrorKind::InvalidLead(self)), // utf8[impl validate.no-overlong]
                0xC2..=0xDF => Ok(2),                             // utf8[impl encoding.two-byte]
                0xE0..=0xEF => Ok(3),                             // utf8[impl encoding.three-byte]
                0xF0..=0xF4 => Ok(4),                             // utf8[impl encoding.four-byte]
                0xF5..=0xF7 => Err(ErrorKind::InvalidLead(self)), // utf8[impl validate.max-codepoint]
                0xF8..=0xFF => Err(ErrorKind::InvalidSequenceLength(self)), // utf8[impl validate.max-sequence-length]
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub(super) struct UtfStateInner {
        codepoint: u32,
        bytes: u8,
        max_bytes: u8,
    }

    impl UtfStateInner {
        fn push(&mut self, b: u8) {
            debug_assert!(
                self.bytes < self.max_bytes,
                "push called past expected sequence length"
            );

            if self.bytes == 0 {
                self.codepoint = Self::lead_payload_bits(b, self.max_bytes);
            } else {
                self.codepoint = (self.codepoint << 6) | Self::continuation_payload_bits(b);
            }

            self.bytes += 1;
        }

        fn lead_payload_bits(b: u8, max_bytes: u8) -> u32 {
            let mask = match max_bytes {
                2 => TWO_BYTE_LEAD_PAYLOAD_MASK,
                3 => THREE_BYTE_LEAD_PAYLOAD_MASK,
                4 => FOUR_BYTE_LEAD_PAYLOAD_MASK,
                _ => {
                    unreachable!("max_bytes is always 2, 3, or 4 per utf8_seq_len, got {max_bytes}")
                }
            };
            u32::from(b & mask)
        }

        fn continuation_payload_bits(b: u8) -> u32 {
            u32::from(b & CONTINUATION_PAYLOAD_MASK)
        }

        fn is_done(&self) -> bool {
            self.bytes == self.max_bytes
        }

        fn new(max_bytes: u8) -> Self {
            Self {
                codepoint: 0,
                bytes: 0,
                max_bytes,
            }
        }

        fn from_byte(b: u8, remaining_bytes: u8) -> Self {
            let mut s = Self::new(remaining_bytes);
            s.push(b);
            s
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Default)]
    pub(super) enum Utf8State {
        #[default]
        Idle,
        Expecting(UtfStateInner),
        Done(char),
    }

    impl Utf8State {
        pub(super) fn process(self, b: u8) -> Utf8Result<Self> {
            match self {
                Utf8State::Idle => {
                    let seq_len = b.utf8_seq_len()?;
                    match seq_len {
                        2..=4 => Ok(Utf8State::Expecting(UtfStateInner::from_byte(b, seq_len))),
                        _ => unreachable!("`utf8_seq_len` only returns 2..=4 for non-ASCII bytes"),
                    }
                }
                Utf8State::Expecting(mut state) => {
                    // utf8[impl validate.expected-continuation]
                    if !b.is_utf8_continuation() {
                        return Err(ErrorKind::ExpectedContinuation(b));
                    }

                    state.push(b);

                    if state.is_done() {
                        validate_utf8_sequence(&state)?;
                        Ok(Utf8State::Done(
                            // SAFETY: byte sequence is validated by `validate_utf8_sequence`
                            unsafe { char::from_u32_unchecked(state.codepoint) },
                        ))
                    } else {
                        Ok(Utf8State::Expecting(state))
                    }
                }
                Utf8State::Done(_) => {
                    unreachable!(
                        "`Utf8Decoder::push` resets `Done` to `Idle` before calling `process`"
                    )
                }
            }
        }
    }

    fn validate_utf8_sequence(bytes: &UtfStateInner) -> Utf8Result<()> {
        let codepoint = bytes.codepoint;

        // utf8[impl validate.no-surrogates]
        if SURROGATE_RANGE.contains(&codepoint) {
            crate::hint::cold_path();
            return Err(ErrorKind::InvalidSurrogate(codepoint));
        }

        // utf8[impl validate.max-codepoint]
        if codepoint > MAX_CODEPOINT {
            crate::hint::cold_path();
            return Err(ErrorKind::OutOfRange(codepoint));
        }

        // utf8[impl validate.no-overlong]
        if bytes.bytes != min_bytes_for_code_point(codepoint) {
            crate::hint::cold_path();
            return Err(ErrorKind::Overlong(codepoint));
        }

        Ok(())
    }

    // utf8[depends validate.no-overlong]
    fn min_bytes_for_code_point(cp: u32) -> u8 {
        match cp {
            0x0000..=0x007F => 1,       // utf8[depends encoding.ascii]
            0x0080..=0x07FF => 2,       // utf8[depends encoding.two-byte]
            0x0800..=0xFFFF => 3,       // utf8[depends encoding.three-byte]
            0x10000..=0x0010_FFFF => 4, // utf8[depends encoding.four-byte]
            _ => unreachable!(
                "code points above U+10FFFF are rejected by validate.max-codepoint before reaching this function"
            ),
        }
    }
}

/// push based UTF-8 decoder that tracks byte positions
///
/// # Examples
///
/// decode a valid character
///
/// ```
/// # use bytes2chars::{Utf8Decoder, Result};
/// # fn main() -> Result<()> {
/// let mut decoder = Utf8Decoder::new(0);
/// assert_eq!(decoder.push(0xF0), None); // accumulating
/// assert_eq!(decoder.push(0x9F), None);
/// assert_eq!(decoder.push(0xA6), None);
/// assert_eq!(decoder.push(0x80), Some(Ok((0, '🦀')))); // complete
/// decoder.finish()?; // check for truncated sequence
///
/// # Ok(())
/// # }
/// ```
///
/// keeps going after an error. offending bytes are thrown in the garbage can
///
/// ```
/// # use bytes2chars::{Error, ErrorKind, Utf8Decoder};
/// let mut decoder = Utf8Decoder::new(0);
/// assert_eq!(decoder.push(b'a'), Some(Ok((0, 'a'))));
/// assert_eq!(decoder.push(0xC3), None);
/// assert_eq!(
///     decoder.push(0xC3),
///     Some(Err(Error {
///         range: 1..3,
///         kind: ErrorKind::ExpectedContinuation(0xC3),
///     }))
/// );
/// assert_eq!(decoder.push(b'b'), Some(Ok((3, 'b'))));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Utf8Decoder {
    state: Utf8State,
    /// offset of the current sequence
    sequence_offset: usize,
    /// byte offset of the next byte to be consumed
    offset: usize,
}

impl Utf8Decoder {
    /// create a decoder starting at byte offset `offset`
    pub fn new(offset: usize) -> Self {
        Self {
            state: Utf8State::Idle,
            sequence_offset: offset,
            offset,
        }
    }

    /// process a single byte
    ///
    /// # Examples
    ///
    /// ```
    /// # use bytes2chars::{Error, ErrorKind, Utf8Decoder, Result};
    /// let mut decoder = Utf8Decoder::default();
    ///
    /// assert_eq!(decoder.push(0xC3), None); // accumulating
    /// assert_eq!(decoder.push(0xA9), Some(Ok((0, 'é')))); // complete
    ///
    /// // error
    /// let expected = Some(Err(Error {
    ///     range: 2..3,
    ///     kind: ErrorKind::InvalidLead(0x80),
    /// }));
    /// assert_eq!(decoder.push(0x80), expected);
    ///
    /// // after error, decoder is reset to idle and continues
    /// assert_eq!(decoder.push(b'b'), Some(Ok((3, 'b'))));
    /// assert_eq!(decoder.finish(), Ok(4)); // no truncated sequence
    /// ```
    pub fn push(&mut self, b: u8) -> Option<Result<(usize, char)>> {
        let curr_idx = self.offset;
        self.offset += 1;

        if matches!(self.state, Utf8State::Idle) {
            // utf8[impl encoding.ascii]
            if b.is_ascii() {
                return Some(Ok((curr_idx, char::from(b))));
            }
            self.sequence_offset = curr_idx;
        }

        match mem::take(&mut self.state).process(b) {
            Err(kind) => {
                return Some(Err(Error {
                    range: self.sequence_offset..curr_idx + 1,
                    kind,
                }));
            }
            Ok(state) => self.state = state,
        }

        if let Utf8State::Done(c) = self.state {
            self.state = Utf8State::Idle;
            Some(Ok((self.sequence_offset, c)))
        } else {
            None
        }
    }

    /// flush the decoder when there are no more bytes left
    ///
    /// on success, returns the total number of bytes consumed
    ///
    /// # Errors
    ///
    /// Returns an error of kind [`ErrorKind::UnfinishedSequence`] when current byte sequence is truncated
    ///
    /// # Examples
    ///
    /// ```
    /// # use bytes2chars::{ErrorKind, Utf8Decoder};
    /// // idle decoder is all good
    /// assert_eq!(Utf8Decoder::new(0).finish(), Ok(0));
    ///
    /// // incomplete sequence returns `UnfinishedSequence`
    /// let mut decoder = Utf8Decoder::new(0);
    /// assert_eq!(decoder.push(0xC3), None);
    /// assert_eq!(
    ///     decoder.finish().unwrap_err().kind,
    ///     ErrorKind::UnfinishedSequence
    /// );
    /// ```
    pub fn finish(self) -> Result<usize> {
        // utf8[impl validate.unfinished]
        if matches!(self.state, Utf8State::Expecting(_)) {
            Err(Error {
                range: self.sequence_offset..self.offset,
                kind: ErrorKind::UnfinishedSequence,
            })
        } else {
            Ok(self.offset)
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::vec::Vec;

    use super::*;

    fn collect_from_bytes(bytes: &[u8]) -> Vec<Result<(usize, char)>> {
        let mut decoder = Utf8Decoder::new(0);
        bytes
            .iter()
            .copied()
            .filter_map(|b| decoder.push(b))
            .collect()
    }

    fn collect_string(bytes: &[u8]) -> Result<alloc::string::String> {
        collect_from_bytes(bytes)
            .into_iter()
            .map(|r| r.map(|(_, c)| c))
            .collect()
    }

    // utf8[verify encoding.ascii]
    #[test]
    fn ascii() {
        assert_eq!(collect_from_bytes(b"a"), Vec::from([Ok((0, 'a'))]));
    }

    // utf8[verify encoding.two-byte]
    #[test]
    fn valid_2byte() {
        assert_eq!(collect_from_bytes(&[0xC3, 0xA9]), Vec::from([Ok((0, 'é'))]));
    }

    // utf8[verify encoding.three-byte]
    #[test]
    fn valid_3byte() {
        assert_eq!(
            collect_from_bytes(&[0xE2, 0x82, 0xAC]),
            Vec::from([Ok((0, '€'))])
        );
    }

    // utf8[verify encoding.four-byte]
    #[test]
    fn valid_4byte() {
        assert_eq!(
            collect_from_bytes(&[0xF0, 0x9F, 0x98, 0x80]),
            Vec::from([Ok((0, '😀'))])
        );
    }

    // utf8[verify validate.invalid-lead]
    #[test]
    fn invalid_lead_byte() {
        assert_eq!(
            collect_from_bytes(&[0x80]),
            Vec::from([Err(Error {
                range: 0..1,
                kind: ErrorKind::InvalidLead(0x80),
            })])
        );
    }

    // utf8[verify validate.max-sequence-length]
    #[test]
    fn five_plus_byte_lead() {
        assert_eq!(
            collect_from_bytes(&[0xF8]),
            Vec::from([Err(Error {
                range: 0..1,
                kind: ErrorKind::InvalidSequenceLength(0xF8),
            })])
        );
    }

    #[test]
    fn missing_continuation() {
        // offending bytes are thrown in the garbage can — 'a' lands at offset 2, not 1
        assert_eq!(
            collect_from_bytes(&[0xC3, 0xC3, b'a']),
            Vec::from([
                Err(Error {
                    range: 0..2,
                    kind: ErrorKind::ExpectedContinuation(0xC3),
                }),
                Ok((2, 'a')),
            ])
        );
    }

    // utf8[verify validate.expected-continuation]
    #[test]
    fn invalid_continuation() {
        assert_eq!(
            collect_from_bytes(&[0xC3, 0x40]),
            Vec::from([Err(Error {
                range: 0..2,
                kind: ErrorKind::ExpectedContinuation(0x40),
            })])
        );
    }

    // utf8[verify validate.no-overlong]
    #[test]
    fn e0_overlong() {
        assert_eq!(
            collect_from_bytes(&[0xE0, 0x80, 0x80]),
            Vec::from([Err(Error {
                range: 0..3,
                kind: ErrorKind::Overlong(0x0000),
            })])
        );
    }

    // utf8[verify validate.no-surrogates]
    #[test]
    fn ed_surrogate_rejected() {
        assert_eq!(
            collect_from_bytes(&[0xED, 0xA0, 0x80]),
            Vec::from([Err(Error {
                range: 0..3,
                kind: ErrorKind::InvalidSurrogate(0xD800),
            })])
        );
    }

    // utf8[verify validate.no-overlong]
    #[test]
    fn f0_overlong() {
        assert_eq!(
            collect_from_bytes(&[0xF0, 0x80, 0x80, 0x80]),
            Vec::from([Err(Error {
                range: 0..4,
                kind: ErrorKind::Overlong(0x0000),
            })])
        );
    }

    // utf8[verify validate.max-codepoint]
    #[test]
    fn f4_out_of_range() {
        assert_eq!(
            collect_from_bytes(&[0xF4, 0x90, 0x80, 0x80]),
            Vec::from([Err(Error {
                range: 0..4,
                kind: ErrorKind::OutOfRange(0x0011_0000),
            })])
        );
    }

    // utf8[verify validate.unfinished]
    #[test]
    fn incomplete_sequence_errors_on_finish() {
        let mut decoder = Utf8Decoder::new(0);
        assert!(decoder.push(0xC3).is_none()); // 2-byte sequence start
        assert_eq!(
            decoder.finish(),
            Err(Error {
                range: 0..1,
                kind: ErrorKind::UnfinishedSequence
            })
        );

        let mut decoder = Utf8Decoder::new(0);
        assert!(decoder.push(0xF0).is_none()); // 4-byte sequence start
        assert_eq!(
            decoder.finish(),
            Err(Error {
                range: 0..1,
                kind: ErrorKind::UnfinishedSequence
            })
        );
    }

    #[test]
    fn chars_collect() {
        assert_eq!(collect_string(b"\xF0\x9F\xA6\x80 rust").unwrap(), "🦀 rust");
    }

    #[test]
    fn done_starts_fresh_sequence() {
        let mut decoder = Utf8Decoder::new(0);
        assert_eq!(decoder.push(0xE2), None);
        assert_eq!(decoder.push(0x82), None);
        assert_eq!(decoder.push(0xAC), Some(Ok((0, '€'))));

        // now in Done state, next byte should start new sequence
        assert_eq!(decoder.push(b'a'), Some(Ok((3, 'a'))));
        assert_eq!(decoder.push(0xC3), None);
        // 0xC3 0x80 = U+00C0 (À), valid 2-byte sequence
        assert_eq!(decoder.push(0x80), Some(Ok((4, 'À'))));
    }

    #[test]
    fn new_at_offsets_indices() {
        // decoder starts mid-stream at byte 10
        let mut decoder = Utf8Decoder::new(10);
        // ASCII byte at effective position 10
        assert_eq!(decoder.push(b'x'), Some(Ok((10, 'x'))));
        // 2-byte sequence starting at effective position 11
        assert_eq!(decoder.push(0xC3), None);
        assert_eq!(decoder.push(0xA9), Some(Ok((11, 'é'))));
        // Error range is also offset
        assert_eq!(
            decoder.push(0x80),
            Some(Err(Error {
                range: 13..14,
                kind: ErrorKind::InvalidLead(0x80),
            }))
        );
    }
}
