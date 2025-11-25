use core::{iter::Peekable, ops::Range};

use crate::{
    Error, ErrorKind, Result,
    tokens::{CharWithContext, Token, TokenWithContext},
};

#[derive(Debug, PartialEq, Eq, Clone)]
enum NumberState {
    MinusOrInteger,
    Leading(Range<usize>),
    IntegerOrDecimalOrExponentOrEnd {
        leading: Option<char>,
        leading_ctx: Range<usize>,
        number_ctx: Range<usize>,
    },
    Fraction {
        number_ctx: Range<usize>,
        dot_ctx: Range<usize>,
    },
    FractionOrExponentOrEnd(Range<usize>),
    MinusOrPlusOrDigit {
        number_ctx: Range<usize>,
        e_ctx: Range<usize>,
    },
    ExponentDigit {
        number_ctx: Range<usize>,
        e_ctx: Range<usize>,
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
                Some(CharWithContext(range, '-')) => NumberState::Leading(range),
                Some(CharWithContext(range, leading @ '0'..='9')) => {
                    NumberState::IntegerOrDecimalOrExponentOrEnd {
                        leading: Some(leading),
                        leading_ctx: range.clone(),
                        number_ctx: range,
                    }
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        ErrorKind::ExpectedMinusOrDigit,
                        0,
                        maybe_c.map(|CharWithContext(range, c)| (range.start, c)),
                        input,
                    ));
                }
            },
            NumberState::Leading(leading_range) => match chars.next() {
                Some(CharWithContext(digit_range, digit @ '0'..='9')) => {
                    NumberState::IntegerOrDecimalOrExponentOrEnd {
                        leading: Some(digit),
                        leading_ctx: digit_range.clone(),
                        number_ctx: leading_range.start..digit_range.end,
                    }
                }
                c @ (Some(_) | None) => {
                    let maybe_c = c;
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedDigitFollowingMinus(leading_range.clone(), c),
                        leading_range.start,
                        maybe_c.map(|CharWithContext(range, c)| (range.start, c)),
                        input,
                    ));
                }
            },
            NumberState::IntegerOrDecimalOrExponentOrEnd {
                leading,
                leading_ctx,
                number_ctx,
            } => match chars.peek().cloned() {
                Some(CharWithContext(_, '0')) if matches!(leading, Some('0')) => {
                    while chars.peek().is_some_and(|CharWithContext(_, c)| *c == '0') {
                        chars.next();
                    }
                    return Err(Error::new(
                        ErrorKind::UnexpectedLeadingZero {
                            initial: leading_ctx.clone(),
                            extra: leading_ctx.end
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
                Some(CharWithContext(range, '0'..='9')) => {
                    chars.next();
                    NumberState::IntegerOrDecimalOrExponentOrEnd {
                        leading: None,
                        leading_ctx,
                        number_ctx: number_ctx.start..range.end,
                    }
                }
                Some(CharWithContext(range, '.')) => {
                    chars.next();
                    NumberState::Fraction {
                        number_ctx: number_ctx.start..range.end,
                        dot_ctx: range.clone(),
                    }
                }
                Some(CharWithContext(range, 'e' | 'E')) => {
                    chars.next();
                    NumberState::MinusOrPlusOrDigit {
                        number_ctx: number_ctx.start..range.end,
                        e_ctx: range.clone(),
                    }
                }
                _ => Self::make_end(input, number_ctx),
            },
            NumberState::Fraction {
                number_ctx,
                dot_ctx,
            } => match chars.peek().cloned() {
                Some(CharWithContext(range, '0'..='9')) => {
                    chars.next();
                    NumberState::FractionOrExponentOrEnd(number_ctx.start..range.end)
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedDigitAfterDot {
                            number_ctx: number_ctx.clone(),
                            dot_ctx: dot_ctx.clone(),
                            maybe_c: c,
                        },
                        number_ctx.start,
                        maybe_c.map(|CharWithContext(range, c)| (range.start, c)),
                        input,
                    ));
                }
            },
            NumberState::FractionOrExponentOrEnd(ctx) => match chars.peek().cloned() {
                Some(CharWithContext(range, '0'..='9')) => {
                    chars.next();
                    NumberState::FractionOrExponentOrEnd(ctx.start..range.end)
                }
                Some(CharWithContext(range, 'e' | 'E')) => {
                    chars.next();
                    NumberState::MinusOrPlusOrDigit {
                        number_ctx: ctx.start..range.end,
                        e_ctx: range.clone(),
                    }
                }
                _ => Self::make_end(input, ctx),
            },
            NumberState::MinusOrPlusOrDigit { number_ctx, e_ctx } => match chars.peek().cloned() {
                Some(CharWithContext(range, '+' | '-')) => {
                    chars.next();
                    NumberState::ExponentDigit {
                        number_ctx: number_ctx.start..range.end,
                        e_ctx,
                    }
                }
                Some(CharWithContext(range, '0'..='9')) => {
                    chars.next();
                    NumberState::ExponentDigitOrEnd(number_ctx.start..range.end)
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedPlusOrMinusOrDigitAfterE {
                            number_ctx: number_ctx.clone(),
                            e_ctx: e_ctx.clone(),
                            maybe_c: c,
                        },
                        number_ctx.start,
                        maybe_c.map(|CharWithContext(range, c)| (range.start, c)),
                        input,
                    ));
                }
            },
            NumberState::ExponentDigit { number_ctx, e_ctx } => match chars.peek().cloned() {
                Some(CharWithContext(range, '0'..='9')) => {
                    chars.next();
                    NumberState::ExponentDigitOrEnd(number_ctx.start..range.end)
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedDigitAfterE {
                            number_ctx: number_ctx.clone(),
                            exponent_ctx: e_ctx.clone(),
                            maybe_c: c,
                        },
                        e_ctx.start,
                        maybe_c.map(|CharWithContext(range, c)| (range.start, c)),
                        input,
                    ));
                }
            },
            NumberState::ExponentDigitOrEnd(number_ctx) => match chars.peek().cloned() {
                Some(CharWithContext(range, '0'..='9')) => {
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
