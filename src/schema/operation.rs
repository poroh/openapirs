// SPDX-License-Identifier: MIT
//
// OpenAPI Operation Object
//

use crate::schema::external_doc::ExternalDoc;
use crate::schema::parameter::ParameterOrReference;
use crate::schema::request_body::RequestBodyOrReference;
use crate::schema::responses::Responses;
use crate::schema::ApiDocTag;
use crate::typing::TaggedString;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Operation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<ApiDocTag>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<Summary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    #[serde(rename = "externalDocs", skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDoc>,
    #[serde(rename = "operationId", skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<OperationId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<ParameterOrReference>>,
    #[serde(rename = "requestBody", skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBodyOrReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responses: Option<Responses>,
}

pub type Summary = TaggedString<OperationSummaryTag>;
pub enum OperationSummaryTag {}

pub type Description = TaggedString<OperationDescriptionTag>;
pub enum OperationDescriptionTag {}

pub type OperationId = TaggedString<OperationIdTag>;
pub enum OperationIdTag {}
