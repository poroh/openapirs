// SPDX-License-Identifier: MIT
//
// Schema reference ($ref)
//

use crate::schema::PropertyName;
use crate::typing::TaggedString;
use serde::de;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;

#[derive(Debug)]
pub struct SRef(String);

pub type SRefParameter = TaggedString<SRefParameterTag>;
pub enum SRefParameterTag {}

pub type SRefRequestBody = TaggedString<SRefRequestBodyTag>;
pub enum SRefRequestBodyTag {}

pub type SRefSchemasObjectName = TaggedString<SRefSchemasObjectNameTag>;
pub enum SRefSchemasObjectNameTag {}

#[derive(Debug, Clone)]
pub enum SRefSchemas {
    // Normal reference
    Normal(SRefSchemasObjectName),
    ObjProperty((SRefSchemasObjectName, PropertyName)),
}

pub enum SRefSchemasTag {}

const PARAMETERS_PREFIX: &str = "#/components/parameters/";
const REQUEST_BODIES_PREFIX: &str = "#/components/requestBodies/";
const SCHEMAS_PREFIX: &str = "#/components/schemas/";

impl SRef {
    pub fn parameter_sref(&self) -> Option<SRefParameter> {
        if self.0.starts_with(PARAMETERS_PREFIX) {
            Some(SRefParameter::new(
                self.0.as_str()[PARAMETERS_PREFIX.len()..].into(),
            ))
        } else {
            None
        }
    }

    pub fn request_body_sref(&self) -> Option<SRefRequestBody> {
        if self.0.starts_with(REQUEST_BODIES_PREFIX) {
            Some(SRefRequestBody::new(
                self.0.as_str()[REQUEST_BODIES_PREFIX.len()..].into(),
            ))
        } else {
            None
        }
    }

    pub fn schemas_sref(&self) -> Result<Option<SRefSchemas>, Error> {
        if self.0.starts_with(SCHEMAS_PREFIX) {
            Ok(Some(self.0.as_str()[SCHEMAS_PREFIX.len()..].parse()?))
        } else {
            Ok(None)
        }
    }
}

impl std::fmt::Display for SRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug)]
pub enum Error {
    URIReferenceError(uriparse::URIReferenceError),
    SRefSchemaParseError(String),
}

impl std::str::FromStr for SRef {
    type Err = Error;
    fn from_str(s: &str) -> Result<SRef, Self::Err> {
        uriparse::URIReference::try_from(s)
            .map_err(Error::URIReferenceError)
            .map(|v| SRef(v.into()))
    }
}

impl<'de> Deserialize<'de> for SRef {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SRefVisitor;

        impl<'de> Visitor<'de> for SRefVisitor {
            type Value = SRef;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("MUST be in the form of a URI")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let _ = uriparse::URIReference::try_from(value).map_err(de::Error::custom)?;
                Ok(SRef(value.into()))
            }
        }

        de.deserialize_string(SRefVisitor)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::URIReferenceError(err) => write!(f, "uri reference error: {err}"),
            Self::SRefSchemaParseError(err) => write!(f, "reference parse error: {err}"),
        }
    }
}

impl std::str::FromStr for SRefSchemas {
    type Err = Error;
    fn from_str(s: &str) -> Result<SRefSchemas, Self::Err> {
        let mut split = s.split("/");
        let objname = SRefSchemasObjectName::new(
            split
                .next()
                .ok_or(Error::SRefSchemaParseError(format!("empty name: {s}")))?
                .into(),
        );
        if let Some(v) = split.next() {
            if v == "properties" {
                let propery_name = PropertyName::new(
                    split
                        .next()
                        .ok_or(Error::SRefSchemaParseError(format!(
                            "empty property name: {s}"
                        )))?
                        .into(),
                );
                Ok(SRefSchemas::ObjProperty((objname, propery_name)))
            } else {
                Err(Error::SRefSchemaParseError(format!(
                    "unexpected token: {v}"
                )))
            }
        } else {
            Ok(SRefSchemas::Normal(objname))
        }
    }
}
