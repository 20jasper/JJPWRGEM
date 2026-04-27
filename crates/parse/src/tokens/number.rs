use core::{iter::Peekable, range::Range};

use itertools::Itertools;

use crate::{
    Error, ErrorKind, Result,
    tokens::{CharWithContext, Token, TokenWithContext, lexical::JsonChar},
};

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
        chars: &mut Peekable<impl Iterator<Item = CharWithContext>>,
        input: &'a str,
    ) -> Result<'a, Self> {
        let res = match self {
            MantissaState::MinusOrInteger => match chars.next() {
                Some(CharWithContext(range, JsonChar('-'))) => MantissaState::Leading(range),
                Some(CharWithContext(range, JsonChar('0'))) => {
                    MantissaState::IntegerOrDecimalOrEnd {
                        leading_zero: Some(range),
                        mantissa: range,
                    }
                }
                Some(CharWithContext(range, JsonChar('1'..='9'))) => {
                    MantissaState::IntegerOrDecimalOrEnd {
                        leading_zero: None,
                        mantissa: range,
                    }
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        ErrorKind::ExpectedMinusOrDigit,
                        maybe_c,
                        input,
                    ));
                }
            },
            MantissaState::Leading(mantissa) => match chars.next() {
                Some(CharWithContext(leading_range, JsonChar('0'))) => {
                    MantissaState::IntegerOrDecimalOrEnd {
                        leading_zero: Some(leading_range),
                        mantissa: mantissa.start..leading_range.end,
                    }
                }
                Some(CharWithContext(leading_range, JsonChar('1'..='9'))) => {
                    MantissaState::IntegerOrDecimalOrEnd {
                        leading_zero: None,
                        mantissa: mantissa.start..leading_range.end,
                    }
                }
                maybe_char @ (Some(_) | None) => {
                    return Err(Error::new(
                        ErrorKind::ExpectedDigitFollowingMinus(
                            mantissa,
                            maybe_char.map(|CharWithContext(_, c)| c).into(),
                        ),
                        mantissa,
                        input,
                    ));
                }
            },
            MantissaState::IntegerOrDecimalOrEnd {
                leading_zero,
                mantissa,
            } => match (leading_zero, chars.peek().copied()) {
                (Some(initial_range), Some(CharWithContext(_, JsonChar('0'..='9')))) => {
                    let final_zero_range = chars
                        .peeking_take_while(|CharWithContext(_, JsonChar(c))| *c == '0')
                        .last()
                        .map(|CharWithContext(r, _)| r)
                        .unwrap_or(initial_range);

                    let extra_end = match chars.peek().copied() {
                        Some(CharWithContext(_, JsonChar('1'..='9'))) => final_zero_range.end,
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
                (_, Some(CharWithContext(range, JsonChar('0'..='9')))) => {
                    chars.next();
                    MantissaState::IntegerOrDecimalOrEnd {
                        leading_zero: None,
                        mantissa: mantissa.start..range.end,
                    }
                }
                (_, Some(CharWithContext(dot_range, JsonChar('.')))) => {
                    chars.next();
                    MantissaState::Fraction {
                        mantissa: mantissa.start..dot_range.end,
                        dot_range,
                    }
                }
                _ => Self::finish(input, mantissa),
            },
            MantissaState::Fraction {
                mantissa,
                dot_range,
            } => match chars.peek().copied() {
                Some(CharWithContext(range, JsonChar('0'..='9'))) => {
                    chars.next();
                    MantissaState::FractionOrEnd(mantissa.start..range.end)
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedDigitAfterDot {
                            number_range: mantissa,
                            dot_range,
                            maybe_c: c,
                        },
                        maybe_c,
                        input,
                    ));
                }
            },
            MantissaState::FractionOrEnd(mantissa) => match chars.peek().copied() {
                Some(CharWithContext(range, JsonChar('0'..='9'))) => {
                    chars.next();
                    MantissaState::FractionOrEnd(mantissa.start..range.end)
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
        chars: &mut Peekable<impl Iterator<Item = CharWithContext>>,
        input: &'a str,
    ) -> Result<'a, Self> {
        let res = match self {
            ExponentState::MinusOrPlusOrDigit { exponent_range } => match chars.peek().copied() {
                Some(CharWithContext(range, JsonChar('+'))) => {
                    chars.next();
                    ExponentState::AfterSign {
                        exponent_range,
                        sign_end: range.end,
                        exp_start: None,
                    }
                }
                Some(CharWithContext(range, JsonChar('-'))) => {
                    chars.next();
                    ExponentState::AfterSign {
                        exponent_range,
                        sign_end: range.end,
                        exp_start: Some(range.start),
                    }
                }
                Some(CharWithContext(range, JsonChar(d @ '0'..='9'))) => {
                    chars.next();
                    ExponentState::Digits {
                        exp_start: range.start,
                        has_nonzero: d != '0',
                        exp_end: range.end,
                    }
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedPlusOrMinusOrDigitAfterE {
                            e_range: exponent_range,
                            maybe_c: c,
                        },
                        maybe_c,
                        input,
                    ));
                }
            },
            ExponentState::AfterSign {
                exponent_range,
                sign_end,
                exp_start,
            } => match chars.peek().copied() {
                Some(CharWithContext(range, JsonChar(d @ '0'..='9'))) => {
                    chars.next();
                    ExponentState::Digits {
                        exp_start: exp_start.unwrap_or(range.start),
                        has_nonzero: d != '0',
                        exp_end: range.end,
                    }
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedDigitAfterE {
                            exponent_range: exponent_range.start..sign_end,
                            maybe_c: c,
                        },
                        maybe_c,
                        input,
                    ));
                }
            },
            ExponentState::Digits {
                exp_start,
                has_nonzero,
                exp_end,
            } => match chars.peek().copied() {
                Some(CharWithContext(range, JsonChar(d @ '0'..='9'))) => {
                    chars.next();
                    ExponentState::Digits {
                        exp_start,
                        has_nonzero: has_nonzero || d != '0',
                        exp_end: range.end,
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
    chars: &mut Peekable<impl Iterator<Item = CharWithContext>>,
) -> Result<'a, TokenWithContext<'a>> {
    let mut state = MantissaState::MinusOrInteger;

    loop {
        state = state.process(chars, input)?;
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
    chars: &mut Peekable<impl Iterator<Item = CharWithContext>>,
) -> Result<'a, Option<TokenWithContext<'a>>> {
    let mut state = ExponentState::MinusOrPlusOrDigit { exponent_range };

    loop {
        state = state.process(chars, input)?;
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
    use crate::tokens::stream;

    fn str_to_tokens<'a>(s: &'a str) -> Result<'a, Vec<TokenWithContext<'a>>> {
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
            [m] => (&input[m.range.start..m.range.end], None),
            [m, e] => (
                &input[m.range.start..m.range.end],
                Some(&input[e.range.start..e.range.end]),
            ),
            _ => panic!("unexpected token count: {}", tokens.len()),
        };
        assert_eq!(result, (mantissa, exponent));
    }
}
