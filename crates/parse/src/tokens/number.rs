use core::{iter::Peekable, range::Range};

use crate::{
    Error, ErrorKind, Result,
    tokens::{
        ByteWithContext, CharWithContext, Token, TokenWithContext, byte_range, current_byte_pos,
        lexical::JsonByte,
    },
};

fn char_at(input: &str, pos: usize) -> Option<CharWithContext> {
    input[pos..].chars().next().map(|c| (pos, c).into())
}

fn current_char_with_context(
    input: &str,
    bytes: &mut Peekable<impl Iterator<Item = ByteWithContext>>,
) -> Option<CharWithContext> {
    char_at(input, current_byte_pos(bytes, input))
}

/// Where [`parse_mantissa`] is in the number grammar, without data so it stays one byte
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
enum MantissaState {
    MinusOrInteger,
    Minus,
    Zero,
    Integer,
    Dot,
    Fraction,
    End,
}

impl MantissaState {
    fn process(self, next: Option<u8>) -> Option<Self> {
        let state = match self {
            MantissaState::MinusOrInteger => match next? {
                b'-' => MantissaState::Minus,
                b'0' => MantissaState::Zero,
                b'1'..=b'9' => MantissaState::Integer,
                _ => return None,
            },
            MantissaState::Minus => match next? {
                b'0' => MantissaState::Zero,
                b'1'..=b'9' => MantissaState::Integer,
                _ => return None,
            },
            MantissaState::Zero => match next {
                // a digit after a leading zero
                Some(b'0'..=b'9') => return None,
                Some(b'.') => MantissaState::Dot,
                _ => MantissaState::End,
            },
            MantissaState::Integer => match next {
                Some(b'0'..=b'9') => MantissaState::Integer,
                Some(b'.') => MantissaState::Dot,
                _ => MantissaState::End,
            },
            MantissaState::Dot => match next? {
                b'0'..=b'9' => MantissaState::Fraction,
                _ => return None,
            },
            MantissaState::Fraction => match next {
                Some(b'0'..=b'9') => MantissaState::Fraction,
                _ => MantissaState::End,
            },
            MantissaState::End => self,
        };
        Some(state)
    }
}

fn mantissa_error(state: MantissaState, input: &str, start: usize, at: usize) -> Error {
    match state {
        MantissaState::MinusOrInteger => Error::from_maybe_json_char_with_context(
            ErrorKind::ExpectedMinusOrDigit,
            char_at(input, at),
            input,
        ),
        MantissaState::Minus => {
            let minus = byte_range(start);
            Error::new(
                ErrorKind::ExpectedDigitFollowingMinus(
                    minus,
                    char_at(input, at).map(|CharWithContext(_, c)| c).into(),
                ),
                minus,
                input,
            )
        }
        MantissaState::Zero => leading_zero_error(input, start, at),
        MantissaState::Dot => Error::from_maybe_json_char_with_context(
            |c| ErrorKind::ExpectedDigitAfterDot {
                number_range: start..at,
                dot_range: byte_range(at - 1),
                maybe_c: c,
            },
            char_at(input, at),
            input,
        ),
        MantissaState::Integer | MantissaState::Fraction | MantissaState::End => {
            unreachable!("{state:?} accepts any byte")
        }
    }
}

/// `at` is the digit that follows the leading zero at `at - 1`.
fn leading_zero_error(input: &str, start: usize, at: usize) -> Error {
    let bytes = input.as_bytes();
    let initial = byte_range(at - 1);
    let mut end = at;
    while bytes.get(end) == Some(&b'0') {
        end += 1;
    }

    // the last zero, which is the first one when no more follow
    let last_zero = byte_range(end - 1);
    let extra_end = match bytes.get(end) {
        Some(b'1'..=b'9') => last_zero.end,
        _ => last_zero.start,
    };

    Error::new(
        ErrorKind::UnexpectedLeadingZero {
            initial,
            extra: initial.start..extra_end,
        },
        start..last_zero.end,
        input,
    )
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum ExponentState {
    MinusOrPlusOrDigit {
        exponent_range: Range<usize>,
    },
    AfterSign {
        exponent_range: Range<usize>,
        sign_end: usize,
        exp_start: Option<usize>,
    },
    Digits {
        exp_start: usize,
        has_nonzero: bool,
        exp_end: usize,
    },
    Zero,
    End(Range<usize>),
}

impl ExponentState {
    fn process(
        self,
        bytes: &mut Peekable<impl Iterator<Item = ByteWithContext>>,
        input: &str,
    ) -> Result<Self> {
        let res = match self {
            ExponentState::MinusOrPlusOrDigit { exponent_range } => match bytes.peek().copied() {
                Some(byte @ ByteWithContext(_, JsonByte(b'+'))) => {
                    bytes.next();
                    ExponentState::AfterSign {
                        exponent_range,
                        sign_end: byte.range().end,
                        exp_start: None,
                    }
                }
                Some(byte @ ByteWithContext(_, JsonByte(b'-'))) => {
                    bytes.next();
                    ExponentState::AfterSign {
                        exponent_range,
                        sign_end: byte.range().end,
                        exp_start: Some(byte.range().start),
                    }
                }
                Some(byte @ ByteWithContext(_, JsonByte(d @ b'0'..=b'9'))) => {
                    bytes.next();
                    ExponentState::Digits {
                        exp_start: byte.range().start,
                        has_nonzero: d != b'0',
                        exp_end: byte.range().end,
                    }
                }
                _ => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedPlusOrMinusOrDigitAfterE {
                            e_range: exponent_range,
                            maybe_c: c,
                        },
                        current_char_with_context(input, bytes),
                        input,
                    ));
                }
            },
            ExponentState::AfterSign {
                exponent_range,
                sign_end,
                exp_start,
            } => match bytes.peek().copied() {
                Some(byte @ ByteWithContext(_, JsonByte(d @ b'0'..=b'9'))) => {
                    bytes.next();
                    ExponentState::Digits {
                        exp_start: exp_start.unwrap_or(byte.range().start),
                        has_nonzero: d != b'0',
                        exp_end: byte.range().end,
                    }
                }
                _ => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedDigitAfterE {
                            exponent_range: exponent_range.start..sign_end,
                            maybe_c: c,
                        },
                        current_char_with_context(input, bytes),
                        input,
                    ));
                }
            },
            ExponentState::Digits {
                exp_start,
                has_nonzero,
                exp_end,
            } => match bytes.peek().copied() {
                Some(byte @ ByteWithContext(_, JsonByte(d @ b'0'..=b'9'))) => {
                    bytes.next();
                    ExponentState::Digits {
                        exp_start,
                        has_nonzero: has_nonzero || d != b'0',
                        exp_end: byte.range().end,
                    }
                }
                _ => {
                    if has_nonzero {
                        ExponentState::End(exp_start..exp_end)
                    } else {
                        ExponentState::Zero
                    }
                }
            },
            ExponentState::Zero | ExponentState::End(_) => self,
        };

        Ok(res)
    }
}

