// SPDX-License-Identifier: MIT
//
// OpenAPI Schema
// Discriminator Object
//

use crate::schema::PropertyName;
use crate::schema::PropertyStringValue;
use crate::schema::SRef;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Discriminator {
    pub property_name: PropertyName,
    pub mapping: Option<indexmap::IndexMap<PropertyStringValue, SRef>>,
}
