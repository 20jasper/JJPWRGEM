use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Sign {
    Negative,
    Zero,
    Positive,
}

impl Sign {
    /// Orders two numbers of this sign from how their magnitudes compare, which is only computed when the sign is not zero.
    fn then_with(self, magnitude: impl FnOnce() -> Ordering) -> Ordering {
        match self {
            Self::Zero => Ordering::Equal,
            Self::Positive => magnitude(),
            Self::Negative => magnitude().reverse(),
        }
    }
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

impl<'a> Decimal<'a> {
    /// `mantissa` includes any leading `-`. `exponent` excludes `e`/`E` but may carry a sign.
    /// Expects validated JSON number text, so digits only, as produced by the parser.
    pub fn from_parts(mantissa: &'a str, exponent: Option<&str>) -> Self {
        let (negative, unsigned) = mantissa
            .strip_prefix('-')
            .map_or((false, mantissa), |rest| (true, rest));
        let (int, frac) = unsigned.split_once('.').unwrap_or((unsigned, ""));
        let exp = parse_exponent(exponent).saturating_sub(to_i64(frac.len()));
        let (int, frac) = trim_leading_zeros(int, frac);
        let (int, frac, exp) = trim_trailing_zeros(int, frac, exp);
        Self::new(negative, int, frac, exp)
    }

    fn new(negative: bool, int: &'a str, frac: &'a str, exp: i64) -> Self {
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

    pub fn is_integer(&self) -> bool {
        self.exp >= 0
    }

    fn digits(&self) -> impl Iterator<Item = u8> {
        self.int.bytes().chain(self.frac.bytes())
    }

    fn order_of_magnitude(&self) -> i64 {
        to_i64(self.int.len() + self.frac.len()).saturating_add(self.exp)
    }

    /// Compares absolute values.
    fn cmp_magnitude(&self, other: &Self) -> Ordering {
        self.order_of_magnitude()
            .cmp(&other.order_of_magnitude())
            .then_with(|| self.digits().cmp(other.digits()))
    }
}

fn to_i64(n: usize) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

fn trim_leading_zeros<'a>(int: &'a str, frac: &'a str) -> (&'a str, &'a str) {
    let int = int.trim_start_matches('0');
    let frac = if int.is_empty() {
        frac.trim_start_matches('0')
    } else {
        frac
    };
    (int, frac)
}

fn trim_trailing_zeros<'a>(int: &'a str, frac: &'a str, exp: i64) -> (&'a str, &'a str, i64) {
    let trimmed = frac.trim_end_matches('0');
    let exp = exp.saturating_add(to_i64(frac.len() - trimmed.len()));
    if !trimmed.is_empty() {
        return (int, trimmed, exp);
    }
    let int_trimmed = int.trim_end_matches('0');
    let exp = exp.saturating_add(to_i64(int.len() - int_trimmed.len()));
    (int_trimmed, trimmed, exp)
}

fn parse_exponent(exponent: Option<&str>) -> i64 {
    let Some(text) = exponent else { return 0 };
    let (negative, digits) = match text.strip_prefix('-') {
        Some(d) => (true, d),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    digits.bytes().fold(0i64, |acc, b| {
        let digit = i64::from(b - b'0');
        let acc = acc.saturating_mul(10);
        if negative {
            acc.saturating_sub(digit)
        } else {
            acc.saturating_add(digit)
        }
    })
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
        self.sign
            .cmp(&other.sign)
            .then_with(|| self.sign.then_with(|| self.cmp_magnitude(other)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(text: &str) -> Decimal<'_> {
        let (mantissa, exponent) = match text.find(['e', 'E']) {
            Some(i) => (&text[..i], Some(&text[i + 1..])),
            None => (text, None),
        };
        Decimal::from_parts(mantissa, exponent)
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
        assert_eq!(dec(lhs), dec(rhs));
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
        assert_eq!(dec(text).is_integer(), expected);
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
        assert!(dec(smaller) < dec(larger));
        assert!(dec(larger) > dec(smaller));
    }

    #[test]
    fn huge_exponents() {
        let big = dec("1e99999999999999999999");
        let small = dec("1e-99999999999999999999");
        assert!(big.is_integer());
        assert!(!small.is_integer());
        assert!(small < dec("1"));
        assert!(dec("1") < big);
        assert!(small < big);
        assert!(dec("-1e99999999999999999999") < dec("-1"));
    }
}
