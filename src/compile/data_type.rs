// SPDX-License-Identifier: MIT
//
// Compiled data type
// with all required schemas with it.
//

use crate::schema::data_type::default::NonNullableDefault;
use crate::schema::data_type::default::NullableDefault;
use crate::schema::data_type::numerical;
use crate::schema::data_type::BooleanType;
use crate::schema::data_type::StringType;
use crate::schema::PropertyName;
use crate::schema::sref::SRefSchemas;

#[derive(Debug)]
pub enum DataType<'a> {
    ActualType(ActualType<'a>),
    OneOf(OneOfType<'a>),
    AllOf(AllOfType<'a>),
    AnyOf(AnyOfType<'a>),
}

// Resolve can be either DataType that refers to one or more other
// DataTypes or just reference to schema datatype.
#[derive(Debug)]
pub enum TypeOrRef<'a> {
    ActualType(DataType<'a>),
    Reference(SRefSchemas),
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
pub enum CompiledType<'a> {
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

#[derive(Debug, Default)]
pub struct CompiledObject<'a> {
    pub properties: indexmap::IndexMap<PropertyName, TypeOrRef<'a>>,
}

#[derive(Debug)]
pub enum CompiledAdditionalProperties<'a> {
    Bool(bool),
    Schema(DataType<'a>),
}

#[derive(Debug)]
pub struct CompiledArray<'a> {
    pub items: Box<TypeOrRef<'a>>,
}
