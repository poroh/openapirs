// SPDX-License-Identifier: MIT
//
// OpenAPI Schema
// Reference Object
//

use crate::schema::Sref;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Reference {
    #[serde(rename = "$ref")]
    pub sref: Sref,
}
