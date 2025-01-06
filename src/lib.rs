// (c) 2024 Ross Younger

#![doc = include_str!("../README.md")]
//!
//! # Feature flags
#![cfg_attr(
    feature = "document-features",
    cfg_attr(doc, doc = ::document_features::document_features!())
)]

use std::cmp::Ordering;

use num_traits::{checked_pow, ConstZero, PrimInt, ToPrimitive};

mod string;
pub use string::{DisplayAdapter, EngineeringRepr};

#[cfg(feature = "serde")]
mod serde_support;

/// Helper type for expressing numbers in engineering notation
///
/// # Type parameter
/// The type parameter `T` is the underlying storage type used for the significand of the number.
/// That is to say, an `EngineeringQuantity<u32>` uses a `u32` to store the numeric part.
#[derive(Debug, Clone, Copy, Default)]
pub struct EngineeringQuantity<T: EQSupported<T>> {
    /// Significant bits
    significand: T,
    /// Engineering exponent i.e. powers of 1e3
    exponent: i8,
}

/////////////////////////////////////////////////////////////////////////
// META (SUPPORTED STORAGE TYPES)

/// Marker trait indicating that a type is supported as a storage type for [`EngineeringQuantity`].
pub trait EQSupported<T: PrimInt>:
    PrimInt
    + std::fmt::Display
    + ConstZero
    + SignHelper<T>
    + TryInto<i64>
    + TryInto<i128>
    + TryInto<u64>
    + TryInto<u128>
{
    /// Always 1000 (used internally)
    const EXPONENT_BASE: T;
}

macro_rules! supported_types {
    {$($t:ty),+} => {$(
        impl<> EQSupported<$t> for $t {
            const EXPONENT_BASE: $t = 1000;
        }
    )+}
}

supported_types!(i16, i32, i64, i128, isize, u16, u32, u64, u128, usize);

/// Signedness helper data, used by string conversions
#[derive(Debug, Clone)]
pub struct AbsAndSign<T: PrimInt> {
    abs: T,
    negative: bool,
}

/// Signedness helper trait, used by string conversions.
///
/// This trait exists because `abs` is, quite reasonably, only implemented
/// for types which impl [`num_traits::Signed`].
pub trait SignHelper<T: PrimInt> {
    /// Unpacks a maybe-signed integer into its absolute value and sign bit
    fn abs_and_sign(&self) -> AbsAndSign<T>;
}

macro_rules! impl_unsigned_helpers {
    {$($t:ty),+} => {$(
        impl<> SignHelper<$t> for $t {
            fn abs_and_sign(&self) -> AbsAndSign<$t> {
                AbsAndSign { abs: *self, negative: false }
            }
        }
    )+}
}

macro_rules! impl_signed_helpers {
    {$($t:ty),+} => {$(
        impl<> SignHelper<$t> for $t {
            fn abs_and_sign(&self) -> AbsAndSign<$t> {
                AbsAndSign { abs: self.abs(), negative: self.is_negative() }
            }
        }
    )+}
}

impl_unsigned_helpers!(u16, u32, u64, u128, usize);
impl_signed_helpers!(i16, i32, i64, i128, isize);

/////////////////////////////////////////////////////////////////////////
// BASICS

// Constructors & accessors
impl<T: EQSupported<T>> EngineeringQuantity<T> {
    /// Raw constructor from component parts
    ///
    /// Construction fails if the number would overflow the storage type `T`.
    pub fn from_raw(significand: T, exponent: i8) -> Result<Self, Error> {
        Self::from_raw_unchecked(significand, exponent).check_for_int_overflow()
    }
    /// Raw accessor to retrieve the component parts
    #[must_use]
    pub fn to_raw(self) -> (T, i8) {
        (self.significand, self.exponent)
    }
    /// Internal raw constructor
    fn from_raw_unchecked(significand: T, exponent: i8) -> Self {
        Self {
            significand,
            exponent,
        }
    }
}

// Comparisons

