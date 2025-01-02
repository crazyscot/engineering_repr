//! Conversions to [`num::rational::Ratio`] and float

use num_rational::Ratio;
use num_traits::checked_pow;

use crate::{EQSupported, EngineeringQuantity, Error};

/////////////////////////////////////////////////////////////////////////////////
// RATIO

impl<T: EQSupported<T> + num_integer::Integer + std::convert::From<EngineeringQuantity<T>>>
    TryFrom<EngineeringQuantity<T>> for Ratio<T>
{
    type Error = Error;

    fn try_from(value: EngineeringQuantity<T>) -> Result<Self, Self::Error> {
        Ok(if value.exponent >= 0 {
            // it cannot have a fractional part
            let result: T = value.into();
            Ratio::new(Into::<T>::into(result), T::ONE)
        } else {
            let denom: T = checked_pow(T::EXPONENT_BASE, value.exponent.unsigned_abs().into())
                .ok_or(Error::Underflow)?;
            Ratio::new(value.significand, denom)
        })
    }
}

impl<T: EQSupported<T>> TryFrom<Ratio<T>> for EngineeringQuantity<T>
where
    T: num_integer::Integer,
{
    type Error = Error;

    /// This is a precise conversion, which only succeeds if the denominator of the input Ratio is a power of 1000.
    fn try_from(value: Ratio<T>) -> Result<Self, Self::Error> {
        let (num, mut denom) = value.into_raw();
        let (sig, exp) = if denom == T::ONE {
            (num, 0i8)
        } else {
            let mut exp = 0i8;
            // Scale away any powers of 1000
            loop {
                let (div, rem) = denom.div_rem(&T::EXPONENT_BASE);
                if div == T::ZERO || rem != T::ZERO {
                    break;
                }
                exp -= 1;
                denom = div;
            }

            // if 1000 divides by denom precisely, we can scale up to make a precise conversion
            let (scale, rem) = T::EXPONENT_BASE.div_rem(&denom);
            if rem != T::ZERO {
                return Err(Error::ImpreciseConversion);
            }
            // The denominator is _divided_ by scale, which means we're rounding up to the next exponent.
            // Even when the denominator is 1, this logic still works, though it might overflow so special-case it.
            if scale == T::EXPONENT_BASE {
                (num, exp)
            } else {
                (num * scale, exp - 1)
            }
        };
        EngineeringQuantity::from_raw(sig, exp)
    }
}

/////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::EngineeringQuantity as EQ;
    use super::Error;
    use num_rational::Ratio;

    #[test]
    fn to_ratio() {
        for (sig, exp, num, denom) in &[
            (1i64, 0i8, 1i64, 1i64),
            (1, 1, 1000, 1),
            (27, 2, 27_000_000, 1),
            (1, -1, 1, 1000),
            (4, -3, 4, 1_000_000_000),
            (12_345, -1, 12_345, 1000),
            (9, 6, 9_000_000_000_000_000_000, 1),
            (-9, -6, -9, 1_000_000_000_000_000_000),
        ] {
            let eq = EQ::from_raw(*sig, *exp).unwrap();
            let ratio: Ratio<i64> = eq.try_into().unwrap();
            assert_eq!(ratio, Ratio::new(*num, *denom));
        }
    }

    #[test]
    fn to_ratio_errors() {
        for (sig, exp, err) in &[
            (1i64, -7, Error::Underflow),
            (1_000_000i64, -7, Error::Underflow), // This quantity is technically valid but getting there underflows
            (1i64, -11, Error::Underflow),
        ] {
            let eq = EQ::from_raw_unchecked(*sig, *exp);
            let ratio = std::convert::TryInto::<Ratio<i64>>::try_into(eq);
            assert_eq!(ratio, Err(*err), "case: {}, {}", *sig, *exp);
        }
    }

    #[test]
    fn from_ratio() {
        for (num, denom, sig, exp) in &[
            (1i64, 1i64, 1i64, 0i8),
            (1000, 1, 1, 1),
            (27_000_000, 1, 27, 2),
            (1, 1000, 1, -1),
            (4, 1_000_000_000, 4, -3),
            (12_345, 1000, 12_345, -1),
            (9_000_000_000_000_000_000, 1, 9, 6),
            (-9, 1_000_000_000_000_000_000, -9, -6),
        ] {
            let ratio = Ratio::new(*num, *denom);
            let eq: EQ<i64> = ratio.try_into().unwrap();
            let expected = EQ::from_raw(*sig, *exp).unwrap();
            assert_eq!(eq, expected, "inputs: {num:?}, {denom:?}",);
        }
    }

    #[test]
    fn from_ratio_errors() {
        let ratio = Ratio::new(1, 333);
        let result = EQ::<i64>::try_from(ratio).unwrap_err();
        assert_eq!(result, Error::ImpreciseConversion);
    }
}
