// SPDX-License-Identifier: MIT
//
// Request body
//

use crate::schema::media_type::MediaType;
use crate::schema::Sref;
use crate::typing::TaggedString;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

#[derive(Deserialize, Debug)]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    pub content: indexmap::IndexMap<String, MediaType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

pub type Description = TaggedString<RequestBodyDescriptionTag>;
pub enum RequestBodyDescriptionTag {}

#[derive(Debug)]
pub enum RequestBodyOrReference {
    RequestBody(RequestBody),
    Reference(Sref),
}

impl<'de> Deserialize<'de> for RequestBodyOrReference {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LocalVisitor;

        impl<'de> Visitor<'de> for LocalVisitor {
            type Value = RequestBodyOrReference;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Request body must be either Reference or Request body object")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value
                    .parse()
                    .map_err(de::Error::custom)
                    .map(RequestBodyOrReference::Reference)
            }

            fn visit_map<V>(self, map: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
                    .map(RequestBodyOrReference::RequestBody)
            }
        }

        de.deserialize_any(LocalVisitor)
    }
}