impl<T: EQSupported<T> + From<EngineeringQuantity<T>>> PartialEq for EngineeringQuantity<T> {
    /// ```
    /// use engineering_repr::EngineeringQuantity as EQ;
    /// let q1 = EQ::from_raw(42u32,0);
    /// let q2 = EQ::from_raw(42u32,0);
    /// assert_eq!(q1, q2);
    /// let q3 = EQ::from_raw(42,1);
    /// let q4 = EQ::from_raw(42000,0);
    /// assert_eq!(q3, q4);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        // Easy case first
        if self.exponent == other.exponent {
            return self.significand == other.significand;
        }
        let cmp = self.partial_cmp(other);
        matches!(cmp, Some(Ordering::Equal))
    }
}

impl<T: EQSupported<T> + From<EngineeringQuantity<T>>> Eq for EngineeringQuantity<T> {}

impl<T: EQSupported<T> + From<EngineeringQuantity<T>>> PartialOrd for EngineeringQuantity<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: EQSupported<T> + From<EngineeringQuantity<T>>> Ord for EngineeringQuantity<T> {
    /// ```
    /// use engineering_repr::EngineeringQuantity as EQ;
    /// use more_asserts::assert_lt;
    /// let q2 = EQ::from_raw(41999,0).unwrap();
    /// let q3 = EQ::from_raw(42,1).unwrap();
    /// let q4 = EQ::from_raw(42001,0).unwrap();
    /// assert_lt!(q2, q3);
    /// assert_lt!(q3, q4);
    /// ```
    fn cmp(&self, other: &Self) -> Ordering {
        let v1 = <T as From<EngineeringQuantity<T>>>::from(*self);
        let v2 = <T as From<EngineeringQuantity<T>>>::from(*other);
        v1.cmp(&v2)
    }
}

// Type conversions
impl<T: EQSupported<T>> EngineeringQuantity<T> {
    /// Conversion to a different storage type.
    /// If you can convert from type A to type B,
    /// then you can convert from `EngineeringQuantity<A>` to `EngineeringQuantity<B>`.
    /// ```
    /// use engineering_repr::EngineeringQuantity as EQ;
    /// let q = EQ::from_raw(42u32, 0).unwrap();
    /// let q2 = q.convert::<u64>();
    /// assert_eq!(q2.to_raw(), (42u64, 0));
    /// ```
    pub fn convert<U: EQSupported<U> + From<T>>(&self) -> EngineeringQuantity<U> {
        let (sig, exp) = self.to_raw();
        EngineeringQuantity::<U>::from_raw_unchecked(sig.into(), exp)
    }

    /// Fallible conversion to a different storage type.
    ///
    /// Conversion fails if the number cannot be represented in the the destination storage type.
    /// ```
    /// type EQ = engineering_repr::EngineeringQuantity<u32>;
    /// let million = EQ::from_raw(1, 2).unwrap();
    /// let r1 = million.try_convert::<u32>().unwrap();
    /// let r2 = million.try_convert::<u16>().expect_err("overflow"); // Overflow, because 1_000_000 won't fit into a u16
    /// ```
    pub fn try_convert<U: EQSupported<U> + TryFrom<T>>(
        &self,
    ) -> Result<EngineeringQuantity<U>, Error> {
        let (sig, exp) = self.to_raw();
        EngineeringQuantity::<U>::from_raw(sig.try_into().map_err(|_| Error::Overflow)?, exp)
    }
}

impl<T: EQSupported<T>> EngineeringQuantity<T> {
    /// Scales the number to remove any unnecessary groups of trailing zeroes.
    #[must_use]
    pub fn normalise(self) -> Self {
        let mut working = self;
        loop {
            let (div, rem) = (
                working.significand / T::EXPONENT_BASE,
                working.significand % T::EXPONENT_BASE,
            );
            if rem != T::ZERO {
                break;
            }
            working.significand = div;
            working.exponent += 1;
        }
        working
    }
}

/////////////////////////////////////////////////////////////////////////
// CONVERSION FROM INTEGER

