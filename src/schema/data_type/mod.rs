// SPDX-License-Identifier: MIT
//
// OpenAPI Data Type Schema Object
//
//

pub mod array;
pub mod default;
pub mod numerical;
pub mod object;

use crate::schema::data_type::default::NonNullableDefault;
use crate::schema::data_type::default::NullableDefault;
use crate::schema::discriminator::Discriminator;
use crate::schema::external_doc::ExternalDoc;
use crate::schema::reference::Reference;
use crate::typing::AlwaysTrue;
use serde::de;
use serde::de::Deserializer;
use serde::de::MapAccess;
use serde::de::Visitor;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum DataType {
    Reference(Reference),
    ActualType(Box<ActualType>),
    OneOf(OneOfType),
    AllOf(AllOfType),
    AnyOf(AnyOfType),
    Empty(EmptyType),
    UnknownType(UnknownType),
}

#[derive(Deserialize, Debug)]
pub struct ActualType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<Discriminator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDoc>,
    #[serde(flatten)]
    pub type_schema: MaybeNullableTypeSchema,
    // 3.0.X specification (removed by 3.1.X)
    #[serde(rename = "readOnly", default)]
    pub readonly: bool,
    #[serde(rename = "writeOnly", default)]
    pub writeonly: bool,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum MaybeNullableTypeSchema {
    Nullable(NullableTypeSchema),
    Normal(TypeSchema),
    // For some reasons many specs doesn't add "type: object" and "type: array"
    Object(object::Object),
    Array(array::Array),
}

#[derive(Deserialize, Debug)]
pub struct NullableTypeSchema {
    pub nullable: AlwaysTrue,
    #[serde(flatten)]
    pub schema: NullalbleTypeSchema,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TypeSchema {
    #[serde(rename = "null")]
    Null,
    #[serde(rename = "boolean")]
    Boolean(BooleanType<NonNullableDefault<bool>>),
    #[serde(rename = "integer")]
    Integer(numerical::IntegerType),
    #[serde(rename = "number")]
    Number(numerical::NumberType),
    #[serde(rename = "string")]
    String(StringType<NonNullableDefault<String>>),
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum NullalbleTypeSchema {
    #[serde(rename = "null")]
    Null,
    #[serde(rename = "boolean")]
    Boolean(BooleanType<NullableDefault<bool>>),
    #[serde(rename = "object")]
    Object(object::Object),
    #[serde(rename = "array")]
    Array(array::Array),
    #[serde(rename = "integer")]
    Integer(numerical::NullableIntegerType),
    #[serde(rename = "number")]
    Number(numerical::NullableNumberType),
    #[serde(rename = "string")]
    String(StringType<NullableDefault<String>>),
}

#[derive(Deserialize, Debug)]
pub struct StringType<D> {
    #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u64>,
    #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u64>,
    #[serde(flatten)]
    pub default: D,
}

#[derive(Deserialize, Debug)]
pub struct BooleanType<D> {
    #[serde(flatten)]
    pub default: D,
}

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

#[derive(Debug)]
pub struct EmptyType {}

impl<'de> Deserialize<'de> for EmptyType {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LocalVisitor;

        impl<'de> Visitor<'de> for LocalVisitor {
            type Value = EmptyType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("empty object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                // If the object has any key-value pairs, that's an error.
                if let Some(key) = map.next_key::<String>()? {
                    Err(de::Error::unknown_field(&key, &[]))
                } else {
                    Ok(EmptyType {})
                }
            }
        }

        de.deserialize_map(LocalVisitor)
    }
}
