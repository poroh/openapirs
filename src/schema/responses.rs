// SPDX-License-Identifier: MIT
//
// OpenAPI Schema
// Responses Object
//

use crate::schema::http_status_code::HttpStatusCode;
use crate::schema::reference::Reference;
use crate::schema::response::Response;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Responses {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<ResponseOrReference>,
    #[serde(flatten)]
    pub codes: indexmap::IndexMap<HttpStatusCode, ResponseOrReference>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ResponseOrReference {
    Reference(Reference),
    Response(Response),
}
