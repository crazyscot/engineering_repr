//! Serde support

use std::{marker::PhantomData, str::FromStr};

use serde::{de, Deserialize, Serialize};

use crate::{EQSupported, EngineeringQuantity};

impl<T: EQSupported<T>> Serialize for EngineeringQuantity<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.with_precision(0).to_string();
        serializer.serialize_str(&s)
    }
}

impl<'de, T: EQSupported<T> + FromStr> Deserialize<'de> for EngineeringQuantity<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EQVisitor<U: EQSupported<U>> {
            phantom: PhantomData<U>,
        }
        impl<U: EQSupported<U>> EQVisitor<U> {
            fn new() -> Self {
                Self {
                    phantom: PhantomData,
                }
            }
        }
        impl<U: EQSupported<U> + FromStr> de::Visitor<'_> for EQVisitor<U> {
            type Value = EngineeringQuantity<U>;
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let res = EngineeringQuantity::from_str(v);
                res.map_err(|_| de::Error::invalid_value(de::Unexpected::Str(v), &self))
            }

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an engineering quantity (1M, 4.5k, etc)")
            }
        }
        deserializer.deserialize_str(EQVisitor::<T>::new())
    }
}

#[cfg(test)]
mod test {
    use crate::EngineeringQuantity as EQ;

    #[test]
    fn pairwise_precision() {
        let e1 = EQ::from_raw(1_234_567, 2);
        let json = serde_json::to_string(&e1).unwrap();
        println!("{json}");
        let e2 = serde_json::from_str(&json).unwrap();
        assert_eq!(e1, e2);
    }
}
