// SPDX-License-Identifier: MIT
//
// OpenAPI Version
//

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

#[derive(PartialEq, Eq, Debug)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Debug)]
pub enum Error {
    NoExpectedVersionPart(&'static str),
    InvalidVersionPart((&'static str, std::num::ParseIntError)),
    UnexpectedRemainder(String),
}

impl std::str::FromStr for Version {
    type Err = Error;
    fn from_str(s: &str) -> Result<Version, Self::Err> {
        let mut split = s.split('.');
        let parse_part = |split: &mut std::str::Split<_>, name: &'static str| {
            split
                .next()
                .ok_or(Error::NoExpectedVersionPart(name))
                .and_then(|v| v.parse().map_err(|e| Error::InvalidVersionPart((name, e))))
        };
        let major: u32 = parse_part(&mut split, "major")?;
        let minor: u32 = parse_part(&mut split, "minor")?;
        let patch: u32 = parse_part(&mut split, "patch")?;
        if let Some(v) = split.next() {
            Err(Error::UnexpectedRemainder(v.into()))?
        }
        Ok(Version {
            major,
            minor,
            patch,
        })
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoExpectedVersionPart(name) => write!(f, "invalid version part: {name}"),
            Self::InvalidVersionPart((name, num_err)) => {
                write!(f, "invalid version part: {name}: {num_err}")
            }
            Self::UnexpectedRemainder(v) => write!(f, "invalid version remainder: {v}"),
        }
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LocalVisitor;

        impl<'de> Visitor<'de> for LocalVisitor {
            type Value = Version;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("OpenAPI version in major.minor.patch versioning scheme")
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn correct_version_parse() {
        let parse_result: Result<Version, _> = "3.1.2".parse();
        assert!(parse_result.is_ok());
        assert_eq!(
            parse_result.unwrap(),
            Version {
                major: 3,
                minor: 1,
                patch: 2
            }
        );
    }

    #[test]
    fn version_invalid_major_num_format() {
        let parse_result: Result<Version, _> = "a.1.2".parse();
        assert!(parse_result.is_err());
        assert!(parse_result.unwrap_err().to_string().contains("major"));
    }

    #[test]
    fn version_invalid_minor_num_format() {
        let parse_result: Result<Version, _> = "2.a.2".parse();
        assert!(parse_result.is_err());
        assert!(parse_result.unwrap_err().to_string().contains("minor"));
    }

    #[test]
    fn version_invalid_patch_num_format() {
        let parse_result: Result<Version, _> = "3.1.p".parse();
        assert!(parse_result.is_err());
        assert!(parse_result.unwrap_err().to_string().contains("patch"));
    }
}
