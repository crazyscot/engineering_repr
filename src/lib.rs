// (c) 2024 Ross Younger

#![doc = include_str!("../README.md")]

use std::cmp::Ordering;

use num_traits::{checked_pow, ConstZero, PrimInt};

mod string;
pub use string::{DisplayAdapter, EngineeringRepr};

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
pub trait EQSupported<T: PrimInt>: PrimInt + std::fmt::Display + ConstZero + SignHelper<T> {
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
/// This trait exists because `abs()` is, quite reasonably, only implemented
/// for types which are `num_traits::Signed`.
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
    #[must_use]
    pub fn from_raw(significand: T, exponent: i8) -> Self {
        Self {
            significand,
            exponent,
        }
    }
    /// Raw accessor to retrieve the component parts
    #[must_use]
    pub fn to_raw(self) -> (T, i8) {
        (self.significand, self.exponent)
    }
}

// Comparisons

impl<T: EQSupported<T> + TryFrom<EngineeringQuantity<T>>> PartialEq for EngineeringQuantity<T> {
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

impl<T: EQSupported<T> + TryFrom<EngineeringQuantity<T>>> PartialOrd for EngineeringQuantity<T> {
    /// ```
    /// use engineering_repr::EngineeringQuantity as EQ;
    /// use more_asserts::assert_lt;
    /// let q2 = EQ::from_raw(41999,0);
    /// let q3 = EQ::from_raw(42,1);
    /// let q4 = EQ::from_raw(42001,0);
    /// assert_lt!(q2, q3);
    /// assert_lt!(q3, q4);
    /// ```
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let v1 = T::try_from(*self);
        let v2 = T::try_from(*other);
        match (v1, v2) {
            (Ok(vv1), Ok(vv2)) => Some(vv1.cmp(&vv2)),
            _ => None,
        }
    }
}

// Type conversions
impl<T: EQSupported<T>> EngineeringQuantity<T> {
    /// Conversion to a different storage type.
    /// If you can convert from type A to type B,
    /// then you can convert from `EngineeringQuantity<A>` to `EngineeringQuantity<B>`.
    /// ```
    /// use engineering_repr::EngineeringQuantity as EQ;
    /// let q = EQ::from_raw(42u32, 0);
    /// let q2 = q.convert::<u64>();
    /// assert_eq!(q2.to_raw(), (42u64, 0));
    /// ```
    pub fn convert<U: EQSupported<U> + From<T>>(&self) -> EngineeringQuantity<U> {
        let (sig, exp) = self.to_raw();
        EngineeringQuantity::<U>::from_raw(sig.into(), exp)
    }

    /// Fallible conversion to a different storage type.
    ///
    /// Note that conversion only fails if the significand doesn't fit into the destination storage type,
    /// without reference to the exponent. This means that two numbers, which might be equal, may not both
    /// be convertible to the same destination type if they are not normalised. For example:
    /// ```
    /// use engineering_repr::EngineeringQuantity as EQ;
    /// let million1 = EQ::from_raw(1, 2); // 1e6
    /// let million2 = EQ::from_raw(1_000_000, 0);
    /// assert_eq!(million1, million2);
    /// let r1 = million1.try_convert::<u16>().unwrap(); // OK, because stored as (1,2)
    /// let r2 = million2.try_convert::<u16>().expect_err("overflow"); // Overflow, because 1_000_000 won't fit into a u16
    /// ```
    pub fn try_convert<U: EQSupported<U> + TryFrom<T>>(
        &self,
    ) -> Result<EngineeringQuantity<U>, <U as std::convert::TryFrom<T>>::Error> {
        let (sig, exp) = self.to_raw();
        Ok(EngineeringQuantity::<U>::from_raw(sig.try_into()?, exp))
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

macro_rules! impl_try_from {
    {$($t:ty),+} => {$(
        impl<U: EQSupported<U>> TryFrom<EngineeringQuantity<U>> for $t
        where $t: TryFrom<U>,
        {
            type Error = crate::Error;
            #[doc = concat!("\
Conversion to integer is always fallible, as the exponent might cause us to under or overflow.
```
use engineering_repr::EngineeringQuantity;
use engineering_repr::Error as EErr;
let i = EngineeringQuantity::<u32>::from_raw(11, 1);
assert_eq!(", stringify!($t), "::try_from(i).unwrap(), 11000);
```
")]
            fn try_from(eq: EngineeringQuantity<U>) -> Result<Self, Error> {
                // TODO: This conversion fails on negative exponents
                let exp: usize = eq.exponent.try_into().map_err(|_| Error::Underflow)?;
                let Some(factor) = checked_pow(U::EXPONENT_BASE, exp) else {
                    return Err(Error::Overflow);
                };
                let result: U = factor * eq.significand;
                std::convert::TryInto::<$t>::try_into(result).map_err(|_| Error::Overflow)
            }
        }

    )+}
}

impl_try_from!(u16, u32, u64, u128, usize, i16, i32, i64, i128, isize);

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
            let ee = EQ::from_raw(*i, 0);
            assert_eq!(i64::try_from(ee).unwrap(), *i);
            let ee2 = EQ::from_raw(*i, 1);
            assert_eq!(i64::try_from(ee2).unwrap(), *i * 1000, "input is {}", *i);
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
        let t = EQ::<u32>::from_raw(12345, 0);
        let u = t.convert::<u64>();
        assert_eq!(u.to_raw().0, <u32 as Into<u64>>::into(t.to_raw().0));
        assert_eq!(t.to_raw().1, u.to_raw().1);
    }

    #[test]
    fn overflow() {
        let t = EQ::<u32>::from_raw(100_000, 0);
        let _ = t.try_convert::<u16>().expect_err("TryFromIntError");
        assert_eq!(u16::try_from(t), Err(EQErr::Overflow));

        // 10^15 is too big for a u32, so will overflow on conversion to integer:
        let t = EQ::<u32>::from_raw(1, 5);
        assert_eq!(u64::try_from(t), Err(EQErr::Overflow));
    }
    #[test]
    fn underflow() {
        let t = EQ::<u32>::from_raw(1, -1);
        assert_eq!(u32::try_from(t), Err(EQErr::Underflow));
    }

    #[test]
    fn normalise() {
        let q = EQ::from_raw(1_000_000, 0);
        let q2 = q.normalise();
        assert_eq!(q, q2);
        assert_eq!(q2.to_raw(), (1, 2));
    }
}
