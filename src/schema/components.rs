// SPDX-License-Identifier: MIT
//
// OpenAPI Schema
// Components object
//

use crate::schema::data_type::DataType;
use crate::schema::header::HeaderOrReference;
use crate::schema::parameter::ParameterOrReference;
use crate::schema::request_body::RequestBodyOrReference;
use crate::schema::responses::ResponseOrReference;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Components {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<indexmap::IndexMap<String, DataType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responses: Option<indexmap::IndexMap<String, ResponseOrReference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<indexmap::IndexMap<String, ParameterOrReference>>,
    #[serde(rename = "requestBodies", skip_serializing_if = "Option::is_none")]
    pub request_bodies: Option<indexmap::IndexMap<String, RequestBodyOrReference>>,
    #[serde(rename = "headers", skip_serializing_if = "Option::is_none")]
    pub headers: Option<indexmap::IndexMap<String, HeaderOrReference>>,
    // TODO:
    // examples
    // securitySchemes
    // links
    // callbacks
}
