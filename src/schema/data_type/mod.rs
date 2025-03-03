// SPDX-License-Identifier: MIT
//
// OpenAPI Data Type Schema Object
//
//

pub mod array;
pub mod object;

use crate::schema::discriminator::Discriminator;
use crate::schema::external_doc::ExternalDoc;
use crate::schema::reference::Reference;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum DataType {
    Reference(Reference),
    ActualType(ActualType),
    OneOf(OneOfType),
    AllOf(AllOfType),
    AnyOf(AnyOfType),
    UnknownType(UnknownType),
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ActualType {
    #[serde(rename = "null")]
    Null,
    #[serde(rename = "boolean")]
    Boolean(BooleanType),
    #[serde(rename = "object")]
    Object(object::Object),
    #[serde(rename = "array")]
    Array(array::Array),
    #[serde(rename = "integer")]
    Integer(IntegerType),
    #[serde(rename = "number")]
    Number(NumberType),
    #[serde(rename = "string")]
    String(StringType),
}

#[derive(Deserialize, Debug)]
pub struct StringType {}

#[derive(Deserialize, Debug)]
pub struct BooleanType {}

#[derive(Deserialize, Debug)]
pub struct IntegerType {}

#[derive(Deserialize, Debug)]
pub struct NumberType {}

#[derive(Deserialize, Debug)]
pub struct OneOfType {
    #[serde(rename = "oneOf")]
    pub one_of: Vec<DataType>,
}

#[derive(Deserialize, Debug)]
pub struct AllOfType {
    #[serde(rename = "allOf")]
    pub all_of: Vec<DataType>,
}

#[derive(Deserialize, Debug)]
pub struct AnyOfType {
    #[serde(rename = "anyOf")]
    pub any_of: Vec<DataType>,
}

#[derive(Deserialize, Debug)]
pub struct UnknownType {}

#[derive(Deserialize, Debug)]
pub struct DataTypeSchemaX {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<Discriminator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDoc>,
}
