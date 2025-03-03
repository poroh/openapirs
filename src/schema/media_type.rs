// SPDX-License-Identifier: MIT
//
// OpenAPI Media Type Object
//

use crate::schema::data_type::DataType;
use crate::schema::encoding::Encoding;
use crate::schema::PropertyName;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MediaType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<DataType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<indexmap::IndexMap<PropertyName, Encoding>>,
}
