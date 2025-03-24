// SPDX-License-Identifier: MIT
//
// OpenAPI Schema Path
//

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Path(String);

#[derive(Debug)]
pub enum Error {
    MustNotbeEmpty,
    MustStartWithRoot,
}

impl Path {
    pub fn path_params_iter(&self) -> PathParamTryIter {
        PathParamTryIter {
            data: &self,
            pos: 0,
        }
    }
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

#[derive(Debug)]
pub enum PathParseError {
    CannotFindCloseBrackets(String, usize),
}

pub struct PathParamTryIter<'a> {
    data: &'a Path,
    pos: usize,
}

impl<'a> Iterator for PathParamTryIter<'a> {
    type Item = Result<&'a str, PathParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.data.0.len() {
            return None;
        }
        let rest = &self.data.0[self.pos..];
        rest.find('{')
            .map(|start_index| {
                rest[start_index..]
                    .find('}')
                    .ok_or(PathParseError::CannotFindCloseBrackets(
                        self.data.0.clone().into(),
                        start_index + self.pos,
                    ))
                    .map(|end_index| (start_index, start_index + end_index))
            })
            .map(|r| {
                r.map(|(start, end)| {
                    self.pos = self.pos + end + 1;
                    &rest[start + 1..end]
                })
            })
    }
}
