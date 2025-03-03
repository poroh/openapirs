// SPDX-License-Identifier: MIT
//
// OpenAPI Data
// Array data type definition
//
// See https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-00
// 10.3.1.  Keywords for Applying Subschemas to Arrays
//

use crate::schema::data_type::DataType;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Array {
    #[serde(rename = "prefixItems", skip_serializing_if = "Option::is_none")]
    pub prefix_items: Option<Vec<DataType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<DataType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contains: Option<Box<DataType>>,
}
