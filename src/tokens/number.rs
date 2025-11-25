use core::{iter::Peekable, ops::Range};

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
        number_ctx: Range<usize>,
    },
    Fraction {
        number_ctx: Range<usize>,
        dot_ctx: CharWithContext,
    },
    FractionOrExponentOrEnd(Range<usize>),
    MinusOrPlusOrDigit {
        number_ctx: Range<usize>,
        e_ctx: CharWithContext,
    },
    ExponentDigit {
        number_ctx: Range<usize>,
        e_ctx: CharWithContext,
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
                Some(CharWithContext(range, c @ JsonChar('0'..='9'))) => {
                    NumberState::IntegerOrDecimalOrExponentOrEnd {
                        leading: Some(CharWithContext(range.clone(), c)),
                        number_ctx: range,
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
            NumberState::Leading(number_ctx) => match chars.next() {
                Some(CharWithContext(leading_range, c @ JsonChar('0'..='9'))) => {
                    NumberState::IntegerOrDecimalOrExponentOrEnd {
                        leading: Some(CharWithContext(leading_range.clone(), c)),
                        number_ctx: number_ctx.start..leading_range.end,
                    }
                }
                maybe_char @ (Some(_) | None) => {
                    return Err(Error::new(
                        ErrorKind::ExpectedDigitFollowingMinus(
                            number_ctx.clone(),
                            maybe_char.map(|CharWithContext(_, c)| c).into(),
                        ),
                        number_ctx,
                        input,
                    ));
                }
            },
            NumberState::IntegerOrDecimalOrExponentOrEnd {
                leading,
                number_ctx,
            } => match (leading.as_ref(), chars.peek().cloned()) {
                (
                    Some(CharWithContext(initial_range, JsonChar('0'))),
                    Some(CharWithContext(_, JsonChar('0'))),
                ) => {
                    while chars
                        .next_if(|CharWithContext(_, JsonChar(c))| *c == '0')
                        .is_some()
                    {}
                    return Err(Error::new(
                        ErrorKind::UnexpectedLeadingZero {
                            initial: initial_range.clone(),
                            extra: initial_range.end
                                ..chars
                                    .peek()
                                    .map(|CharWithContext(range, _)| range.start)
                                    .unwrap_or(input.len()),
                        },
                        number_ctx.start
                            ..chars
                                .peek()
                                .map(|CharWithContext(range, _)| range.start)
                                .unwrap_or(input.len()),
                        input,
                    ));
                }
                (_, Some(CharWithContext(range, JsonChar('0'..='9')))) => {
                    chars.next();
                    NumberState::IntegerOrDecimalOrExponentOrEnd {
                        leading: None,
                        number_ctx: number_ctx.start..range.end,
                    }
                }
                (_, Some(ref dot @ CharWithContext(ref range, JsonChar('.')))) => {
                    chars.next();
                    NumberState::Fraction {
                        number_ctx: number_ctx.start..range.end,
                        dot_ctx: dot.clone(),
                    }
                }
                (_, Some(ref exponent @ CharWithContext(_, JsonChar('e' | 'E')))) => {
                    chars.next();
                    NumberState::MinusOrPlusOrDigit {
                        number_ctx: number_ctx.start..exponent.0.end,
                        e_ctx: exponent.clone(),
                    }
                }
                _ => Self::make_end(input, number_ctx),
            },
            NumberState::Fraction {
                number_ctx,
                dot_ctx,
            } => match chars.peek().cloned() {
                Some(CharWithContext(range, JsonChar('0'..='9'))) => {
                    chars.next();
                    NumberState::FractionOrExponentOrEnd(number_ctx.start..range.end)
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedDigitAfterDot {
                            number_ctx: number_ctx.clone(),
                            dot_ctx: dot_ctx.0.clone(),
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
                        number_ctx: ctx.start..range.end,
                        e_ctx: exponent.clone(),
                    }
                }
                _ => Self::make_end(input, ctx),
            },
            NumberState::MinusOrPlusOrDigit { number_ctx, e_ctx } => match chars.peek().cloned() {
                Some(CharWithContext(range, JsonChar('+' | '-'))) => {
                    chars.next();
                    NumberState::ExponentDigit {
                        number_ctx: number_ctx.start..range.end,
                        e_ctx,
                    }
                }
                Some(CharWithContext(range, JsonChar('0'..='9'))) => {
                    chars.next();
                    NumberState::ExponentDigitOrEnd(number_ctx.start..range.end)
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedPlusOrMinusOrDigitAfterE {
                            number_ctx: number_ctx.clone(),
                            e_ctx: e_ctx.0.clone(),
                            maybe_c: c,
                        },
                        maybe_c,
                        input,
                    ));
                }
            },
            NumberState::ExponentDigit { number_ctx, e_ctx } => match chars.peek().cloned() {
                Some(CharWithContext(range, JsonChar('0'..='9'))) => {
                    chars.next();
                    NumberState::ExponentDigitOrEnd(number_ctx.start..range.end)
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedDigitAfterE {
                            number_ctx: number_ctx.clone(),
                            exponent_ctx: e_ctx.0.clone(),
                            maybe_c: c,
                        },
                        maybe_c,
                        input,
                    ));
                }
            },
            NumberState::ExponentDigitOrEnd(number_ctx) => match chars.peek().cloned() {
                Some(CharWithContext(range, JsonChar('0'..='9'))) => {
                    chars.next();
                    NumberState::ExponentDigitOrEnd(number_ctx.start..range.end)
                }
                _ => Self::make_end(input, number_ctx),
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
