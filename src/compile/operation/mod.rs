// SPDX-License-Identifier: MIT
//
// Compiled operation
//

pub mod parameter;
pub mod request_body;
pub mod response_body;

use crate::schema::http_status_code::HttpStatusCode;
use crate::schema::parameter::Name as SchemaParameterName;
use crate::schema::path::Path;
use crate::schema::path_item::OperationType;
use parameter::Parameter;
use request_body::RequestBodyOrReference;
use response_body::ResponseBodyOrReference;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Operation<'a> {
    pub op_type: OperationType,
    pub path: &'a Path,
    pub path_params: HashMap<&'a SchemaParameterName, Parameter<'a>>,
    pub query_params: Vec<Parameter<'a>>,
    pub header_params: Vec<Parameter<'a>>,
    pub cookie_params: Vec<Parameter<'a>>,
    pub request_body_or_ref: Option<RequestBodyOrReference<'a>>,
    pub request_responses: Responses<'a>,
}

#[derive(Debug, Default)]
pub struct Responses<'a> {
    pub default: Option<ResponseBodyOrReference<'a>>,
    pub codes: indexmap::IndexMap<&'a HttpStatusCode, ResponseBodyOrReference<'a>>,
}
