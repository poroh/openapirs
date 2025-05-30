// SPDX-License-Identifier: MIT
//
// Compiled request body
//

use crate::compile::data_type::TypeOrSchemaRef;
use crate::schema::sref::SRefRequestBody;

#[derive(Debug)]
pub struct RequestBody<'a> {
    pub json_type_or_ref: Option<TypeOrSchemaRef<'a>>,
}

#[derive(Debug)]
pub enum RequestBodyOrReference<'a> {
    Body(RequestBody<'a>),
    Reference(SRefRequestBody),
}