impl<T: EQSupported<T>, U: EQSupported<U>> From<T> for EngineeringQuantity<U>
where
    U: From<T>,
{
    /// Integers can always be promoted on conversion to [`EngineeringQuantity`].
    /// (For demotions, you have to convert the primitive yourself and handle any failures.)
    /// ```
    /// let i = 42u32;
    /// let _e = engineering_repr::EngineeringQuantity::<u64>::from(i);
    /// ```
    fn from(value: T) -> Self {
        Self {
            significand: value.into(),
            exponent: 0,
        }
    }
}

/////////////////////////////////////////////////////////////////////////
// CONVERSION TO INTEGER

impl<T: EQSupported<T>> EngineeringQuantity<T> {
    fn check_for_int_overflow(self) -> Result<Self, Error> {
        if self.exponent < 0 {
            // This function does NOT trap underflow.
            return Ok(self);
        }
        let exp: usize = self.exponent.unsigned_abs().into();
        let Some(factor) = checked_pow(T::EXPONENT_BASE, exp) else {
            return Err(Error::Overflow);
        };
        let result: T = factor
            .checked_mul(&self.significand)
            .ok_or(Error::Overflow)?;
        let _ = std::convert::TryInto::<T>::try_into(result).map_err(|_| Error::Overflow)?;
        Ok(self)
    }
}

macro_rules! impl_from {
    {$($t:ty),+} => {$(
        impl<T: EQSupported<T>> From<EngineeringQuantity<T>> for $t
        where $t: From<T>,
        {
            #[doc = concat!("\
Conversion to the same storage type (or a larger type)
is infallible due to the checks at construction time.
")]
            fn from(eq: EngineeringQuantity<T>) -> Self {
                let abs_exp: usize = eq.exponent.unsigned_abs().into();
                let factor: Self = num_traits::pow(T::EXPONENT_BASE.into(), abs_exp);
                if eq.exponent > 0 {
                    Self::from(eq.significand) * factor
                } else {
                    Self::from(eq.significand) / factor
                }
            }
        }

    )+}
}

impl_from!(u16, u32, u64, u128, usize, i16, i32, i64, i128, isize);

impl<T: EQSupported<T>> EngineeringQuantity<T> {
    fn apply_factor<U: EQSupported<U>>(self, sig: U) -> U {
        let abs_exp: usize = self.exponent.unsigned_abs().into();
        let factor: U = num_traits::pow(U::EXPONENT_BASE, abs_exp);
        if self.exponent >= 0 {
            sig * factor
        } else {
            sig / factor
        }
    }
}

impl<T: EQSupported<T>> ToPrimitive for EngineeringQuantity<T> {
    /// ```
    /// use num_traits::cast::ToPrimitive as _;
    /// let e = engineering_repr::EngineeringQuantity::<u32>::from(65_537u32);
    /// assert_eq!(e.to_u128(), Some(65_537));
    /// assert_eq!(e.to_u64(), Some(65_537));
    /// assert_eq!(e.to_u16(), None); // overflow
    /// assert_eq!(e.to_i128(), Some(65_537));
    /// assert_eq!(e.to_i64(), Some(65_537));
    /// assert_eq!(e.to_i16(), None); // overflow
    /// ```
    fn to_i64(&self) -> Option<i64> {
        let i: i64 = match self.significand.try_into() {
            Ok(ii) => ii,
            Err(_) => return None,
        };
        Some(self.apply_factor(i))
    }

    fn to_u64(&self) -> Option<u64> {
        let i: u64 = match self.significand.try_into() {
            Ok(ii) => ii,
            Err(_) => return None,
        };
        Some(self.apply_factor(i))
    }

    fn to_i128(&self) -> Option<i128> {
        let i: i128 = match self.significand.try_into() {
            Ok(ii) => ii,
            Err(_) => return None,
        };
        Some(self.apply_factor(i))
    }

    fn to_u128(&self) -> Option<u128> {
        let i: u128 = match self.significand.try_into() {
            Ok(ii) => ii,
            Err(_) => return None,
        };
        Some(self.apply_factor(i))
    }
}

/////////////////////////////////////////////////////////////////////////
// ERRORS

/// Local error type returned by failing conversions
#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
#[allow(missing_docs)]
pub enum Error {
    #[error("Numeric overflow")]
    Overflow,
    #[error("Numeric underflow")]
    Underflow,
    #[error("The string could not be parsed")]
    ParseError,
}

