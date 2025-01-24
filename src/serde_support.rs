//! Serde support

use std::{marker::PhantomData, str::FromStr};

use serde::{de, Deserialize, Serialize};

use crate::{EQSupported, EngineeringQuantity};

/// <div class="warning">
/// Available on feature <b>serde</b> only.
/// </div>
impl<T: EQSupported<T>> Serialize for EngineeringQuantity<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.with_precision(0).to_string();
        serializer.serialize_str(&s)
    }
}

struct EQVisitor<U: EQSupported<U>>(pub PhantomData<U>);
impl<U: EQSupported<U>> EQVisitor<U> {
    fn new() -> Self {
        Self(PhantomData)
    }
}
impl<U: EQSupported<U> + FromStr + std::convert::TryFrom<u128> + std::convert::TryFrom<i128>>
    de::Visitor<'_> for EQVisitor<U>
{
    type Value = EngineeringQuantity<U>;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let res = EngineeringQuantity::from_str(v);
        res.map_err(|_| de::Error::invalid_value(de::Unexpected::Str(v), &self))
    }

    fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let u = U::try_from(value).map_err(|_| de::Error::custom("failed to convert integer"))?;
        Ok(Self::Value::from(u))
    }
    fn visit_i128<E>(self, value: i128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let u = U::try_from(value).map_err(|_| de::Error::custom("failed to convert integer"))?;
        Ok(Self::Value::from(u))
    }
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_u128(value.into())
    }
    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_i128(value.into())
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an engineering quantity (1M, 4.5k, etc) or an integer")
    }
}

/// <div class="warning">
/// Available on feature <b>serde</b> only.
/// </div>
impl<
        'de,
        T: EQSupported<T> + FromStr + std::convert::TryFrom<u128> + std::convert::TryFrom<i128>,
    > Deserialize<'de> for EngineeringQuantity<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(EQVisitor::<T>::new())
    }
}

#[cfg(test)]
mod test {
    use crate::EngineeringQuantity as EQ;

    #[test]
    fn pairwise_precision() {
        let e1 = EQ::from_raw(1_234, 2).unwrap();
        let json = serde_json::to_string(&e1).unwrap();
        println!("{json}");
        let e2 = serde_json::from_str(&json).unwrap();
        assert_eq!(e1, e2);
    }

    #[test]
    fn type_mismatch() {
        let _ = serde_json::from_str::<EQ<i32>>("false").expect_err("type mismatch");
    }

    #[test]
    fn deserialize_int() {
        let eq = serde_json::from_str::<EQ<u128>>("42768").unwrap();
        assert_eq!(eq.to_raw(), (42768, 0));
        let eq = serde_json::from_str::<EQ<i128>>("-42768").unwrap();
        assert_eq!(eq.to_raw(), (-42768, 0));
        let eq = serde_json::from_str::<EQ<u32>>("12345").unwrap();
        assert_eq!(eq.to_raw(), (12345, 0));
        let eq = serde_json::from_str::<EQ<i16>>("-9876").unwrap();
        assert_eq!(eq.to_raw(), (-9876, 0));

        // overflow (raw integer that won't fit)
        let _ = serde_json::from_str::<EQ<u16>>("65537").unwrap_err();
    }
}
