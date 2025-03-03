// SPDX-License-Identifier: MIT
//
// OpenAPI Schema Path
//

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Path(String);

#[derive(Debug)]
pub enum Error {
    MustNotbeEmpty,
    MustStartWithRoot,
}

impl std::str::FromStr for Path {
    type Err = Error;
    fn from_str(s: &str) -> Result<Path, Self::Err> {
        let mut split = s.split('/');
        let first = split.next().ok_or(Error::MustNotbeEmpty)?;
        if !first.is_empty() {
            Err(Error::MustStartWithRoot)
        } else {
            Ok(Path(s.into()))
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MustNotbeEmpty => "path must not be empty".fmt(f),
            Self::MustStartWithRoot => "path must begin with forward slash".fmt(f),
        }
    }
}

impl<'de> Deserialize<'de> for Path {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VersionVisitor;

        impl<'de> Visitor<'de> for VersionVisitor {
            type Value = Path;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("OpenAPI path")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value.parse().map_err(de::Error::custom)
            }
        }

        de.deserialize_string(VersionVisitor)
    }
}
