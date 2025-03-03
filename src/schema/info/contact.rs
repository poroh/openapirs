// SPDX-License-Identifier: MIT
//
// OpenAPI Info Contact Object
//

use crate::typing::{TaggedString, TaggedURI};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<ContactName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<ContactURI>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<ContactEmail>,
}

pub type ContactName = TaggedString<ContactNameTag>;
pub enum ContactNameTag {}

pub type ContactURI = TaggedURI<ContactURITag>;
pub enum ContactURITag {}

// TODO: Use email:
pub type ContactEmail = TaggedString<ContactEmailTag>;
pub enum ContactEmailTag {}
