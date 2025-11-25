use core::{iter::Peekable, ops::Range, str::CharIndices};

use crate::{
    Error, ErrorKind, Result,
    tokens::{Token, TokenWithContext},
};

#[derive(Debug, PartialEq, Eq, Clone)]
enum NumberState {
    MinusOrInteger,
    Leading(Range<usize>), // TODO char with context type??
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
        exponent_ctx: Range<usize>,
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
        chars: &mut Peekable<impl Iterator<Item = (usize, char)>>,
        input: &str,
    ) -> Result<Self> {
        let res = match self {
            NumberState::MinusOrInteger => match chars.next() {
                Some((i, c @ '-')) => NumberState::Leading(i..i + c.len_utf8()),
                Some((i, leading @ '0'..='9')) => NumberState::IntegerOrDecimalOrExponentOrEnd {
                    leading: Some(leading),
                    leading_ctx: i..i + leading.len_utf8(),
                    number_ctx: i..i + leading.len_utf8(),
                },
                _ => todo!("err, number must start with `-` or digit"),
            },
            NumberState::Leading(range) => match chars.next() {
                Some((i, digit @ '0'..='9')) => NumberState::IntegerOrDecimalOrExponentOrEnd {
                    leading: Some(digit),
                    leading_ctx: i..i + digit.len_utf8(),
                    number_ctx: range.start..i + digit.len_utf8(),
                },
                c @ (Some(_) | None) => {
                    let maybe_c = c;
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedDigitFollowingMinus(range.clone(), c),
                        range.start,
                        maybe_c,
                        input,
                    ));
                }
            },
            NumberState::IntegerOrDecimalOrExponentOrEnd {
                leading,
                leading_ctx,
                number_ctx,
            } => match chars.peek() {
                Some((_, '0')) if matches!(leading, Some('0')) => {
                    while chars.next_if(|(_, c)| *c == '0').is_some() {}
                    let end = chars.peek().map(|&(i, _)| i).unwrap_or(input.len());

                    return Err(Error::new(
                        ErrorKind::UnexpectedLeadingZero {
                            initial: leading_ctx.clone(),
                            extra: leading_ctx.end..end,
                        },
                        number_ctx.start..end,
                        input,
                    ));
                }
                Some(&(i, c @ '0'..='9')) => {
                    chars.next();
                    NumberState::IntegerOrDecimalOrExponentOrEnd {
                        leading: None,
                        leading_ctx,
                        number_ctx: number_ctx.start..i + c.len_utf8(),
                    }
                }
                Some(&(i, c @ '.')) => {
                    chars.next();
                    NumberState::Fraction {
                        number_ctx: number_ctx.start..i + c.len_utf8(),
                        dot_ctx: i..c.len_utf8(),
                    }
                }
                Some(&(i, c @ ('e' | 'E'))) => {
                    chars.next();
                    NumberState::MinusOrPlusOrDigit {
                        number_ctx: number_ctx.start..i + c.len_utf8(),
                        e_ctx: i..i + c.len_utf8(),
                    }
                }
                _ => Self::make_end(input, number_ctx),
            },
            NumberState::Fraction {
                number_ctx,
                dot_ctx,
            } => match chars.peek() {
                Some(&(i, c @ '0'..='9')) => {
                    chars.next();
                    NumberState::FractionOrExponentOrEnd(number_ctx.start..i + c.len_utf8())
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedDigitAfterDot {
                            number_ctx: number_ctx.clone(),
                            dot_ctx: dot_ctx.clone(),
                            maybe_c: c,
                        },
                        number_ctx.start,
                        maybe_c.copied(),
                        input,
                    ));
                }
            },
            NumberState::FractionOrExponentOrEnd(ctx) => match chars.peek() {
                Some(&(i, c @ '0'..='9')) => {
                    chars.next();
                    NumberState::FractionOrExponentOrEnd(ctx.start..i + c.len_utf8())
                }
                Some(&(i, c @ ('e' | 'E'))) => {
                    chars.next();
                    NumberState::MinusOrPlusOrDigit {
                        number_ctx: ctx.start..i + c.len_utf8(),
                        e_ctx: i..i + c.len_utf8(),
                    }
                }
                _ => Self::make_end(input, ctx),
            },
            NumberState::MinusOrPlusOrDigit { number_ctx, e_ctx } => match chars.peek() {
                Some(&(i, c @ ('+' | '-'))) => {
                    chars.next();
                    NumberState::ExponentDigit {
                        number_ctx: number_ctx.start..i + c.len_utf8(),
                        exponent_ctx: i + i..c.len_utf8(),
                    }
                }
                Some(&(i, c @ '0'..='9')) => {
                    chars.next();
                    NumberState::ExponentDigitOrEnd(number_ctx.start..i + c.len_utf8())
                }
                maybe_c => {
                    return Err(Error::from_maybe_json_char_with_context(
                        |c| ErrorKind::ExpectedPlusOrMinusOrDigitAfterE {
                            number_ctx: number_ctx.clone(),
                            e_ctx: e_ctx.clone(),
                            maybe_c: c,
                        },
                        number_ctx.start,
                        maybe_c.copied(),
                        input,
                    ));
                }
            },
            NumberState::ExponentDigit {
                number_ctx,
                exponent_ctx,
            } => match chars.peek() {
                Some(&(i, c @ ('0'..='9'))) => {
                    chars.next();
                    NumberState::ExponentDigitOrEnd(number_ctx.start..i + c.len_utf8())
                }
                _ => todo!("expected at least one digit after +/- in exponent"),
            },
            NumberState::ExponentDigitOrEnd(number_ctx) => match chars.peek() {
                Some(&(i, c @ ('0'..='9'))) => {
                    chars.next();
                    NumberState::ExponentDigitOrEnd(number_ctx.start..i + c.len_utf8())
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
pub fn parse_num<'a>(
    input: &'a str,
    chars: &mut Peekable<CharIndices<'a>>,
) -> Result<TokenWithContext> {
    let mut state = NumberState::MinusOrInteger;

    loop {
        state = state.process(chars, input)?;
        if let NumberState::End(tok) = state {
            break Ok(tok);
        }
    }

    // Does the RFC require whitespace after nums?
    // can start with - optionally
    // can't start with extra leading 0

    // exponent E or e
}
