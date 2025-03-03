// SPDX-License-Identifier: MIT
//
// OpenAPI Parameter Object
//

use crate::schema::data_type::DataType;
use crate::schema::media_type::MediaType;
use crate::schema::Sref;
use crate::typing::{AlwaysTrue, TaggedString};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

#[derive(Deserialize, Debug)]
pub struct Parameter {
    pub name: Name,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    #[serde(flatten)]
    pub place: Place,
    #[serde(flatten)]
    pub content_schema: ContentSchema,
}

pub type Name = TaggedString<ParameterNameTag>;
pub enum ParameterNameTag {}

pub type Description = TaggedString<ParameterDescriptionTag>;
pub enum ParameterDescriptionTag {}

#[derive(Deserialize, Debug)]
#[serde(tag = "in")]
pub enum Place {
    #[serde(rename = "query")]
    Query(QueryFlags),
    #[serde(rename = "header")]
    Header(OtherFlags),
    #[serde(rename = "path")]
    Path(PathFlags),
    #[serde(rename = "cookie")]
    Cookie(OtherFlags),
}

#[derive(Deserialize, Debug)]
pub struct QueryFlags {
    #[serde(default)] // default is false
    pub deprecated: bool,
    #[serde(rename = "allowEmptyValue", default)] // default is false
    pub allow_empty_value: bool,
}

#[derive(Deserialize, Debug)]
pub struct PathFlags {
    pub required: AlwaysTrue,
    #[serde(default)] // default is false
    pub deprecated: bool,
}

#[derive(Deserialize, Debug)]
pub struct OtherFlags {
    #[serde(default)] // default is false
    pub required: bool,
    #[serde(default)] // default is false
    pub deprecated: bool,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ContentSchema {
    SchemaAndStyle(SchemaAndStyle),
    Content(Content),
    None(None),
}

#[derive(Deserialize, Debug)]
pub struct SchemaAndStyle {
    pub style: SerializeStyle,
    pub schema: DataType,
}

#[derive(Deserialize, Debug)]
pub enum SerializeStyle {
    #[serde(rename = "form")]
    Form,
    #[serde(rename = "simple")]
    Simple,
}

#[derive(Deserialize, Debug)]
pub struct Content {
    pub content: indexmap::IndexMap<String, MediaType>,
}

#[derive(Deserialize, Debug)]
pub struct None {}

#[derive(Debug)]
pub enum ParameterOrReference {
    Parameter(Parameter),
    Reference(Sref),
}

impl<'de> Deserialize<'de> for ParameterOrReference {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LocalVisitor;

        impl<'de> Visitor<'de> for LocalVisitor {
            type Value = ParameterOrReference;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Path parameter must be either Reference or Parameter object")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value
                    .parse()
                    .map_err(de::Error::custom)
                    .map(ParameterOrReference::Reference)
            }

            fn visit_map<V>(self, map: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
                    .map(ParameterOrReference::Parameter)
            }
        }

        de.deserialize_any(LocalVisitor)
    }
}
