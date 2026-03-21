use core::mem;

use crate::{Result, decoder::Utf8Decoder};

/// fallible analog to [`CharIndices`], backed by a byte iterator
///
/// yields [`char`]s and their offset or an [`Error`] on invalid utf-8
///
/// [`CharIndices`]: core::str::CharIndices
/// [`Error`]: crate::Error
///
/// # Examples
///
/// valid bytes yield [`char`]s and their positions
///
/// ```
/// # use bytes2chars::{Result, Utf8CharIndices};
/// # fn main() -> Result<()> {
/// let input = b"\xF0\x9F\xA6\x80 hi";
/// let actual: Result<Vec<_>> = Utf8CharIndices::from(input.iter().copied()).collect();
/// let expected = vec![(0, '🦀'), (4, ' '), (5, 'h'), (6, 'i')];
/// assert_eq!(actual?, expected);
/// # Ok(())
/// # }
/// ```
///
/// iterator keeps going after an error. offending bytes are thrown in the garbage can
///
/// ```
/// # use bytes2chars::{Error, ErrorKind, Utf8CharIndices};
/// let results: Vec<_> = Utf8CharIndices::from(b"a\xC3\xC3b".iter().copied()).collect();
/// let expected = [
///     Ok((0, 'a')),
///     Err(Error {
///         range: 1..3,
///         kind: ErrorKind::ExpectedContinuation(0xC3),
///     }),
///     Ok((3, 'b')),
/// ];
/// assert_eq!(results, expected);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utf8CharIndices<I> {
    bytes: I,
    state: Utf8Decoder,
}

impl<I> Utf8CharIndices<I>
where
    I: Iterator<Item = u8>,
{
    /// create an iterator starting at byte offset `offset`
    pub fn new(bytes: I, offset: usize) -> Self {
        Self {
            bytes,
            state: Utf8Decoder::new(offset),
        }
    }

    /// convert into a [`Utf8Chars`] iterator yielding [`char`]s and omitting its position
    pub fn into_chars(self) -> Utf8Chars<I> {
        Utf8Chars { inner: self }
    }
}

impl<I> Iterator for Utf8CharIndices<I>
where
    I: Iterator<Item = u8>,
{
    type Item = Result<(usize, char)>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(b) = self.bytes.next() {
                if let x @ Some(_) = self.state.push(b) {
                    break x;
                }
            } else {
                break mem::take(&mut self.state).finish().err().map(Err);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.bytes.size_hint();
        (lower / 4, upper)
    }
}

impl<I> From<I> for Utf8CharIndices<I>
where
    I: Iterator<Item = u8>,
{
    fn from(value: I) -> Self {
        Self::new(value, 0)
    }
}

impl<I> From<Utf8CharIndices<I>> for Utf8Chars<I>
where
    I: Iterator<Item = u8>,
{
    fn from(value: Utf8CharIndices<I>) -> Self {
        value.into_chars()
    }
}

impl<I> core::iter::FusedIterator for Utf8CharIndices<I> where
    I: core::iter::FusedIterator<Item = u8>
{
}

/// fallible analog to [`Chars`], backed by a byte iterator
///
/// yields [`char`]s or an [`Error`] on invalid utf-8. a convenience wrapper over [`Utf8CharIndices`]
///
/// [`Chars`]: core::str::Chars
/// [`Error`]: crate::Error
///
/// # Examples
///
/// valid bytes yield [`char`]s
///
/// ```
/// # use bytes2chars::{Result, Utf8Chars};
/// # fn main() -> Result<()> {
/// let s: String =
///     Utf8Chars::from(b"\xF0\x9F\xA6\x80 rust".iter().copied()).collect::<Result<_>>()?;
/// assert_eq!(s, "🦀 rust");
/// # Ok(())
/// # }
/// ```
///
/// the iterator keeps going after an error. offending bytes are thrown in the garbage can
///
/// ```
/// # use bytes2chars::{Error, ErrorKind, Utf8Chars};
/// let results: Vec<_> = Utf8Chars::from(b"a\xC3\xC3b".iter().copied()).collect();
/// let expected = [
///     Ok('a'),
///     Err(Error {
///         range: 1..3,
///         kind: ErrorKind::ExpectedContinuation(0xC3),
///     }),
///     Ok('b'),
/// ];
/// assert_eq!(results, expected);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utf8Chars<I> {
    inner: Utf8CharIndices<I>,
}

impl<I> Utf8Chars<I>
where
    I: Iterator<Item = u8>,
{
    /// create an iterator starting at byte offset `offset`
    pub fn new(bytes: I, offset: usize) -> Self {
        Self {
            inner: Utf8CharIndices::new(bytes, offset),
        }
    }

    /// convert into a [`Utf8CharIndices`], which also yields byte offsets
    pub fn into_char_indices(self) -> Utf8CharIndices<I> {
        self.inner
    }
}

impl<I> Iterator for Utf8Chars<I>
where
    I: Iterator<Item = u8>,
{
    type Item = Result<char>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|r| r.map(|(_, c)| c))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I> From<I> for Utf8Chars<I>
where
    I: Iterator<Item = u8>,
{
    fn from(value: I) -> Self {
        Self::new(value, 0)
    }
}

impl<I> From<Utf8Chars<I>> for Utf8CharIndices<I>
where
    I: Iterator<Item = u8>,
{
    fn from(value: Utf8Chars<I>) -> Self {
        value.into_char_indices()
    }
}

impl<I> core::iter::FusedIterator for Utf8Chars<I> where I: core::iter::FusedIterator<Item = u8> {}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;

    const CRAB: &[u8] = b"\xF0\x9F\xA6\x80";

    #[test]
    fn unterminated_sequence_yields_eof_error() {
        // 0xC3 starts a 2-byte sequence but the stream ends after one byte
        let result: crate::Result<alloc::string::String> =
            Utf8Chars::new([0xC3u8].iter().copied(), 0).collect();
        assert_eq!(
            result.unwrap_err(),
            crate::Error {
                range: 0..1,
                kind: crate::ErrorKind::UnfinishedSequence,
            }
        );
    }

    #[test]
    fn indices_into_chars() {
        let indices = Utf8CharIndices::new(CRAB.iter().copied(), 0);
        let chars: Utf8Chars<_> = indices.into();
        let s: crate::Result<alloc::string::String> = chars.collect();
        assert_eq!(s.unwrap(), "🦀");
    }

    #[test]
    fn chars_into_indices() {
        let chars = Utf8Chars::new(CRAB.iter().copied(), 0);
        let indices: Utf8CharIndices<_> = chars.into();
        let v: crate::Result<alloc::vec::Vec<(usize, char)>> = indices.collect();
        assert_eq!(v.unwrap(), [(0, '🦀')]);
    }

    #[test]
    fn chars_dot_into_char_indices() {
        let chars = Utf8Chars::new(CRAB.iter().copied(), 0);
        let v: crate::Result<alloc::vec::Vec<(usize, char)>> = chars.into_char_indices().collect();
        assert_eq!(v.unwrap(), [(0, '🦀')]);
    }

    #[test]
    fn indices_dot_into_chars() {
        let chars: Utf8Chars<_> = Utf8CharIndices::new(CRAB.iter().copied(), 0).into_chars();
        let s: crate::Result<alloc::string::String> = chars.collect();
        assert_eq!(s.unwrap(), "🦀");
    }
}
