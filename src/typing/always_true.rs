// SPDX-License-Identifier: MIT
//
// Unit type that is can be deserialized from true
//

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

#[derive(Debug)]
pub struct AlwaysTrue {}

pub enum Error {
    NotTrue,
}

impl<'de> Deserialize<'de> for AlwaysTrue {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LocalVisitor;

        impl<'de> Visitor<'de> for LocalVisitor {
            type Value = AlwaysTrue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("MUST be boolean with true value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value {
                    Ok(AlwaysTrue {})
                } else {
                    Err(de::Error::custom(Error::NotTrue))
                }
            }
        }

        de.deserialize_bool(LocalVisitor)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotTrue => "boolean value must be true but set to false".fmt(f),
        }
    }
}
