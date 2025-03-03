// SPDX-License-Identifier: MIT
//
// OpenAPI Schema
// Components object
//

use crate::schema::data_type::DataType;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Components {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<indexmap::IndexMap<String, DataType>>,
}
