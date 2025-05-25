// SPDX-License-Identifier: MIT
//
// Compiled data type
//

use crate::schema::components::Components;
use crate::schema::data_type::default::NonNullableDefault;
use crate::schema::data_type::default::NullableDefault;
use crate::schema::data_type::numerical;
use crate::schema::data_type::BooleanType;
use crate::schema::data_type::DataType as SchemaDataType;
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
    ) -> Result<Option<ResolveResult<'a>>, super::Error<'a>> {
        body.content
            .get("application/json")
            .and_then(|json| json.schema.as_ref())
            .map(|json_schema| Self::resolve(json_schema, components))
            .transpose()
    }

    pub fn resolve(
        schema_dt: &'a SchemaDataType,
        components: Option<&'a Components>,
    ) -> Result<ResolveResult<'a>, super::Error<'a>> {
        todo!()

        // match schema_dt {
        //     SchemaDataType::Reference(r) => {
        //         println!("{:?}", r);
        //         Ok(todo!())
        //     }
        //     SchemaDataType::ActualType(t) => Ok(DataType::ActualType(ActualType {
        //         compiled_type: todo!(),
        //         readonly: todo!(),
        //         writeonly: todo!(),
        //     })),
        //     SchemaDataType::OneOf(t) => Ok(DataType::OneOf(OneOfType { one_of: todo!() })),
        //     SchemaDataType::AllOf(AllOfType) => Ok(DataType::AllOf(AllOfType { all_of: todo!() })),
        //     SchemaDataType::AnyOf(AnyOfType) => Ok(DataType::AnyOf(AnyOfType { any_of: todo!() })),
        //     SchemaDataType::Empty(EmptyType) => {
        //         todo!()
        //     }
        //     SchemaDataType::UnknownType(UnknownType) => Err(super::Error::UnknownBodyType(path)),
        // }
    }
}

#[derive(Debug)]
pub struct ResolveResult<'a> {
    pub data_type: DataType<'a>,
    pub schema_name: Option<&'a String>,
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
