//! String conversions

use std::{cmp::min, fmt::Display, str::FromStr};

use crate::{EQSupported, EngineeringQuantity, Error};

static POSITIVE_MULTIPLIERS: &str = " kMGTPEZYRQ";

fn exponent_to_multiplier(exp: usize) -> &'static str {
    if exp == 0 {
        return "";
    }
    &POSITIVE_MULTIPLIERS[exp..=exp]
}

const fn multiplier_to_exponent(prefix: char) -> Option<i8> {
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

fn find_multiplier(s: &str) -> Option<(usize /* index */, i8 /* exponent */)> {
    for (i, c) in s.chars().enumerate() {
        if let Some(p) = multiplier_to_exponent(c) {
            return Some((i, p));
        }
    }
    None
}

/////////////////////////////////////////////////////////////////////////
// STRING TO NUMBER

impl<T: EQSupported<T> + FromStr> FromStr for EngineeringQuantity<T> {
    type Err = Error;

    /// # Example
    /// ```
    /// use engineering_repr::EngineeringQuantity as EQ;
    /// use std::str::FromStr as _;
    /// let eq = EQ::<i64>::from_str("1.5k").unwrap();
    /// assert_eq!(i64::try_from(eq).unwrap(), 1500);
    /// // RKM style strings
    /// let eq2 = EQ::<i64>::from_str("1k5").unwrap();
    /// assert_eq!(eq, eq2);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let prefix = find_multiplier(s);
        let Some((prefix_index, exponent)) = prefix else {
            // Easy case: direct integer conversion.
            // There had better not be a decimal point as that would imply a non-integer!
            return T::from_str(s)
                .map(|i| EngineeringQuantity::from_raw(i, 0))
                .map_err(|_| Error::ParseError);
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
        let whole_groups = (trailing.len() / 3) as i8;
        // convert to signed so we can trap a panic
        #[allow(clippy::cast_possible_wrap)]
        let mut exponent = exponent;
        match trailing.len() % 3 {
            0 => {
                exponent -= whole_groups;
            }
            1 => {
                exponent -= whole_groups + 1;
                to_convert.push_str("00");
            }
            2 => {
                exponent -= whole_groups + 1;
                to_convert.push('0');
            }
            3.. => panic!("impossible"), // coverage cannot reach this line
        }
        if exponent < 0 {
            return Err(Error::ParseError);
        }

        let significand = T::from_str(&to_convert).map_err(|_| Error::ParseError)?;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap,
            clippy::cast_sign_loss
        )]
        Ok(Self::from_raw(significand, exponent))
    }
}

/////////////////////////////////////////////////////////////////////////
// NUMBER TO STRING

impl<T: EQSupported<T>> Display for EngineeringQuantity<T> {
    /// Default behaviour is to output to 3 significant figures, standard (not RKM) mode.
    /// See [`EngineeringQuantity::default()`].
    /// # Examples
    /// ```
    /// use engineering_repr::EngineeringQuantity as EQ;
    /// let ee1 = EQ::<i32>::from(1200);
    /// assert_eq!(ee1.to_string(), "1.20k");
    /// let ee2 = EQ::<i32>::from(123456);
    /// assert_eq!(ee2.to_string(), "123k");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayAdapter {
            value: *self,
            ..Default::default()
        }
        .fmt(f)
    }
}

/// A wrapper type which allows you to specify the desired output format.
/// It implements [`Display`].
///
/// This type may be conveniently created by [`EngineeringQuantity::with_precision()`]
/// and [`EngineeringQuantity::rkm_with_precision()`].
#[derive(Copy, Clone, Debug)]
pub struct DisplayAdapter<T: EQSupported<T>>
where
    T: ToString,
{
    /// The value to be displayed
    pub value: EngineeringQuantity<T>,
    /// The precision at which to display, or 0 to work it out losslessly
    pub max_significant_figures: usize,
    /// Specifies [RKM code](https://en.wikipedia.org/wiki/RKM_code) mode
    pub rkm: bool,
}

impl<T: EQSupported<T>> Default for DisplayAdapter<T> {
    fn default() -> Self {
        Self {
            value: EngineeringQuantity {
                significand: T::ZERO,
                exponent: 0,
            },
            max_significant_figures: 3,
            rkm: false,
        }
    }
}

impl<T: EQSupported<T>> PartialEq<DisplayAdapter<T>> for &str {
    /// This is intended for use in tests.
    #[allow(clippy::cmp_owned)]
    fn eq(&self, other: &DisplayAdapter<T>) -> bool {
        *self == other.to_string()
    }
}

