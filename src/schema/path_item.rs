// SPDX-License-Identifier: MIT
//
// OpenAPI Schema Path objects
//

use crate::schema::operation::Operation;
use crate::schema::parameter::ParameterOrReference;
use crate::schema::server::Server;
use crate::schema::Sref;
use crate::typing::TaggedString;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PathItem {
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub sref: Option<Sref>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<Summary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<ParameterOrReference>>,
}

pub type Summary = TaggedString<SummaryTag>;
pub enum SummaryTag {}

pub type Description = TaggedString<DescriptionTag>;
pub enum DescriptionTag {}