/// ```abnf
/// number        = [ minus ] int [ frac ] [ exp ]
/// decimal-point = %x2E              ; .
/// digit1-9      = %x31-39           ; 1-9
/// e             = %x65 / %x45       ; e E
/// exp           = e [ minus / plus ] 1*DIGIT
/// frac          = decimal-point 1*DIGIT
/// int           = zero / ( digit1-9 *DIGIT )
/// minus         = %x2D              ; -
/// plus          = %x2B              ; +
/// zero          = %x30              ; 0
/// ```
/// See [RFC 8259 Section 6](https://datatracker.ietf.org/doc/html/rfc8259#section-6)
pub fn parse_mantissa(input: &str, start: usize) -> Result<TokenWithContext> {
    let bytes = input.as_bytes();
    let mut at = start;
    let mut state = MantissaState::MinusOrInteger;

    loop {
        match state.process(bytes.get(at).copied()) {
            Some(MantissaState::End) => {
                break Ok(TokenWithContext::new(Token::Mantissa, start..at));
            }
            Some(next) => state = next,
            None => break Err(mantissa_error(state, input, start, at)),
        }
        at += 1;
    }
}

/// ```abnf
/// number        = [ minus ] int [ frac ] [ exp ]
/// decimal-point = %x2E              ; .
/// digit1-9      = %x31-39           ; 1-9
/// e             = %x65 / %x45       ; e E
/// exp           = e [ minus / plus ] 1*DIGIT
/// frac          = decimal-point 1*DIGIT
/// int           = zero / ( digit1-9 *DIGIT )
/// minus         = %x2D              ; -
/// plus          = %x2B              ; +
/// zero          = %x30              ; 0
/// ```
/// See [RFC 8259 Section 6](https://datatracker.ietf.org/doc/html/rfc8259#section-6)
pub fn parse_exponent(
    input: &str,
    exponent_range: Range<usize>,
    bytes: &mut Peekable<impl Iterator<Item = ByteWithContext>>,
) -> Result<Option<TokenWithContext>> {
    let mut state = ExponentState::MinusOrPlusOrDigit { exponent_range };

    loop {
        state = state.process(bytes, input)?;
        match state {
            ExponentState::End(range) => {
                break Ok(Some(TokenWithContext::new(Token::Exponent, range)));
            }
            ExponentState::Zero => break Ok(None),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::tokens::stream;

    fn str_to_tokens(s: &str) -> Result<Vec<TokenWithContext>> {
        stream::TokenStream::new(s).collect()
    }

    #[rstest]
    #[case("12", "12", None)]
    #[case("12e12", "12", Some("12"))]
    #[case("12e+12", "12", Some("12"))]
    #[case("12e-12", "12", Some("-12"))]
    #[case("1e-0000", "1", None)]
    #[case("1e+0000", "1", None)]
    #[case("1e0000", "1", None)]
    #[case("1e00000000", "1", None)]
    #[case("1e+0015", "1", Some("0015"))]
    #[case("1e00015", "1", Some("00015"))]
    #[case("1e-000015", "1", Some("-000015"))]
    #[case("1e-15", "1", Some("-15"))]
    fn number_tokens(#[case] input: &str, #[case] mantissa: &str, #[case] exponent: Option<&str>) {
        let tokens = str_to_tokens(input).unwrap();
        let result = match tokens.as_slice() {
            [m] => {
                let mr = m.range;
                (&input[mr.start..mr.end], None)
            }
            [m, e] => (
                &input[m.range.start..m.range.end],
                Some(&input[e.range.start..e.range.end]),
            ),
            _ => panic!("unexpected token count: {}", tokens.len()),
        };
        assert_eq!(result, (mantissa, exponent));
    }

    #[test]
    fn mantissa_stops_at_the_first_byte_that_is_not_part_of_it() {
        let token = parse_mantissa("-12.34x", 0).unwrap();

        assert_eq!(token.range, 0..6);
    }

    #[test]
    fn mantissa_keeps_utf8_boundary() {
        let token = parse_mantissa("1é", 0).unwrap();

        assert_eq!(token.range, 0..1);
    }

    #[test]
    fn mantissa_error_on_a_multibyte_char_does_not_split_it() {
        assert!(parse_mantissa("-é", 0).is_err());
    }
}
