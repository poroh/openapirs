// SPDX-License-Identifier: MIT
//
// Request body
//

use crate::schema::media_type::MediaType;
use crate::schema::reference::Reference;
use crate::typing::TaggedString;
use serde::Deserialize;

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

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum RequestBodyOrReference {
    RequestBody(RequestBody),
    Reference(Reference),
}
