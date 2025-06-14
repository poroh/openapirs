// SPDX-License-Identifier: MIT
//
// Model is flattened set of schema objects
// that is needed to be produced as output.
//

pub mod name;

use crate::compile::data_type::ActualType;
use crate::compile::data_type::CompiledObject;
use crate::compile::data_type::CompiledType;
use crate::compile::data_type::DataType;
use crate::compile::data_type::NormalCompiledType;
use crate::compile::data_type::TypeOrSchemaRef;
use crate::compile::Compiled;
use crate::schema::data_type::default::NonNullableDefault;
use crate::schema::data_type::default::NullableDefault;
use crate::schema::data_type::numerical;
use crate::schema::data_type::BooleanType;
use crate::schema::data_type::StringType;
use crate::schema::PropertyName;
use name::Name;

#[derive(Debug)]
pub enum Model<'a> {
    Object(Object<'a>),
    Enum(Enum<'a>),
}

#[derive(Debug)]
pub struct Object<'a> {
    pub properties: Vec<(&'a PropertyName, PropertyType<'a>)>,
}

#[derive(Debug)]
pub struct Enum<'a> {
    pub members: Vec<Name<'a>>,
}

#[derive(Debug)]
pub struct PropertyType<'a> {
    pub simple_type: SimpleType<'a>,
    pub readonly: bool,
    pub writeonly: bool,
}

#[derive(Debug)]
pub enum SimpleType<'a> {
    Nullable(NullableType<'a>),
    Normal(NormalType<'a>),
}

#[derive(Debug)]
pub enum NullableType<'a> {
    Null,
    Boolean(&'a BooleanType<NullableDefault<bool>>),
    Integer(&'a numerical::NullableIntegerType),
    Number(&'a numerical::NullableNumberType),
    String(&'a StringType<NullableDefault<String>>),
    Object(Name<'a>),
    Array(Name<'a>),
}

#[derive(Debug)]
pub enum NormalType<'a> {
    Boolean(&'a BooleanType<NonNullableDefault<bool>>),
    Integer(&'a numerical::IntegerType),
    Number(&'a numerical::NumberType),
    String(&'a StringType<NonNullableDefault<String>>),
    Object(Name<'a>),
    Array(Name<'a>),
}

pub fn build<'a>(compiled: &'a Compiled<'a>) -> Vec<(Name<'a>, Model<'a>)> {
    compiled
        .schemas
        .iter()
        .flat_map(|(name, v)| match v {
            DataType::ActualType(t) => match &t.compiled_type {
                CompiledType::Nullable(_) => todo!(),
                CompiledType::Normal(t) => match t {
                    NormalCompiledType::Boolean(_) => todo!(),
                    NormalCompiledType::Object(obj) => build_object(Name::Schemas(name), obj),
                    NormalCompiledType::Array(_) => todo!(),
                    NormalCompiledType::Integer(_) => todo!(),
                    NormalCompiledType::Number(_) => todo!(),
                    NormalCompiledType::String(_) => todo!(),
                },
            },
            DataType::OneOf(_) => todo!(),
            DataType::AllOf(_) => todo!(),
            DataType::AnyOf(_) => todo!(),
        })
        .collect()
}

pub fn build_object<'a>(name: Name<'a>, obj: &'a CompiledObject<'a>) -> Vec<(Name<'a>, Model<'a>)> {
    let properties = obj
        .properties
        .iter()
        .map(|(name, t)| match t {
            TypeOrSchemaRef::DataType(t) => match &t {
                DataType::ActualType(t) => match &t.compiled_type {
                    CompiledType::Nullable(_) => todo!(),
                    CompiledType::Normal(tn) => match tn {
                        NormalCompiledType::Boolean(v) => {
                            (name, build_npt(t, NormalType::Boolean(v)))
                        }
                        NormalCompiledType::Object(_) => todo!(),
                        NormalCompiledType::Array(_) => todo!(),
                        NormalCompiledType::Integer(v) => {
                            (name, build_npt(t, NormalType::Integer(v)))
                        }
                        NormalCompiledType::Number(v) => {
                            (name, build_npt(t, NormalType::Number(v)))
                        }
                        NormalCompiledType::String(v) => {
                            (name, build_npt(t, NormalType::String(v)))
                        }
                    },
                },
                DataType::OneOf(_) => todo!(),
                DataType::AllOf(_) => todo!(),
                DataType::AnyOf(_) => todo!(),
            },
            TypeOrSchemaRef::Reference(_) => todo!(),
        })
        .collect();
    vec![(name, Model::Object(Object { properties }))]
}

fn build_npt<'a>(t: &'a ActualType<'a>, nt: NormalType<'a>) -> PropertyType<'a> {
    PropertyType {
        simple_type: SimpleType::Normal(nt),
        readonly: t.readonly,
        writeonly: t.writeonly,
    }
}