impl<T: EQSupported<T>> EngineeringQuantity<T> {
    /// Creates a standard [`DisplayAdapter`] for this object, with the given precision.
    /// ```
    /// use engineering_repr::EngineeringQuantity as EQ;
    /// let ee = EQ::<i32>::from(1234567);
    /// assert_eq!(ee.with_precision(2).to_string(), "1.2M");
    /// ```
    #[must_use]
    pub fn with_precision(&self, max_significant_figures: usize) -> DisplayAdapter<T> {
        DisplayAdapter {
            value: *self,
            max_significant_figures,
            rkm: false,
        }
    }
    /// Creates an RKM [`DisplayAdapter`] for this object in RKM mode, with the given precision.
    /// ```
    /// use engineering_repr::EngineeringQuantity as EQ;
    /// let ee = EQ::<i32>::from(1234567);
    /// assert_eq!(ee.rkm_with_precision(2).to_string(), "1M2");
    /// ```
    #[must_use]
    pub fn rkm_with_precision(&self, max_significant_figures: usize) -> DisplayAdapter<T> {
        DisplayAdapter {
            value: *self,
            max_significant_figures,
            rkm: true,
        }
    }
}

impl<T: EQSupported<T>> Display for DisplayAdapter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let detail = self.value.significand.abs_and_sign();
        let mut digits = detail.abs.to_string();
        // at first glance the output might reasonably be this value of `digits`, followed by `exponent` times "000"...
        // but we need to (re)compute the correct exponent for display.
        let prefix = if detail.negative { "-" } else { "" };
        digits.reserve((3 * self.value.exponent + 1).unsigned_abs() as usize);
        if self.value.exponent < 0 {
            return write!(f, "(underflow error! negative exponents not supported)");
        }
        for _ in 0..self.value.exponent {
            digits.push_str("000");
        }
        let output_exponent = (digits.len() - 1) / 3;
        let si = exponent_to_multiplier(output_exponent);
        let leading = digits.len() - output_exponent * 3;

        let precision = match self.max_significant_figures {
            0 => usize::MAX, // automatic mode: take all the digits, we'll trim trailing 0s in a moment
            i => i,
        };
        let trailing = min(
            // number of digits remaining
            digits.len() - leading,
            // number of digits we'd take to reach the requested number of s.f.
            precision - min(precision, leading),
        );
        let leaders = &digits[0..leading];
        let mut trailers = &digits[leading..leading + min(trailing, precision)];
        if precision == usize::MAX {
            while trailers.ends_with('0') {
                trailers = &trailers[0..trailers.len() - 1];
            }
        }
        let mid = if self.rkm {
            si
        } else if trailers.is_empty() {
            ""
        } else {
            "."
        };
        let suffix = if self.rkm { "" } else { si };
        write!(f, "{prefix}{leaders}{mid}{trailers}{suffix}")
    }
}

/////////////////////////////////////////////////////////////////////////
// CONVENIENCE TRAITS

/// A convenience trait for outputting integers directly in engineering notation.
///
/// [`DisplayAdapter`] implements [`Display`], so you can use the returned adapter
/// directly in a formatting macro.
pub trait EngineeringRepr<T: EQSupported<T>> {
    /// Outputs a number in engineering notation
    ///
    /// A request for 0 significant figures outputs exactly as many digits are necessary to maintain precision.
    /// ```
    /// use engineering_repr::EngineeringRepr as _;
    /// assert_eq!("123k", 123456.to_eng(3));
    /// assert_eq!("123.4k", 123456.to_eng(4));
    /// assert_eq!("123.456k", 123456.to_eng(0));
    /// ```
    /// # Panics
    /// If the value could not be rendered
    fn to_eng(self, sig_figures: usize) -> DisplayAdapter<T>;

    /// Outputs a number in RKM notation
    ///
    /// A request for 0 significant figures outputs exactly as many digits are necessary to maintain precision.
    /// ```
    /// use engineering_repr::EngineeringRepr as _;
    /// assert_eq!("123k", 123456.to_rkm(3));
    /// assert_eq!("123k4", 123456.to_rkm(4));
    /// assert_eq!("123k456", 123456.to_rkm(0));
    /// ```
    /// # Panics
    /// If the value could not be rendered
    fn to_rkm(self, sig_figures: usize) -> DisplayAdapter<T>;
}

macro_rules! impl_to_eng {
    {$($t:ty),+} => {$(
        impl<> EngineeringRepr<$t> for $t {
            fn to_eng(self, sig_figures: usize) -> DisplayAdapter<$t>
            {
                EngineeringQuantity::<$t>::try_from(self).unwrap().with_precision(sig_figures)
            }
            fn to_rkm(self, sig_figures: usize) -> DisplayAdapter<$t>
            {
                EngineeringQuantity::<$t>::try_from(self).unwrap().rkm_with_precision(sig_figures)
            }
        }
    )+}
}

