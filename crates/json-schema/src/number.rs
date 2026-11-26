use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Sign {
    Negative,
    Zero,
    Positive,
}

/// An exact JSON number: sign x digits(`int` ++ `frac`) x 10^`exp`, normalised without allocating.
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
        let (negative, unsigned) = match mantissa.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, mantissa),
        };
        let (int, frac) = unsigned.split_once('.').unwrap_or((unsigned, ""));
        let mut exp = parse_exponent(exponent).saturating_sub(to_i64(frac.len()));

        let mut int = int.trim_start_matches('0');
        let mut frac = frac;
        if int.is_empty() {
            frac = frac.trim_start_matches('0');
        }
        let trimmed = frac.trim_end_matches('0');
        exp = exp.saturating_add(to_i64(frac.len() - trimmed.len()));
        frac = trimmed;
        if frac.is_empty() {
            let trimmed = int.trim_end_matches('0');
            exp = exp.saturating_add(to_i64(int.len() - trimmed.len()));
            int = trimmed;
        }

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

    fn magnitude(&self) -> i128 {
        (self.int.len() + self.frac.len()) as i128 + i128::from(self.exp)
    }
}

fn to_i64(n: usize) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
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
        self.sign.cmp(&other.sign).then_with(|| {
            let magnitude = self
                .magnitude()
                .cmp(&other.magnitude())
                .then_with(|| self.digits().cmp(other.digits()));
            match self.sign {
                Sign::Zero => Ordering::Equal,
                Sign::Positive => magnitude,
                Sign::Negative => magnitude.reverse(),
            }
        })
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
