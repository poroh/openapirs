// SPDX-License-Identifier: MIT
//
// OpenAPI Schema
// Nullable and Non-nullable default values
//

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct NullableDefault<T: for<'a> Deserialize<'a>> {
    #[serde(
        rename = "default",
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option",
        default
    )]
    pub value: Option<Option<T>>,
}

#[derive(Deserialize, Debug)]
pub struct NonNullableDefault<T> {
    #[serde(rename = "default", skip_serializing_if = "Option::is_none")]
    pub value: Option<T>,
}
