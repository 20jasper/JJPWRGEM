use core::{iter::Peekable, range::Range};

use crate::{
    Error, ErrorKind, Result,
    tokens::{
        ByteWithContext, CharWithContext, Token, TokenWithContext, current_byte_pos,
        lexical::JsonByte,
    },
};

fn current_char_with_context(
    input: &str,
    bytes: &mut Peekable<impl Iterator<Item = ByteWithContext>>,
) -> Option<CharWithContext> {
    let pos = current_byte_pos(bytes, input);
    input[pos..].chars().next().map(|c| (pos, c).into())
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum MantissaState<'a> {
    MinusOrInteger,
    Leading(Range<usize>),
    IntegerOrDecimalOrEnd {
        leading_zero: Option<Range<usize>>,
        mantissa: Range<usize>,
    },
    Fraction {
        mantissa: Range<usize>,
        dot_range: Range<usize>,
    },
    FractionOrEnd(Range<usize>),
    End(TokenWithContext<'a>),
}

impl<'a> MantissaState<'a> {
    fn finish(s: &'a str, mantissa: Range<usize>) -> Self {
        MantissaState::End(TokenWithContext {
            token: Token::Mantissa(&s[mantissa]),
            range: mantissa,
        })
    }

    fn process(
        self,
        bytes: &mut Peekable<impl Iterator<Item = ByteWithContext>>,
        input: &'a str,
    ) -> Result<'a, Self> {
        let res = match self {
            MantissaState::MinusOrInteger => match bytes.peek().copied() {
                Some(byte @ ByteWithContext(_, JsonByte(b'-'))) => {
                    bytes.next();
                    MantissaState::Leading(byte.range())
                }
                Some(byte @ ByteWithContext(_, JsonByte(b'0'))) => {
                    bytes.next();
                    let range = byte.range();
                    MantissaState::IntegerOrDecimalOrEnd {
                        leading_zero: Some(range),
                        mantissa: range,
                    }
                }
                Some(byte @ ByteWithContext(_, JsonByte(b'1'..=b'9'))) => {
                    bytes.next();
                    let range = byte.range();
                    MantissaState::IntegerOrDecimalOrEnd {
                        leading_zero: None,
                        mantissa: range,
                    }
                }
                _ => {
                    return Err(Error::from_maybe_json_char_with_context(
                        ErrorKind::ExpectedMinusOrDigit,
                        current_char_with_context(input, bytes),
                        input,
                    ));
                }
            },
            MantissaState::Leading(mantissa) => match bytes.peek().copied() {
                Some(byte @ ByteWithContext(_, JsonByte(b'0'))) => {
                    bytes.next();
                    MantissaState::IntegerOrDecimalOrEnd {
                        leading_zero: Some(byte.range()),
                        mantissa: mantissa.start..byte.range().end,
                    }
                }
                Some(byte @ ByteWithContext(_, JsonByte(b'1'..=b'9'))) => {
                    bytes.next();
                    MantissaState::IntegerOrDecimalOrEnd {
                        leading_zero: None,
                        mantissa: mantissa.start..byte.range().end,
                    }
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::ExpectedDigitFollowingMinus(
                            mantissa,
                            current_char_with_context(input, bytes)
                                .map(|CharWithContext(_, c)| c)
                                .into(),
                        ),
                        mantissa,
                        input,
                    ));
                }
            },
            MantissaState::IntegerOrDecimalOrEnd {
                leading_zero,
                mantissa,
            } => match (leading_zero, bytes.peek().copied()) {
                (Some(initial_range), Some(ByteWithContext(_, JsonByte(b'0'..=b'9')))) => {
                    let mut final_zero_range = initial_range;
                    while let Some(byte @ ByteWithContext(_, JsonByte(b'0'))) =
                        bytes.peek().copied()
                    {
                        final_zero_range = byte.range();
                        bytes.next();
                    }

                    let extra_end = match bytes.peek().copied() {
                        Some(ByteWithContext(_, JsonByte(b'1'..=b'9'))) => final_zero_range.end,
                        _ => final_zero_range.start,
                    };

                    return Err(Error::new(
                        ErrorKind::UnexpectedLeadingZero {
                            initial: initial_range,
                            extra: initial_range.start..extra_end,
                        },
                        mantissa.start..final_zero_range.end,
                        input,
                    ));
                }
                (_, Some(byte @ ByteWithContext(_, JsonByte(b'0'..=b'9')))) => {
                    bytes.next();
                    MantissaState::IntegerOrDecimalOrEnd {
                        leading_zero: None,
                        mantissa: mantissa.start..byte.range().end,
                    }
                }
                (_, Some(byte @ ByteWithContext(_, JsonByte(b'.')))) => {
                    bytes.next();
                    MantissaState::Fraction {
                        mantissa: mantissa.start..byte.range().end,
                        dot_range: byte.range(),
                    }
                }
                _ => Self::finish(input, mantissa),
            },
            MantissaState::Fraction {
                mantissa,
                dot_range,
            } => match bytes.peek().copied() {
                Some(byte @ ByteWithContext(_, JsonByte(b'0'..=b'9'))) => {
                    bytes.next();
                    MantissaState::FractionOrEnd(mantissa.start..byte.range().end)
                }
                _ => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedDigitAfterDot {
                            number_range: mantissa,
                            dot_range,
                            maybe_c: c,
                        },
                        current_char_with_context(input, bytes),
                        input,
                    ));
                }
            },
            MantissaState::FractionOrEnd(mantissa) => match bytes.peek().copied() {
                Some(byte @ ByteWithContext(_, JsonByte(b'0'..=b'9'))) => {
                    bytes.next();
                    MantissaState::FractionOrEnd(mantissa.start..byte.range().end)
                }
                _ => Self::finish(input, mantissa),
            },
            MantissaState::End(_) => self,
        };

        Ok(res)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum ExponentState<'a> {
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
    End(&'a str, Range<usize>),
}

