// SPDX-License-Identifier: MIT
//
// OpenAPI Schema
// Header Object
//

use crate::schema::data_type::DataType;
use crate::schema::media_type::MediaType;
use crate::schema::reference::Reference;
use crate::typing::TaggedString;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Header {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    #[serde(default)] // default is false
    pub required: bool,
    #[serde(default)] // default is false
    pub deprecated: bool,
    #[serde(flatten)]
    pub content_schema: ContentSchema,
}

pub type Description = TaggedString<HeaderDescriptionTag>;
pub enum HeaderDescriptionTag {}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ContentSchema {
    SchemaAndStyle(SchemaAndStyle),
    Content(Content),
}

#[derive(Deserialize, Debug)]
pub struct SchemaAndStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<SerializeStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explode: Option<bool>,
    pub schema: DataType,
}

#[derive(Deserialize, Debug)]
pub enum SerializeStyle {
    #[serde(rename = "simple")]
    Simple,
}

#[derive(Deserialize, Debug)]
pub struct Content {
    pub content: indexmap::IndexMap<String, MediaType>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum HeaderOrReference {
    Header(Header),
    Reference(Reference),
}
