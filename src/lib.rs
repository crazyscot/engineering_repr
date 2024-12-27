use std::{cmp::min, fmt::Display, str::FromStr};

const MAX_EXPONENT_U128: u32 = 12;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct EngineeringExponential {
    significand: i128,
    /// Exponent in 10^3 i.e. 0 => 1, 1 => 1000, 2 => 10^6, etc.
    exponent_1e3: u32,
}

impl EngineeringExponential {
    /// Standard checking constructor
    ///
    /// # Panics
    /// If the exponent is guaranteed to overflow
    #[must_use]
    pub fn new(significand: i128, exponent_1e3: u32) -> Self {
        assert!(exponent_1e3 <= MAX_EXPONENT_U128, "exponent would overflow");
        Self {
            significand,
            exponent_1e3,
        }
    }
    #[must_use]
    pub fn contents(self) -> (i128, u32) {
        (self.significand, self.exponent_1e3)
    }
}

/////////////////////////////////////////////////////////////////////////
// CONVERSION FROM INTEGER

macro_rules! impl_from_int {
    {$($t:ty),+} => {$(
        impl From<$t> for EngineeringExponential {
            fn from(value: $t) -> Self {
                Self::new(value.into(), 0)
            }
        }
    )+}
}

impl_from_int!(u8, u16, u32, u64, i8, i16, i32, i64, i128);

impl TryFrom<u128> for EngineeringExponential {
    type Error = EEError;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        let v = i128::try_from(value).map_err(|_| EEError::Overflow)?;
        Ok(Self::new(v, 1))
    }
}

/////////////////////////////////////////////////////////////////////////
// CONVERSION TO INTEGER

macro_rules! impl_into_int {
    {$($t:ty),+} => {$(
        impl TryFrom<EngineeringExponential> for $t {
            type Error = EEError;
            fn try_from(value: EngineeringExponential) -> Result<Self, Self::Error> {
                let Some(mult) = 1000i128.checked_pow(value.exponent_1e3) else { return Err(EEError::Overflow) };
                (mult * value.significand)
                    .try_into()
                    .map_err(|_| EEError::Overflow)
            }
        }
    )+}
}

impl_into_int!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

impl EngineeringExponential {
    #[must_use]
    /// Converting accessor
    /// # Panics
    /// If the resultant value would be too big for an i128
    pub fn value(&self) -> i128 {
        i128::try_from(*self).unwrap()
    }
}

/////////////////////////////////////////////////////////////////////////
// SI MULTIPLIERS

static SI_MULTIPLIERS: &str = " kMGTPEZYRQ";
lazy_static::lazy_static! {
    static ref SI_MULTIPLIERS_STRING: String = String::from(&SI_MULTIPLIERS[1..]);
}

fn exponent_to_multiplier(exp_1e3: usize) -> &'static str {
    if exp_1e3 == 0 {
        return "";
    }
    &SI_MULTIPLIERS[exp_1e3..=exp_1e3]
}

const fn multiplier_to_exponent(prefix: char) -> Option<u32> {
    Some(match prefix {
        //' ' => 0,
        'k' => 1,
        'M' => 2,
        'G' => 3,
        'T' => 4,
        'P' => 5,
        'E' => 6,
        'Z' => 7,
        'Y' => 8,
        'R' => 9,
        'Q' => 10,
        _ => return None,
    })
}

fn find_multiplier(s: &str) -> Option<(usize /* index */, u32 /* exponent */)> {
    for (i, c) in s.chars().enumerate() {
        if let Some(p) = multiplier_to_exponent(c) {
            return Some((i, p));
        }
    }
    None
}

/////////////////////////////////////////////////////////////////////////
// STRING TO NUMBER

impl FromStr for EngineeringExponential {
    type Err = EEError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let prefix = find_multiplier(s);
        let Some((prefix_index, exponent_1e3)) = prefix else {
            // Easy case: direct integer conversion.
            // There had better not be a decimal point as that would imply a non-integer!
            return i128::from_str(s)
                .map(|i| EngineeringExponential::new(i, 0))
                .map_err(|_| EEError::ParseError);
        };

        // Is there a decimal? If so it's RKM mode.
        let decimal = s.find('.');