/////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::EngineeringQuantity as EQ;
    use super::Error as EQErr;

    #[test]
    fn integers() {
        for i in &[1i64, -1, 100, -100, 1000, 4000, -4000, 4_000_000] {
            let ee = EQ::from_raw(*i, 0).unwrap();
            assert_eq!(i64::from(ee), *i);
            let ee2 = EQ::from_raw(*i, 1).unwrap();
            assert_eq!(i64::from(ee2), *i * 1000, "input is {}", *i);
        }
    }

    #[test]
    fn equality() {
        for (a, b, c, d) in &[
            (1i64, 0, 1i64, 0),
            (1, 1, 1000, 0),
            (2000, 0, 2, 1),
            (123_000_000, 0, 123_000, 1),
            (123_000_000, 0, 123, 2),
            (456_000_000_000_000, 0, 456_000, 3),
            (456_000_000_000_000, 0, 456, 4),
        ] {
            let e1 = EQ::from_raw(*a, *b);
            let e2 = EQ::from_raw(*c, *d);
            assert_eq!(e1, e2);
        }
    }

    #[test]
    fn conversion() {
        let t = EQ::<u32>::from_raw(12345, 0).unwrap();
        let u = t.convert::<u64>();
        assert_eq!(u.to_raw().0, <u32 as Into<u64>>::into(t.to_raw().0));
        assert_eq!(t.to_raw().1, u.to_raw().1);
    }

    #[test]
    fn overflow() {
        // When the number is too big to fit into the destination type, the conversion fails.
        let t = EQ::<u32>::from_raw(100_000, 0).unwrap();
        let _ = t.try_convert::<u16>().expect_err("TryFromIntError");

        // 10^15 is too big for a u32, so overflow:
        assert_eq!(EQ::<u32>::from_raw(1, 5), Err(EQErr::Overflow));

        // The significand and exponent may both fit on their own, but overflow when combined:
        assert_eq!(EQ::<u64>::from_raw(100_000, 5), Err(EQErr::Overflow));
    }

    #[test]
    fn normalise() {
        let q = EQ::from_raw(1_000_000, 0).unwrap();
        let q2 = q.normalise();
        assert_eq!(q, q2);
        assert_eq!(q2.to_raw(), (1, 2));
    }

    #[test]
    fn to_primitive() {
        use num_traits::ToPrimitive as _;
        let e = EQ::<i128>::from_raw(1234, 0).unwrap();
        assert_eq!(e.to_i8(), None);
        assert_eq!(e.to_i16(), Some(1234));
        assert_eq!(e.to_i32(), Some(1234));
        assert_eq!(e.to_i64(), Some(1234));
        assert_eq!(e.to_i128(), Some(1234));
        assert_eq!(e.to_isize(), Some(1234));
        assert_eq!(e.to_u8(), None);
        assert_eq!(e.to_u16(), Some(1234));
        assert_eq!(e.to_u32(), Some(1234));
        assert_eq!(e.to_u64(), Some(1234));
        assert_eq!(e.to_u128(), Some(1234));
        assert_eq!(e.to_usize(), Some(1234));

        // negatives cannot fit into an unsigned
        let e = EQ::<i128>::from_raw(-1, 0).unwrap();
        assert_eq!(e.to_u64(), None);
        assert_eq!(e.to_u128(), None);

        // positives which would overflow
        let e = EQ::<u128>::from_raw(u128::MAX, 0).unwrap();
        assert_eq!(e.to_i64(), None);
        assert_eq!(e.to_i128(), None);

        // rounding toward zero
        let e = EQ::from_raw(1, -1).unwrap();
        assert_eq!(e.to_i32(), Some(0));
        let e = EQ::from_raw(1001, -1).unwrap();
        assert_eq!(e.to_i32(), Some(1));
        let e = EQ::from_raw(-1, -1).unwrap();
        assert_eq!(e.to_i32(), Some(0));
        let e = EQ::from_raw(-1001, -1).unwrap();
        assert_eq!(e.to_i32(), Some(-1));
    }
}
