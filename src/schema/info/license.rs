// SPDX-License-Identifier: MIT
//
// OpenAPI Info License Object
//

use crate::typing::{TaggedString, TaggedURI};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct License {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: LicenseName,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<LicenseURI>,
}

pub type LicenseName = TaggedString<LicenseNameTag>;
pub enum LicenseNameTag {}

pub type LicenseURI = TaggedURI<LicenseURITag>;
pub enum LicenseURITag {}
