// SPDX-License-Identifier: MIT
//
// OpenAPI Schema
// HTTP Status Code
//

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum HttpStatusCode {
    Specific(Specific),
    Pattern(Pattern),
}

pub enum Error {
    CodeOutsideRange(u16, u16, u16),
    InvalidPattern(String),
}

impl std::str::FromStr for HttpStatusCode {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //
        if let Ok(v) = s.parse::<u16>() {
            v.try_into().map(Self::Specific)
        } else {
            s.parse().map(Self::Pattern)
        }
    }
}

impl<'de> Deserialize<'de> for HttpStatusCode {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LocalVisitor;

        impl<'de> Visitor<'de> for LocalVisitor {
            type Value = HttpStatusCode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("HTTP status code must be either number in range [100..600) or pattern '1XX', '2XX' ... '5XX'")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value.parse().map_err(de::Error::custom)
            }
        }

        de.deserialize_string(LocalVisitor)
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct Specific(u16);

impl TryFrom<u16> for Specific {
    type Error = Error;
    fn try_from(v: u16) -> Result<Self, Error> {
        if (100..=599).contains(&v) {
            Ok(Self(v))
        } else {
            Err(Error::CodeOutsideRange(v, 100, 599))
        }
    }
}

impl std::fmt::Debug for Specific {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct Pattern {
    // 1 == 1XX, 2 = 2XX, ..., 5 == 5XX
    code_class: u8,
}

impl std::fmt::Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.code_class.fmt(f)?;
        "XX".fmt(f)
    }
}

impl std::str::FromStr for Pattern {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let code_class = chars
            .next()
            .and_then(|v| v.to_digit(10))
            .and_then(|v| {
                if (1..=5).contains(&v) {
                    Some(v as u8)
                } else {
                    None
                }
            })
            .ok_or_else(|| Error::InvalidPattern(s.into()))?;
        chars
            .next()
            .and_then(|v| if v == 'X' { Some(()) } else { None })
            .ok_or_else(|| Error::InvalidPattern(s.into()))?;
        chars
            .next()
            .and_then(|v| if v == 'X' { Some(()) } else { None })
            .ok_or_else(|| Error::InvalidPattern(s.into()))?;
        Ok(Self { code_class })
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CodeOutsideRange(v, min, max) => {
                write!(f, "HTTP code {v} must be in range: [{min}, {max}]")
            }
            Self::InvalidPattern(v) => write!(f, "HTTP code pattern is invalid: {v}"),
        }
    }
}
