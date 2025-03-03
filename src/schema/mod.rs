// SPDX-License-Identifier: MIT
//
// OpenAPI Schema
//

pub mod components;
pub mod content_type;
pub mod data_type;
pub mod discriminator;
pub mod encoding;
pub mod external_doc;
pub mod header;
pub mod http_status_code;
pub mod info;
pub mod media_type;
pub mod operation;
pub mod parameter;
pub mod path;
pub mod path_item;
pub mod reference;
pub mod request_body;
pub mod response;
pub mod responses;
pub mod server;
pub mod sref;
pub mod version;

use crate::typing::TaggedString;
use serde::Deserialize;

pub type Sref = sref::Sref;

#[derive(Deserialize, Debug)]
pub struct Description {
    pub openapi: version::Version,
    pub info: info::Info,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<server::Server>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<indexmap::IndexMap<path::Path, path_item::PathItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<components::Components>,
}

// Note: ApiDocTagTag is kind of conflict of the naming conventions.
// ApiDocTag defines tags inside OpenAPI
pub type ApiDocTag = TaggedString<ApiDocTagTag>;
pub enum ApiDocTagTag {}

// Name of the property inside the object
pub type PropertyName = TaggedString<PropertyNameTag>;
pub enum PropertyNameTag {}

// String value of the property with string type. Used in descriminators.
pub type PropertyStringValue = TaggedString<PropertyStringValueTag>;
pub enum PropertyStringValueTag {}

// Name of the header. TODO: any restrictions on header names?
pub type HeaderName = TaggedString<HeaderNameTag>;
pub enum HeaderNameTag {}