impl_to_eng!(u16, u32, u64, u128, usize, i16, i32, i64, i128, isize);

/////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::EngineeringQuantity as EQ;
    use std::str::FromStr as _;

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
            (1_000_000_000, "1G0"),
            (1_000_000_000_000, "1T0"),
            (1_000_000_000_000_000, "1P0"),
            (1_000_000_000_000_000_000, "1E0"),
            (1_000_000_000_000_000_000_000, "1Z0"),
            (1_000_000_000_000_000_000_000_000, "1Y0"),
            (12_345_000_000_000_000_000_000_000_000, "12R345"), // I wonder if 1R means 1 ohm or 1 ronnaohm? :-)
            (12_345_000_000_000_000_000_000_000_000_000, "12Q345"),
        ] {
            let eq = EQ::<i128>::from_str(s).unwrap();
            let result = i128::try_from(eq).unwrap();
            assert_eq!(result, *i, "input {s} expected {i}");
            let mut str2 = String::with_capacity(1 + s.len());
            str2.push('-');
            str2.push_str(s);
            let ee2 = EQ::<i128>::from_str(&str2).unwrap();
            assert_eq!(i128::try_from(ee2).unwrap(), -*i);
        }
    }

    #[test]
    fn parse_failures() {
        for s in &["foo", "1.2", "1.2.3k", "1.2345k", "--1"] {
            let _ = EQ::<i128>::from_str(s).expect_err(&format!("this should have failed: {s}"));
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
            let ee = EQ::<i128>::from(*i);
            assert_eq!(ee.to_string(), *s);
            let ee2 = EQ::<i128>::from(-*i);
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
            let ee = EQ::<i128>::from(*i);
            assert_eq!(ee.rkm_with_precision(2).to_string(), *s);
            let ee2 = EQ::<i128>::from(-*i);
            let ss2 = ee2.rkm_with_precision(2).to_string();
            assert_eq!(ss2.chars().next().unwrap(), '-');
            assert_eq!(&ss2[1..], *s);
        }
    }

    #[test]
    fn traits() {
        use super::EngineeringRepr as _;
        assert_eq!("123k", 123_456.to_eng(3));
        assert_eq!("123.4k", 123_456.to_eng(4));
        assert_eq!("123k4", 123_456.to_rkm(4));
    }

    #[test]
    fn raw_to_string() {
        for (sig, exp, str) in &[
            (1, 0i8, "1"),
            (1, 1, "1.00k"),
            (1000, 0, "1.00k"),
            (1000, 1, "1.00M"),
        ] {
            let e = EQ::<i128>::from_raw(*sig, *exp);
            assert_eq!(e.to_string(), *str, "test case: {sig},{exp} -> {str}");
        }
    }

    #[test]
    fn overflow() {
        let e = EQ::from_raw(1u16, 0);
        let e2 = EQ::from_raw(1u16, 1);
        assert_ne!(e, e2);
        println!("{e:?} -> {e}");
        println!("{e2:?} -> {e2}");
        let _ = e.to_string();
    }
    #[test]
    fn underflow() {
        let e = EQ::from_raw(1u16, 0);
        let e2 = EQ::from_raw(1u16, -1);
        assert_ne!(e, e2);
        assert!(e2.to_string().contains("underflow"));
    }

    #[test]
    fn auto_precision() {
        for (i, s) in &[
            (1i128, "1"),
            (42, "42"),
            (100, "100"),
            (999, "999"),
            (1000, "1k"),
            (1500, "1.5k"),
            (2345, "2.345k"),
            (9999, "9.999k"),
            (12_345, "12.345k"),
            (13_000, "13k"),
            (999_999, "999.999k"),
            (1_000_000, "1M"),
            (2_345_678, "2.345678M"),
            (999_999_999, "999.999999M"),
            (12_345_600_000_000_000_000_000_000_000, "12.3456R"),
            (12_345_600_000_000_000_000_000_000_000_000, "12.3456Q"),
        ] {
            let ee = EQ::<i128>::from(*i);
            assert_eq!(ee.with_precision(0).to_string(), *s, "input={}", *i);
            let ee2 = EQ::<i128>::from(-*i);
            let ss2 = ee2.with_precision(0).to_string();
            assert_eq!(ss2.chars().next().unwrap(), '-');
            assert_eq!(&ss2[1..], *s, "input={}", -*i);
        }
    }
}
