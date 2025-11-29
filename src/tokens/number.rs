use core::{iter::Peekable, ops::Range};

use itertools::Itertools;

use crate::{
    Error, ErrorKind, Result,
    tokens::{CharWithContext, Token, TokenWithContext, lexical::JsonChar},
};

#[derive(Debug, PartialEq, Eq, Clone)]
enum NumberState {
    MinusOrInteger,
    Leading(Range<usize>),
    IntegerOrDecimalOrExponentOrEnd {
        leading: Option<CharWithContext>,
        number_range: Range<usize>,
    },
    Fraction {
        number_range: Range<usize>,
        dot_range: CharWithContext,
    },
    FractionOrExponentOrEnd(Range<usize>),
    MinusOrPlusOrDigit {
        number_range: Range<usize>,
        e_range: CharWithContext,
    },
    ExponentDigit {
        number_range: Range<usize>,
        e_range: CharWithContext,
    },
    ExponentDigitOrEnd(Range<usize>),
    End(TokenWithContext),
}

impl NumberState {
    fn make_end(s: &str, range: Range<usize>) -> Self {
        NumberState::End(TokenWithContext {
            token: Token::Number(s[range.clone()].into()),
            range,
        })
    }
    fn process(
        self,
        chars: &mut Peekable<impl Iterator<Item = CharWithContext>>,
        input: &str,
    ) -> Result<Self> {
        let res = match self {
            NumberState::MinusOrInteger => match chars.next() {
                Some(CharWithContext(range, JsonChar('-'))) => NumberState::Leading(range),
                Some(leading @ CharWithContext(_, JsonChar('0'..='9'))) => {
                    NumberState::IntegerOrDecimalOrExponentOrEnd {
                        leading: Some(leading.clone()),
                        number_range: leading.0,
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
            NumberState::Leading(number_range) => match chars.next() {
                Some(leading @ CharWithContext(_, JsonChar('0'..='9'))) => {
                    NumberState::IntegerOrDecimalOrExponentOrEnd {
                        leading: Some(leading.clone()),
                        number_range: number_range.start..leading.0.end,
                    }
                }
                maybe_char @ (Some(_) | None) => {
                    return Err(Error::new(
                        ErrorKind::ExpectedDigitFollowingMinus(
                            number_range.clone(),
                            maybe_char.map(|CharWithContext(_, c)| c).into(),
                        ),
                        number_range,
                        input,
                    ));
                }
            },
            NumberState::IntegerOrDecimalOrExponentOrEnd {
                leading,
                number_range,
            } => match (leading.as_ref(), chars.peek().cloned()) {
                (
                    Some(CharWithContext(initial_range, JsonChar('0'))),
                    Some(CharWithContext(_, JsonChar('0'..='9'))),
                ) => {
                    let final_zero_range = chars
                        .peeking_take_while(|CharWithContext(_, JsonChar(c))| *c == '0')
                        .last()
                        .map(|CharWithContext(r, _)| r)
                        .unwrap_or(initial_range.clone());

                    let extra_end = match chars.peek().cloned() {
                        Some(CharWithContext(_, JsonChar('1'..='9'))) => final_zero_range.end,
                        _ => final_zero_range.start,
                    };

                    return Err(Error::new(
                        ErrorKind::UnexpectedLeadingZero {
                            initial: initial_range.clone(),
                            extra: initial_range.start..extra_end,
                        },
                        number_range.start..final_zero_range.end,
                        input,
                    ));
                }
                (_, Some(CharWithContext(range, JsonChar('0'..='9')))) => {
                    chars.next();
                    NumberState::IntegerOrDecimalOrExponentOrEnd {
                        leading: None,
                        number_range: number_range.start..range.end,
                    }
                }
                (_, Some(ref dot @ CharWithContext(ref range, JsonChar('.')))) => {
                    chars.next();
                    NumberState::Fraction {
                        number_range: number_range.start..range.end,
                        dot_range: dot.clone(),
                    }
                }
                (_, Some(ref exponent @ CharWithContext(_, JsonChar('e' | 'E')))) => {
                    chars.next();
                    NumberState::MinusOrPlusOrDigit {
                        number_range: number_range.start..exponent.0.end,
                        e_range: exponent.clone(),
                    }
                }
                _ => Self::make_end(input, number_range),
            },
            NumberState::Fraction {
                number_range,
                dot_range,
            } => match chars.peek().cloned() {
                Some(CharWithContext(range, JsonChar('0'..='9'))) => {
                    chars.next();
                    NumberState::FractionOrExponentOrEnd(number_range.start..range.end)
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedDigitAfterDot {
                            number_range: number_range.clone(),
                            dot_range: dot_range.0.clone(),
                            maybe_c: c,
                        },
                        maybe_c,
                        input,
                    ));
                }
            },
            NumberState::FractionOrExponentOrEnd(ctx) => match chars.peek().cloned() {
                Some(CharWithContext(range, JsonChar('0'..='9'))) => {
                    chars.next();
                    NumberState::FractionOrExponentOrEnd(ctx.start..range.end)
                }
                Some(ref exponent @ CharWithContext(ref range, JsonChar('e' | 'E'))) => {
                    chars.next();
                    NumberState::MinusOrPlusOrDigit {
                        number_range: ctx.start..range.end,
                        e_range: exponent.clone(),
                    }
                }
                _ => Self::make_end(input, ctx),
            },
            NumberState::MinusOrPlusOrDigit {
                number_range,
                e_range,
            } => match chars.peek().cloned() {
                Some(CharWithContext(range, JsonChar('+' | '-'))) => {
                    chars.next();
                    NumberState::ExponentDigit {
                        number_range: number_range.start..range.end,
                        e_range,
                    }
                }
                Some(CharWithContext(range, JsonChar('0'..='9'))) => {
                    chars.next();
                    NumberState::ExponentDigitOrEnd(number_range.start..range.end)
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedPlusOrMinusOrDigitAfterE {
                            number_range: number_range.clone(),
                            e_range: e_range.0.clone(),
                            maybe_c: c,
                        },
                        maybe_c,
                        input,
                    ));
                }
            },
            NumberState::ExponentDigit {
                number_range,
                e_range,
            } => match chars.peek().cloned() {
                Some(CharWithContext(range, JsonChar('0'..='9'))) => {
                    chars.next();
                    NumberState::ExponentDigitOrEnd(number_range.start..range.end)
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedDigitAfterE {
                            number_range: number_range.clone(),
                            exponent_range: e_range.0.clone(),
                            maybe_c: c,
                        },
                        maybe_c,
                        input,
                    ));
                }
            },
            NumberState::ExponentDigitOrEnd(number_range) => match chars.peek().cloned() {
                Some(CharWithContext(range, JsonChar('0'..='9'))) => {
                    chars.next();
                    NumberState::ExponentDigitOrEnd(number_range.start..range.end)
                }
                _ => Self::make_end(input, number_range),
            },
            NumberState::End(_) => self,
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
pub fn parse_num(
    input: &str,
    chars: &mut Peekable<impl Iterator<Item = CharWithContext>>,
) -> Result<TokenWithContext> {
    let mut state = NumberState::MinusOrInteger;

    loop {
        state = state.process(chars, input)?;
        if let NumberState::End(tok) = state {
            break Ok(tok);
        }
    }
}
