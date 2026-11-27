use std::{cmp::Ordering, num::IntErrorKind};

use jjpwrgem_parse::Number;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Sign {
    Negative,
    Zero,
    Positive,
}

/// Decimal is normalized by treating the integer and fraction as a whole integer and changing the exponent,
/// then removing leading and trailing zeros so each value has one form
/// 1.0e1 -> 10e0 -> 1e1
/// 15e-1 -> 15e-1
#[derive(Debug, Clone, Copy, Eq)]
pub struct Decimal<'a> {
    sign: Sign,
    int: &'a str,
    frac: &'a str,
    exp: i64,
}

impl<'a, S: AsRef<str>> From<Number<'a, S>> for Decimal<'a> {
    fn from(number: Number<'a, S>) -> Self {
        let (mantissa, exponent) = (number.mantissa(), number.exponent());
        let (negative, unsigned) = mantissa
            .strip_prefix('-')
            .map_or((false, mantissa), |rest| (true, rest));
        let (int, frac) = unsigned.split_once('.').unwrap_or((unsigned, ""));
        let exp = match exponent.map(str::parse::<i64>) {
            None => 0,
            Some(Ok(exp)) => exp,
            Some(Err(err)) => match err.kind() {
                IntErrorKind::PosOverflow => i64::MAX,
                IntErrorKind::NegOverflow => i64::MIN,
                kind => unreachable!("parser only produces exponent digits, got {kind:?}"),
            },
        };
        let exp = fraction_to_exponent(frac, exp);
        // strip leading and trailing 0s
        let int = int.trim_start_matches('0');
        let frac = if int.is_empty() {
            frac.trim_start_matches('0')
        } else {
            frac
        };
        let (int, frac, exp) = trim_trailing_zeros(int, frac, exp);
        let (sign, exp) = if int.is_empty() && frac.is_empty() {
            (Sign::Zero, 0)
        } else if negative {
            (Sign::Negative, exp)
        } else {
            (Sign::Positive, exp)
        };
        Self {
            sign,
            int,
            frac,
            exp,
        }
    }
}

impl Decimal<'_> {
    pub fn is_integer(&self) -> bool {
        self.exp >= 0
    }

    fn digits(&self) -> impl Iterator<Item = u8> {
        self.int.bytes().chain(self.frac.bytes())
    }

    fn order_of_magnitude(&self) -> i64 {
        i64::try_from(self.int.len() + self.frac.len())
            .unwrap_or(i64::MAX)
            .saturating_add(self.exp)
    }
}

fn fraction_to_exponent(frac: &str, exp: i64) -> i64 {
    exp.saturating_sub(i64::try_from(frac.len()).unwrap_or(i64::MAX))
}

fn trim_trailing_zeros<'a>(int: &'a str, frac: &'a str, exp: i64) -> (&'a str, &'a str, i64) {
    let trimmed = frac.trim_end_matches('0');
    let exp = exp.saturating_add(i64::try_from(frac.len() - trimmed.len()).unwrap_or(i64::MAX));
    if !trimmed.is_empty() {
        return (int, trimmed, exp);
    }
    let int_trimmed = int.trim_end_matches('0');
    let exp = exp.saturating_add(i64::try_from(int.len() - int_trimmed.len()).unwrap_or(i64::MAX));
    (int_trimmed, trimmed, exp)
}

impl PartialEq for Decimal<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for Decimal<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Decimal<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sign.cmp(&other.sign).then_with(|| {
            let magnitude = self
                .order_of_magnitude()
                .cmp(&other.order_of_magnitude())
                .then_with(|| self.digits().cmp(other.digits()));
            if self.sign == Sign::Negative {
                magnitude.reverse()
            } else {
                magnitude
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use jjpwrgem_parse::{Kind, SyntaxOptions, ast::Document};

    use super::*;

    fn parsed(text: &str) -> Document<&str> {
        Document::parse(text, SyntaxOptions::JSON).unwrap()
    }

    fn dec<'a>(doc: &'a Document<&str>) -> Decimal<'a> {
        let Kind::Number(number) = doc.root().kind() else {
            panic!("number");
        };
        Decimal::from(number)
    }

    #[rstest::rstest]
    #[case("1", "1.0")]
    #[case("1", "10e-1")]
    #[case("1", "0.1e1")]
    #[case("-0", "0")]
    #[case("0", "0.0e5")]
    #[case("0", "-0.0")]
    #[case("100", "1e2")]
    #[case("1.5e1", "15")]
    #[case("0.0005", "5e-4")]
    #[case("1e+2", "100")]
    fn equal(#[case] lhs: &str, #[case] rhs: &str) {
        assert_eq!(dec(&parsed(lhs)), dec(&parsed(rhs)));
    }

    // r[verify json-schema-v7.type.integer-subset]
    #[rstest::rstest]
    #[case("1.5e1", true)]
    #[case("1e-1", false)]
    #[case("1.0", true)]
    #[case("1.5", false)]
    #[case("0", true)]
    #[case("-3e2", true)]
    #[case("12e-1", false)]
    fn integer_detection(#[case] text: &str, #[case] expected: bool) {
        assert_eq!(dec(&parsed(text)).is_integer(), expected);
    }

    #[rstest::rstest]
    #[case("-1", "0")]
    #[case("0", "0.5")]
    #[case("0.5", "1")]
    #[case("1", "1.5")]
    #[case("1.5", "10")]
    #[case("10", "1e100")]
    #[case("-10", "-1")]
    #[case("0.01", "0.1")]
    #[case("9.99", "10")]
    #[case("1", "1.0000001")]
    #[case("-1e100", "-1")]
    #[case("-1", "-0.5")]
    fn ordered(#[case] smaller: &str, #[case] larger: &str) {
        let (smaller, larger) = (parsed(smaller), parsed(larger));
        assert!(dec(&smaller) < dec(&larger));
        assert!(dec(&larger) > dec(&smaller));
    }

    #[test]
    fn huge_exponents() {
        let (big, small) = (
            parsed("1e99999999999999999999"),
            parsed("1e-99999999999999999999"),
        );
        let (one, neg_one, neg_big) =
            (parsed("1"), parsed("-1"), parsed("-1e99999999999999999999"));
        let (big, small) = (dec(&big), dec(&small));
        assert!(big.is_integer());
        assert!(!small.is_integer());
        assert!(small < dec(&one));
        assert!(dec(&one) < big);
        assert!(small < big);
        assert!(dec(&neg_big) < dec(&neg_one));
    }
}
