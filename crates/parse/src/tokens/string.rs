use core::range::Range;
use std::iter::Peekable;

use crate::{
    Error, ErrorKind, Result,
    tokens::{
        CharWithContext, JsonByte, JsonChar, Token, TokenWithContext,
        lexical::UNICODE_ESCAPE_DIGITS,
    },
};

enum StringState {
    Open,
    CharOrEscapeOrEnd {
        string_range: Range<usize>,
        quote_range: Range<usize>,
    },
    Escape {
        string_range: Range<usize>,
        quote_range: Range<usize>,
        slash_range: Range<usize>,
    },
    UEscape {
        string_range: Range<usize>,
        quote_range: Range<usize>,
        u_range: Range<usize>,
        slash_range: Range<usize>,
        digits_seen: usize,
    },
    End,
}

impl StringState {
    fn process(
        self,
        chars: &mut Peekable<impl Iterator<Item = CharWithContext>>,
        input: &str,
    ) -> Result<Self> {
        let res = match self {
            StringState::Open => {
                let Some(CharWithContext(starting_quote, JsonChar('"'))) = chars.next() else {
                    unreachable!("must start with a quote");
                };

                StringState::CharOrEscapeOrEnd {
                    string_range: starting_quote,
                    quote_range: starting_quote,
                }
            }
            StringState::CharOrEscapeOrEnd {
                string_range,
                quote_range,
            } => match chars.next() {
                Some(CharWithContext(r, JsonChar('\\'))) => StringState::Escape {
                    string_range: string_range.start..r.end,
                    quote_range,
                    slash_range: r,
                },
                Some(CharWithContext(_, JsonChar('"'))) => StringState::End,
                Some(CharWithContext(r, c)) if c.is_control() => {
                    return Err(Error::new(
                        ErrorKind::UnexpectedControlCharacterInString(c),
                        r,
                        input,
                    ));
                }
                Some(CharWithContext(r, _)) => StringState::CharOrEscapeOrEnd {
                    string_range: string_range.start..r.end,
                    quote_range,
                },
                None => {
                    return Err(Error::from_unterminated(
                        ErrorKind::ExpectedQuote {
                            open_range: quote_range,
                            string_range,
                        },
                        input,
                    ));
                }
            },
            StringState::Escape {
                string_range,
                quote_range,
                slash_range,
            } => match chars.next() {
                Some(CharWithContext(r, c)) if c.can_be_escaped_directly() => {
                    StringState::CharOrEscapeOrEnd {
                        string_range: string_range.start..r.end,
                        quote_range,
                    }
                }
                Some(CharWithContext(r, JsonChar('u'))) => StringState::UEscape {
                    string_range,
                    quote_range,
                    u_range: r,
                    slash_range,
                    digits_seen: 0,
                },
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedEscape {
                            maybe_c: c,
                            slash_range,
                            string_range,
                            quote_range,
                        },
                        maybe_c,
                        input,
                    ));
                }
            },
            StringState::UEscape {
                string_range,
                quote_range,
                u_range,
                slash_range,
                digits_seen,
            } => match chars.next() {
                Some(CharWithContext(r, c)) if c.is_hexdigit() => {
                    let string_range = string_range.start..r.end;
                    let next_digits = digits_seen + 1;
                    if next_digits == UNICODE_ESCAPE_DIGITS {
                        StringState::CharOrEscapeOrEnd {
                            string_range,
                            quote_range,
                        }
                    } else {
                        StringState::UEscape {
                            string_range,
                            quote_range,
                            u_range,
                            slash_range,
                            digits_seen: next_digits,
                        }
                    }
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedHexDigit {
                            quote_range,
                            slash_range,
                            u_range,
                            maybe_c: c,
                            digit_idx: digits_seen + 1,
                        },
                        maybe_c,
                        input,
                    ));
                }
            },
            StringState::End => self,
        };

        Ok(res)
    }
}

