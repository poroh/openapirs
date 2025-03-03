// SPDX-License-Identifier: MIT
//
// OpenAPI Data
// Object data typde definition
//
// See https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-00
// 10.3.2.  Keywords for Applying Subschemas to Objects
//

use crate::schema::data_type::DataType;
use crate::schema::PropertyName;
use crate::typing::TaggedString;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Object {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<indexmap::IndexMap<PropertyName, DataType>>,
    #[serde(rename = "patternProperties", skip_serializing_if = "Option::is_none")]
    pub pattern_properties: Option<indexmap::IndexMap<Ecma262RegEx, DataType>>,
    #[serde(
        rename = "additionalProperties",
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_properties: Option<Box<AdditionalProperties>>,
    #[serde(rename = "propertyNames", skip_serializing_if = "Option::is_none")]
    pub property_names: Option<Box<DataType>>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum AdditionalProperties {
    Bool(bool),
    Schema(DataType),
}

// TODO:
pub type Ecma262RegEx = TaggedString<Ecma262RegExTag>;
pub enum Ecma262RegExTag {}
