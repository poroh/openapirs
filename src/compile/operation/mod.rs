// SPDX-License-Identifier: MIT
//
// Compiled operation
//

pub mod request_body;
pub mod response_body;

use crate::compile::OperationResponses;
use crate::compile::Parameter;
use crate::schema::parameter;
use crate::schema::path::Path;
use crate::schema::path_item::OperationType;
use request_body::RequestBodyOrReference;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Operation<'a> {
    pub op_type: OperationType,
    pub path: &'a Path,
    pub path_params: HashMap<&'a parameter::Name, Parameter<'a>>,
    pub query_params: Vec<Parameter<'a>>,
    pub header_params: Vec<Parameter<'a>>,
    pub cookie_params: Vec<Parameter<'a>>,
    pub request_body_or_ref: Option<RequestBodyOrReference<'a>>,
    pub request_responses: Option<OperationResponses<'a>>,
}
