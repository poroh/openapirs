// SPDX-License-Identifier: MIT
//
// Unit type that is can be deserialized from true
//

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

#[derive(Debug)]
pub struct AlwaysFalse {}

pub enum Error {
    NotFalse,
}

impl<'de> Deserialize<'de> for AlwaysFalse {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LocalVisitor;

        impl<'de> Visitor<'de> for LocalVisitor {
            type Value = AlwaysFalse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("MUST be boolean with false value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if !value {
                    Ok(AlwaysFalse {})
                } else {
                    Err(de::Error::custom(Error::NotFalse))
                }
            }
        }

        de.deserialize_bool(LocalVisitor)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFalse => "boolean value must be false but set to true".fmt(f),
        }
    }
}
