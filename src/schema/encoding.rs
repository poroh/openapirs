// SPDX-License-Identifier: MIT
//
// OpenAPI Schema
// Encoding Object
//

use crate::schema::content_type::ContentType;
use crate::schema::header::Header;
use crate::schema::HeaderName;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Encoding {
    pub content_type: Option<ContentType>,
    pub headers: Option<indexmap::IndexMap<HeaderName, Header>>,
}
