// SPDX-License-Identifier: MIT
//
// Schema reference ($ref)
//

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

#[derive(Debug)]
pub struct Sref(String);

impl std::fmt::Display for Sref {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug)]
pub enum Error {
    URIReferenceError(uriparse::URIReferenceError),
}

impl std::str::FromStr for Sref {
    type Err = Error;
    fn from_str(s: &str) -> Result<Sref, Self::Err> {
        uriparse::URIReference::try_from(s)
            .map_err(Error::URIReferenceError)
            .map(|v| Sref(v.into()))
    }
}

impl<'de> Deserialize<'de> for Sref {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SrefVisitor;

        impl<'de> Visitor<'de> for SrefVisitor {
            type Value = Sref;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("MUST be in the form of a URI")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let _ = uriparse::URIReference::try_from(value).map_err(de::Error::custom)?;
                Ok(Sref(value.into()))
            }
        }

        de.deserialize_string(SrefVisitor)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::URIReferenceError(err) => write!(f, "uri reference error: {err}"),
        }
    }
}