type Chunk = std::simd::u8x32;

/// Index of the first `"`, `\` or control byte at or after `from`.
fn find_special(bytes: &[u8], from: usize) -> Option<usize> {
    use std::simd::cmp::{SimdPartialEq as _, SimdPartialOrd as _};

    let quote = Chunk::splat(b'"');
    let backslash = Chunk::splat(b'\\');
    let last_control = Chunk::splat(JsonChar::CONTROL_RANGE.last as u8);

    let mut i = from;
    while let Some(window) = bytes.get(i..i + Chunk::LEN) {
        let chunk = Chunk::from_slice(window);
        let special = chunk.simd_eq(quote) | chunk.simd_eq(backslash) | chunk.simd_le(last_control);
        let mask = special.to_bitmask();
        if mask != 0 {
            return Some(i + mask.trailing_zeros() as usize);
        }
        i += Chunk::LEN;
    }

    bytes[i..]
        .iter()
        .position(|&b| matches!(b, b'"' | b'\\') || JsonByte(b).is_control())
        .map(|offset| i + offset)
}

/// End of the string whose opening quote is at `pos` if it is valid, `None` otherwise.
///
/// Errors are left to [`invalid_string_error`] so their messages come from one place.
fn scan_valid_string(input: &str, pos: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut i = pos + 1;
    loop {
        i = find_special(bytes, i)?;
        match bytes[i] {
            b'"' => return Some(i + 1),
            b'\\' => {
                let escaped = JsonByte(*bytes.get(i + 1)?);
                // the backslash and the byte it escapes
                i += 2;
                if escaped.0 == b'u' {
                    let digits = bytes.get(i..i + UNICODE_ESCAPE_DIGITS)?;
                    if !digits.iter().all(u8::is_ascii_hexdigit) {
                        return None;
                    }
                    i += UNICODE_ESCAPE_DIGITS;
                } else if !escaped.can_be_escaped_directly() {
                    return None;
                }
            }
            _ => return None,
        }
    }
}

pub fn parse_string(input: &str, pos: usize) -> Result<TokenWithContext> {
    match scan_valid_string(input, pos) {
        Some(end) => Ok(TokenWithContext::new(Token::String, pos..end)),
        None => Err(invalid_string_error(input, pos)),
    }
}

/// Slower variant of [`scan_valid_string`] providing more precise error diagnostics
fn invalid_string_error(input: &str, pos: usize) -> Error {
    let mut chars = input[pos..]
        .char_indices()
        .map(|(i, c)| (i + pos, c).into())
        .peekable();
    let mut state = StringState::Open;

    loop {
        match state.process(&mut chars, input) {
            Ok(StringState::End) => unreachable!("only called on strings that are invalid"),
            Ok(next) => state = next,
            Err(err) => break err,
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    const LANES: usize = Chunk::LEN;

    /// `len` puts the byte that ends the scan on either side of the simd chunk edges
    #[rstest]
    #[case::closed("\"", true)]
    #[case::escape("\\n\"", true)]
    #[case::unicode_escape("\\u00e9\"", true)]
    #[case::multibyte("é\"", true)]
    #[case::bad_unicode_escape("\\u00g\"", false)]
    #[case::bad_escape("\\x\"", false)]
    #[case::control("\n\"", false)]
    #[case::unterminated_escape("\\", false)]
    #[case::unterminated("", false)]
    fn scan_agrees_with_state_machine(
        #[case] tail: &str,
        #[case] valid: bool,
        #[values(0, LANES - 2, LANES - 1, LANES, 2 * LANES - 1)] len: usize,
    ) {
        let text = format!("\"{}{tail}", "a".repeat(len));
        assert_eq!(scan_valid_string(&text, 0), valid.then_some(text.len()));
        // the state machine panics if it finds the string valid after all
        assert_eq!(parse_string(&text, 0).is_ok(), valid);
    }
}