impl<'a> ExponentState<'a> {
    fn process(
        self,
        bytes: &mut Peekable<impl Iterator<Item = ByteWithContext>>,
        input: &'a str,
    ) -> Result<'a, Self> {
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
                        ExponentState::End(&input[exp_start..exp_end], exp_start..exp_end)
                    } else {
                        ExponentState::Zero
                    }
                }
            },
            ExponentState::Zero | ExponentState::End(_, _) => self,
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
pub fn parse_mantissa<'a>(
    input: &'a str,
    bytes: &mut Peekable<impl Iterator<Item = ByteWithContext>>,
) -> Result<'a, TokenWithContext<'a>> {
    let mut state = MantissaState::MinusOrInteger;

    loop {
        state = state.process(bytes, input)?;
        if let MantissaState::End(tok) = state {
            break Ok(tok);
        }
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
pub fn parse_exponent<'a>(
    input: &'a str,
    exponent_range: Range<usize>,
    bytes: &mut Peekable<impl Iterator<Item = ByteWithContext>>,
) -> Result<'a, Option<TokenWithContext<'a>>> {
    let mut state = ExponentState::MinusOrPlusOrDigit { exponent_range };

    loop {
        state = state.process(bytes, input)?;
        match state {
            ExponentState::End(slice, range) => {
                break Ok(Some(TokenWithContext {
                    token: Token::Exponent(slice),
                    range,
                }));
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
    use crate::tokens::{BytesWithContext, stream};

    fn str_to_tokens<'a>(s: &'a str) -> Result<'a, Vec<TokenWithContext<'a>>> {
        stream::TokenStream::new(s).collect()
    }

    fn parse_mantissa_from_start<'a>(input: &'a str) -> (Result<'a, TokenWithContext<'a>>, usize) {
        let mut bytes = BytesWithContext::new(input, 0).peekable();
        let mut probe = bytes.clone();
        let result = parse_mantissa(input, &mut probe);
        let pos = if result.is_ok() {
            current_byte_pos(&mut probe, input)
        } else {
            current_byte_pos(&mut bytes, input)
        };
        (result, pos)
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
            [m] => (&input[m.range.start..m.range.end], None),
            [m, e] => (
                &input[m.range.start..m.range.end],
                Some(&input[e.range.start..e.range.end]),
            ),
            _ => panic!("unexpected token count: {}", tokens.len()),
        };
        assert_eq!(result, (mantissa, exponent));
    }

    #[test]
    fn mantissa_updates_position_on_success() {
        let (token, pos) = parse_mantissa_from_start("-12.34x");
        let token = token.unwrap();

        assert_eq!(token.range, 0..6);
        assert_eq!(pos, 6);
    }

    #[test]
    fn mantissa_keeps_utf8_boundary() {
        let (token, pos) = parse_mantissa_from_start("1é");
        let token = token.unwrap();

        assert_eq!(token.range, 0..1);
        assert_eq!(pos, 1);
    }

    #[test]
    fn mantissa_does_not_advance_position_on_error() {
        let (result, pos) = parse_mantissa_from_start("-é");

        assert!(result.is_err());
        assert_eq!(pos, 0);
    }
}
