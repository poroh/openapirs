// SPDX-License-Identifier: MIT
//
// Compiled data type
//

use crate::schema::components::Components;
use crate::schema::data_type::default::NonNullableDefault;
use crate::schema::data_type::default::NullableDefault;
use crate::schema::data_type::numerical;
use crate::schema::data_type::BooleanType;
use crate::schema::data_type::StringType;
use crate::schema::request_body::RequestBody as SchemaRequestBody;
use crate::schema::PropertyName;

#[derive(Debug)]
pub enum DataType<'a> {
    ActualType(ActualType<'a>),
    OneOf(OneOfType<'a>),
    AllOf(AllOfType<'a>),
    AnyOf(AnyOfType<'a>),
}

impl<'a> DataType<'a> {
    pub fn resolve_body_json(
        body: &'a SchemaRequestBody,
        components: Option<&'a Components>,
    ) -> Result<Option<DataType<'a>>, super::Error<'a>> {
        body.content
            .get("application/json")
            .and_then(|json| json.schema.as_ref())
            .map(|json_schema| {
                println!("{:?}", json_schema);
                Ok(todo!())
            })
            .transpose()
    }
}

#[derive(Debug)]
pub struct OneOfType<'a> {
    pub one_of: Vec<DataType<'a>>,
}

#[derive(Debug)]
pub struct AllOfType<'a> {
    pub all_of: Vec<DataType<'a>>,
}

#[derive(Debug)]
pub struct AnyOfType<'a> {
    pub any_of: Vec<DataType<'a>>,
}

#[derive(Debug)]
pub struct ActualType<'a> {
    pub compiled_type: CompiledType<'a>,
    pub readonly: bool,
    pub writeonly: bool,
}

#[derive(Debug)]
enum CompiledType<'a> {
    Nullable(NullableCompiledType<'a>),
    Normal(NormalCompiledType<'a>),
}

#[derive(Debug)]
pub enum NullableCompiledType<'a> {
    Null,
    Boolean(&'a BooleanType<NullableDefault<bool>>),
    Object(CompiledObject<'a>),
    Array(CompiledArray<'a>),
    Integer(&'a numerical::NullableIntegerType),
    Number(&'a numerical::NullableNumberType),
    String(&'a StringType<NullableDefault<String>>),
}

#[derive(Debug)]
pub enum NormalCompiledType<'a> {
    Boolean(&'a BooleanType<NonNullableDefault<bool>>),
    Object(CompiledObject<'a>),
    Array(CompiledArray<'a>),
    Integer(&'a numerical::IntegerType),
    Number(&'a numerical::NumberType),
    String(&'a StringType<NonNullableDefault<String>>),
}

#[derive(Debug)]
pub struct CompiledObject<'a> {
    pub properties: indexmap::IndexMap<PropertyName, DataType<'a>>,
}

#[derive(Debug)]
pub enum CompiledAdditionalProperties<'a> {
    Bool(bool),
    Schema(DataType<'a>),
}

#[derive(Debug)]
pub struct CompiledArray<'a> {
    pub properties: indexmap::IndexMap<PropertyName, DataType<'a>>,
}
