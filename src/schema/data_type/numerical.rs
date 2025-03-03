// SPDX-License-Identifier: MIT
//
// OpenAPI Data
// Numerical data types
//

use crate::schema::data_type::default::NonNullableDefault;
use crate::schema::data_type::default::NullableDefault;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum NumberType {
    WithFormat(NumberWithFormat),
    WithOutFormat(Float),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum NullableNumberType {
    WithFormat(NullableNumberWithFormat),
    WithOutFormat(NullableFloat),
}

#[derive(Deserialize, Debug)]
#[serde(tag = "format")]
pub enum NumberWithFormat {
    #[serde(rename = "float")]
    Float(Float),
    #[serde(rename = "double")]
    Double(Double),
}

#[derive(Deserialize, Debug)]
#[serde(tag = "format")]
pub enum NullableNumberWithFormat {
    #[serde(rename = "float")]
    Float(NullableFloat),
    #[serde(rename = "double")]
    Double(NullableDouble),
}

pub type Float = Numerical<f32, NonNullableDefault<f32>>;
pub type Double = Numerical<f64, NonNullableDefault<f64>>;
pub type NullableFloat = Numerical<f32, NullableDefault<f32>>;
pub type NullableDouble = Numerical<f64, NullableDefault<f64>>;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum IntegerType {
    WithFormat(IntegerWithFormat),
    WithOutFormat(Integer32),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum NullableIntegerType {
    WithFormat(NullableIntegerWithFormat),
    WithOutFormat(NullableInteger32),
}

#[derive(Deserialize, Debug)]
#[serde(tag = "format")]
pub enum IntegerWithFormat {
    #[serde(rename = "int32")]
    Int32(Integer32),
    #[serde(rename = "int64")]
    Int64(Integer64),
}

#[derive(Deserialize, Debug)]
#[serde(tag = "format")]
pub enum NullableIntegerWithFormat {
    #[serde(rename = "int32")]
    Int32(NullableInteger32),
    #[serde(rename = "int64")]
    Int64(NullableInteger64),
}

pub type Integer32 = Numerical<i32, NonNullableDefault<i32>>;
pub type Integer64 = Numerical<i64, NonNullableDefault<i64>>;
pub type NullableInteger32 = Numerical<i32, NullableDefault<i32>>;
pub type NullableInteger64 = Numerical<i64, NullableDefault<i64>>;

#[derive(Deserialize, Debug)]
pub struct Numerical<T, DefaultV> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<T>,
    #[serde(rename = "exclusiveMinimum", default)]
    pub exclisive_minimum: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<T>,
    #[serde(rename = "exclusiveMaximum", default)]
    pub exclisive_maximum: bool,
    #[serde(flatten)]
    pub default_info: DefaultV,
}