        let split_index = if let Some(d) = decimal {
            // Non-RKM mode (1.5k)
            d
        } else {
            // RKM mode (1k5)
            prefix_index
        };

        let mut to_convert = String::from(&s[0..split_index]);
        let trailing = &s[split_index + 1..];
        // In non-RKM mode, don't convert the prefix (err, the suffix)
        let trailing = if decimal.is_some() {
            &trailing[0..trailing.len() - 1]
        } else {
            trailing
        };
        // Each 3 digits (or part thereof) represents another exponent.
        to_convert.push_str(trailing);
        // If it's not a round multiple of 3, we need to pad !
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let whole_groups = (trailing.len() / 3) as i32;
        // convert to signed so we can trap a panic
        #[allow(clippy::cast_possible_wrap)]
        let mut exponent_1e3 = exponent_1e3 as i32;
        match trailing.len() % 3 {
            0 => {
                exponent_1e3 -= whole_groups;
            }
            1 => {
                exponent_1e3 -= whole_groups + 1;
                to_convert.push_str("00");
            }
            2 => {
                exponent_1e3 -= whole_groups + 1;
                to_convert.push('0');
            }
            3.. => panic!("impossible"),
        }
        if exponent_1e3 < 0 {
            return Err(EEError::ParseError);
        }

        let significand = i128::from_str(&to_convert).map_err(|_| EEError::ParseError)?;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap,
            clippy::cast_sign_loss
        )]
        Ok(Self::new(significand, exponent_1e3 as u32))
    }
}

/////////////////////////////////////////////////////////////////////////
// NUMBER TO STRING

impl Display for EngineeringExponential {
    /// Standard precision is defined as 3 significant figures, standard (not RKM) mode.
    /// See [`EngineeringExponential::default()`].
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let d = DisplayAdapter {
            value: self,
            ..Default::default()
        };

        d.fmt(f)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DisplayAdapter<'a> {
    value: &'a EngineeringExponential,
    max_significant_figures: usize,
    rkm: bool,
}

static DUMMY_VALUE: EngineeringExponential = EngineeringExponential {
    significand: 0,
    exponent_1e3: 0,
};

impl Default for DisplayAdapter<'_> {
    fn default() -> Self {
        Self {
            value: &DUMMY_VALUE,
            max_significant_figures: 3,
            rkm: false,
        }
    }
}

impl EngineeringExponential {
    #[must_use]
    pub fn with_precision(&self, max_sf: usize) -> DisplayAdapter<'_> {
        DisplayAdapter {
            value: self,
            max_significant_figures: max_sf,
            rkm: false,
        }
    }
    #[must_use]
    pub fn rkm_with_precision(&self, max_sf: usize) -> DisplayAdapter<'_> {
        DisplayAdapter {
            value: self,
            max_significant_figures: max_sf,
            rkm: true,
        }
    }
}

impl Display for DisplayAdapter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut digits = self.value.significand.abs().to_string();
        // at first glance the output might reasonably be this value of `digits`, followed by `exponent` times "000"...
        // but we need to (re)compute the correct exponent for display.
        let minus = if self.value.significand < 0 { "-" } else { "" };

        digits.reserve((3 * self.value.exponent_1e3 + 1) as usize);
        for _ in 1..self.value.exponent_1e3 {
            digits.push_str("000");
        }
        let output_exponent = (digits.len() - 1) / 3;
        let si = exponent_to_multiplier(output_exponent);
        let leading = digits.len() - output_exponent * 3;
        let trailing = min(
            digits.len() - leading,
            self.max_significant_figures - min(self.max_significant_figures, leading),
        );
        let leaders = &digits[0..leading];
        let trailers = &digits[leading..leading + min(trailing, self.max_significant_figures)];
        let mid = if self.rkm {
            si
        } else if self.max_significant_figures == 0 || trailers.is_empty() {
            ""
        } else {
            "."
        };
        let suffix = if self.rkm { "" } else { si };
        write!(f, "{minus}{leaders}{mid}{trailers}{suffix}")
    }
}

