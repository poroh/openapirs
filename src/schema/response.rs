// SPDX-License-Identifier: MIT
//
// OpenAPI Schema
// Response Object
//

use crate::schema::header::HeaderOrReference;
use crate::schema::HeaderName;
use crate::schema::media_type::MediaType;
use crate::typing::TaggedString;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    pub content: indexmap::IndexMap<String, MediaType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<indexmap::IndexMap<HeaderName, HeaderOrReference>>,
}

pub type Description = TaggedString<ResponseDescriptionTag>;
pub enum ResponseDescriptionTag {}
