// SPDX-License-Identifier: MIT
//
// Compiled model
//

use crate::schema::data_type::array::Array as SchemaArray;
use crate::schema::data_type::default::NonNullableDefault;
use crate::schema::data_type::default::NullableDefault;
use crate::schema::data_type::numerical;
use crate::schema::data_type::object::Object as SchemaObject;
use crate::schema::data_type::BooleanType;
use crate::schema::data_type::StringType;
use crate::schema::http_status_code::HttpStatusCode;
use crate::schema::path::Path;
use crate::schema::path_item::OperationType;
use crate::schema::PropertyName as SchemaPropertyName;

#[derive(Debug)]
pub enum Name<'a> {
    PathNameRequest(PathNameRequest<'a>),
    PathNameResponse(PathNameResponse),
    SchemaName(String),
    PropertyName(PropertyName<'a>),
}

#[derive(Debug)]
pub struct PathNameRequest<'a> {
    pub path: &'a Path,
    pub op: OperationType,
}

#[derive(Debug)]
pub struct PathNameResponse {
    pub path: Path,
    pub code: HttpStatusCode,
}

#[derive(Debug)]
pub struct PropertyName<'a> {
    pub base_name: Box<Name<'a>>,
    pub property_name: SchemaPropertyName,
}

#[derive(Debug)]
pub enum Model<'a> {
    Object(ObjectModel<'a>),
    Array(ArrayModel<'a>),
}

#[derive(Debug)]
pub struct ObjectModel<'a> {
    properties: Vec<ObjectModelPoperty<'a>>,
}

#[derive(Debug)]
pub struct ArrayModel<'a> {
    pub item_model: Name<'a>,
    pub schema: &'a SchemaArray,
}

#[derive(Debug)]
enum ObjectModelPoperty<'a> {
    Nullable(NullableObjectModelPoperty<'a>),
    Normal(NormalObjectModelPoperty<'a>),
}

#[derive(Debug)]
pub enum NullableObjectModelPoperty<'a> {
    Null,
    Boolean(&'a BooleanType<NullableDefault<bool>>),
    Object(ObjectProperty<'a>),
    Array(ArrayProperty<'a>),
    Integer(&'a numerical::NullableIntegerType),
    Number(&'a numerical::NullableNumberType),
    String(&'a StringType<NullableDefault<String>>),
}

#[derive(Debug)]
pub enum NormalObjectModelPoperty<'a> {
    Boolean(&'a BooleanType<NonNullableDefault<bool>>),
    Object(ObjectProperty<'a>),
    Array(ArrayProperty<'a>),
    Integer(&'a numerical::IntegerType),
    Number(&'a numerical::NumberType),
    String(&'a StringType<NonNullableDefault<String>>),
}

#[derive(Debug)]
pub struct ObjectProperty<'a> {
    pub model: Name<'a>,
    pub schema: &'a SchemaObject,
}

#[derive(Debug)]
pub struct ArrayProperty<'a> {
    pub item_model: Name<'a>,
    pub schema: &'a SchemaArray,
}
