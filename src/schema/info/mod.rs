// SPDX-License-Identifier: MIT
//
// OpenAPI Info
//

pub mod contact;
pub mod license;

use crate::typing::{TaggedString, TaggedURI};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Info {
    pub title: ApiTitle,
    pub version: ApiVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<ApiDescription>,
    #[serde(rename = "termsOfService", skip_serializing_if = "Option::is_none")]
    pub term_of_service: Option<TermOfService>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<contact::Contact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<license::License>,
}

pub type ApiTitle = TaggedString<ApiTitleTag>;
pub enum ApiTitleTag {}

pub type ApiVersion = TaggedString<ApiVersionTag>;
pub enum ApiVersionTag {}

pub type ApiDescription = TaggedString<ApiDescriptionTag>;
pub enum ApiDescriptionTag {}

pub type TermOfService = TaggedURI<TermOfServiceTag>;
pub enum TermOfServiceTag {}
