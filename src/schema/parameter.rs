// SPDX-License-Identifier: MIT
//
// OpenAPI Parameter Object
//

use crate::schema::data_type::DataType;
use crate::schema::media_type::MediaType;
use crate::schema::reference::Reference;
use crate::typing::AlwaysTrue;
use crate::typing::TaggedString;
use serde::Deserialize;

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

impl Parameter {
    pub fn is_query(&self) -> bool {
        matches!(
            self,
            Parameter {
                place: Place::Query(_),
                ..
            }
        )
    }

    pub fn is_header(&self) -> bool {
        matches!(
            self,
            Parameter {
                place: Place::Header(_),
                ..
            }
        )
    }

    pub fn is_cookie(&self) -> bool {
        matches!(
            self,
            Parameter {
                place: Place::Cookie(_),
                ..
            }
        )
    }
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
    pub required: bool,
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

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ParameterOrReference {
    Parameter(Parameter),
    Reference(Reference),
}
