use core::range::Range;
use std::iter::Peekable;

use crate::{
    Error, ErrorKind, Result,
    tokens::{CharWithContext, JsonChar, Token, TokenWithContext},
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
    End(TokenWithContext),
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
                Some(CharWithContext(r, JsonChar('"'))) => StringState::End(TokenWithContext::new(
                    Token::String,
                    string_range.start..r.end,
                )),
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
                    if next_digits == 4 {
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
            StringState::End(_) => self,
        };

        Ok(res)
    }
}

/// Index of the first `"`, `\` or control byte at or after `from`.
fn find_special(bytes: &[u8], from: usize) -> Option<usize> {
    use std::simd::{
        cmp::{SimdPartialEq as _, SimdPartialOrd as _},
        u8x32,
    };

    let quote = u8x32::splat(b'"');
    let backslash = u8x32::splat(b'\\');
    let space = u8x32::splat(b' ');

    let mut i = from;
    while i + 32 <= bytes.len() {
        let chunk = u8x32::from_slice(&bytes[i..i + 32]);
        let special = chunk.simd_eq(quote) | chunk.simd_eq(backslash) | chunk.simd_lt(space);
        let mask = special.to_bitmask();
        if mask != 0 {
            return Some(i + mask.trailing_zeros() as usize);
        }
        i += 32;
    }

    bytes[i..]
        .iter()
        .position(|&b| matches!(b, b'"' | b'\\') || b < b' ')
        .map(|offset| i + offset)
}

/// End of the string starting at `pos` if it is valid, `None` otherwise.
///
/// Errors are left to [`StringState`] so their messages come from one place.
fn scan_valid_string(input: &str, pos: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut i = pos + 1;
    loop {
        i = find_special(bytes, i)?;
        match bytes[i] {
            b'"' => return Some(i + 1),
            b'\\' => match *bytes.get(i + 1)? {
                b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => i += 2,
                b'u' if bytes.get(i + 2..i + 6)?.iter().all(u8::is_ascii_hexdigit) => i += 6,
                _ => return None,
            },
            _ => return None,
        }
    }
}

pub fn parse_string(input: &str, pos: usize) -> Result<TokenWithContext> {
    if let Some(end) = scan_valid_string(input, pos) {
        return Ok(TokenWithContext::new(Token::String, pos..end));
    }

    parse_string_slow(input, pos)
}

fn parse_string_slow(input: &str, pos: usize) -> Result<TokenWithContext> {
    let mut chars = input[pos..]
        .char_indices()
        .map(|(i, c)| (i + pos, c).into())
        .peekable();
    let mut state = StringState::Open;

    loop {
        state = state.process(&mut chars, input)?;
        if let StringState::End(tok) = state {
            break Ok(tok);
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    /// `len` puts the special byte on either side of the 32 byte simd chunk edges
    #[rstest]
    #[case::closed("\"")]
    #[case::escape("\\n\"")]
    #[case::unicode_escape("\\u00e9\"")]
    #[case::bad_unicode_escape("\\u00g\"")]
    #[case::bad_escape("\\x\"")]
    #[case::control("\n\"")]
    #[case::multibyte("é\"")]
    #[case::unterminated_escape("\\")]
    #[case::unterminated("")]
    fn fast_scan_matches_state_machine(
        #[case] tail: &str,
        #[values(0, 30, 31, 32, 63)] len: usize,
    ) {
        let text = format!("\"{}{tail}", "a".repeat(len));
        assert_eq!(parse_string(&text, 0), parse_string_slow(&text, 0));
    }
}
