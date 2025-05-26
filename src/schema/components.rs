// SPDX-License-Identifier: MIT
//
// OpenAPI Schema
// Components object
//

use crate::schema::data_type::DataType;
use crate::schema::header::HeaderOrReference;
use crate::schema::parameter::Parameter;
use crate::schema::parameter::ParameterOrReference;
use crate::schema::request_body::RequestBody;
use crate::schema::request_body::RequestBodyOrReference;
use crate::schema::responses::ResponseOrReference;
use crate::schema::sref::SRefParameter;
use crate::schema::sref::SRefRequestBody;
use crate::schema::sref::SRefSchemas;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Components {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<indexmap::IndexMap<SRefSchemas, DataType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responses: Option<indexmap::IndexMap<String, ResponseOrReference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<indexmap::IndexMap<SRefParameter, ParameterOrReference>>,
    #[serde(rename = "requestBodies", skip_serializing_if = "Option::is_none")]
    pub request_bodies: Option<indexmap::IndexMap<SRefRequestBody, RequestBodyOrReference>>,
    #[serde(rename = "headers", skip_serializing_if = "Option::is_none")]
    pub headers: Option<indexmap::IndexMap<String, HeaderOrReference>>,
    // TODO:
    // examples
    // securitySchemes
    // links
    // callbacks
}

const MAX_DEPTH: u32 = 1024;

impl Components {
    pub fn find_parameter(&self, sref: &SRefParameter) -> Option<&Parameter> {
        self.do_find_parameter(sref, 0)
    }

    fn do_find_parameter(&self, sref: &SRefParameter, depth: u32) -> Option<&Parameter> {
        if depth > MAX_DEPTH {
            None
        } else {
            self.parameters
                .as_ref()
                .and_then(|ps| ps.get(sref))
                .and_then(|por| match por {
                    ParameterOrReference::Parameter(p) => Some(p),
                    ParameterOrReference::Reference(sref) => sref
                        .sref
                        .parameter_sref()
                        .and_then(|sref| self.do_find_parameter(&sref, depth + 1)),
                })
        }
    }

    pub fn find_request_body(&self, sref: &SRefRequestBody) -> Option<&RequestBody> {
        self.do_find_request_body(sref, 0)
    }

    fn do_find_request_body(&self, sref: &SRefRequestBody, depth: u32) -> Option<&RequestBody> {
        if depth > MAX_DEPTH {
            None
        } else {
            self.request_bodies
                .as_ref()
                .and_then(|bodies| bodies.get(sref))
                .and_then(|por| match por {
                    RequestBodyOrReference::RequestBody(x) => Some(x),
                    RequestBodyOrReference::Reference(sref) => sref
                        .sref
                        .request_body_sref()
                        .and_then(|sref| self.do_find_request_body(&sref, depth + 1)),
                })
        }
    }
}
