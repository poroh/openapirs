// SPDX-License-Identifier: MIT
//
// Compiled response body
//

use crate::compile::data_type::TypeOrSchemaRef;
use crate::schema::sref::SRefResponsesName;

#[derive(Debug)]
pub struct ResponseBody<'a> {
    pub json_type_or_ref: Option<TypeOrSchemaRef<'a>>,
}

#[derive(Debug)]
pub enum ResponseBodyOrReference<'a> {
    Body(ResponseBody<'a>),
    Reference(SRefResponsesName),
}
