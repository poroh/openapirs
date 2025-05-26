// SPDX-License-Identifier: MIT
//
// OpenAPI Schema
// Reference Object
//

use crate::schema::SRef;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Reference {
    #[serde(rename = "$ref")]
    pub sref: SRef,
}
