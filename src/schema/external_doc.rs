// SPDX-License-Identifier: MIT
//
// OpenAPI Schema
// External documnent
//

use crate::typing::TaggedString;
use crate::typing::TaggedURI;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ExternalDoc {
    pub url: ExternalDocURI,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
}

pub type ExternalDocURI = TaggedURI<ExternalDocURITag>;
pub enum ExternalDocURITag {}

pub type Description = TaggedString<ExternalDocDescriptionTag>;
pub enum ExternalDocDescriptionTag {}