/////////////////////////////////////////////////////////////////////////
// ERRORS

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EEError {
    Overflow,
    Underflow,
    ParseError,
}
/////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::EngineeringExponential as EE;

    #[test]
    fn integers() {
        for i in &[1, -1, 100, -100, 1000, 4000, -4000, 4_000_000] {
            let ee = EE::new(*i, 0);
            assert_eq!(ee.value(), *i);
            let ee2 = EE::new(*i, 1);
            assert_eq!(ee2.value(), *i * 1000, "input is {}", *i);
        }
    }
    #[test]
    fn to_string() {
        for (i, s) in &[
            (1i128, "1"),
            (42, "42"),
            (999, "999"),
            (1000, "1.00k"),
            (1500, "1.50k"),
            (2345, "2.34k"),
            (9999, "9.99k"),
            (12_345, "12.3k"),
            (13_000, "13.0k"),
            (999_999, "999k"),
            (1_000_000, "1.00M"),
            (2_345_678, "2.34M"),
            (999_999_999, "999M"),
            (12_345_000_000_000_000_000_000_000_000, "12.3R"),
            (12_345_000_000_000_000_000_000_000_000_000, "12.3Q"),
        ] {
            let ee = EE::from(*i);
            assert_eq!(ee.to_string(), *s);
            let ee2 = EE::from(-*i);
            let ss2 = ee2.to_string();
            assert_eq!(ss2.chars().next().unwrap(), '-');
            assert_eq!(&ss2[1..], *s);
        }
    }
    #[test]
    fn to_string_rkm() {
        for (i, s) in &[
            (1i128, "1"),
            (42, "42"),
            (999, "999"),
            (1000, "1k0"),
            (1500, "1k5"),
            (2345, "2k3"),
            (9999, "9k9"),
            (12_345, "12k"),
            (13_000, "13k"),
            (999_999, "999k"),
            (1_000_000, "1M0"),
            (2_345_678, "2M3"),
            (999_999_999, "999M"),
            (12_345_000_000_000_000_000_000_000_000, "12R"),
            (12_345_000_000_000_000_000_000_000_000_000, "12Q"),
        ] {
            let ee = EE::from(*i);
            assert_eq!(ee.rkm_with_precision(2).to_string(), *s);
            let ee2 = EE::from(-*i);
            let ss2 = ee2.rkm_with_precision(2).to_string();
            assert_eq!(ss2.chars().next().unwrap(), '-');
            assert_eq!(&ss2[1..], *s);
        }
    }

    #[test]
    fn from_string() {
        for (i, s) in &[
            (1i128, "1"),
            (42, "42"),
            (999, "999"),
            (1000, "1k"),
            (1500, "1.5k"),
            (2345, "2.345k"),
            (9999, "9.999k"),
            (12_345, "12.345k"),
            (13_000, "13k"),
            (13_000, "13.k"),
            (13_000, "13.0k"),
            (999_999, "999.999k"),
            (1_000_000, "1.00M"),
            (2_345_678, "2.345678M"),
            (999_999_999, "999.999999M"),
            (12_345_000_000_000_000_000_000_000_000, "12.345R"),
            (12_345_000_000_000_000_000_000_000_000_000, "12.345Q"),
            (1000, "1k0"),
            (1500, "1k5"),
            (2345, "2k345"),
            (9999, "9k999"),
            (12_345, "12k345"),
            (13_000, "13k0"),
            (999_999, "999k999"),
            (1_000_000, "1M0"),
            (2_345_678, "2M345678"),
            (999_999_999, "999M999999"),
            (12_345_000_000_000_000_000_000_000_000, "12R345"), // I wonder if 1R means 1 ohm or 1 ronnaohm? :-)
            (12_345_000_000_000_000_000_000_000_000_000, "12Q345"),
        ] {
            let ee = EE::from_str(s).unwrap();
            assert_eq!(ee.value(), *i, "input {s} expected {i}");
            let mut str2 = String::with_capacity(1 + s.len());
            str2.push('-');
            str2.push_str(s);
            let ee2 = EE::from_str(&str2).unwrap();
            assert_eq!(ee2.value(), -*i);
        }
    }
    #[test]
    fn parse_failures() {
        for s in &["foo", "1.2", "1.2.3k", "1.2345k", "--1"] {
            let _ = EE::from_str(s).expect_err(&format!("this should have failed: {s}"));
        }
    }
        }
    }
}
