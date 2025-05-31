// SPDX-License-Identifier: MIT
//
// Compiled request body
//

use crate::compile::data_type::DataTypeWithSchema;
use crate::compile::data_type::TypeOrSchemaRef;
use crate::compile::schema_chain::SchemaChain;
use crate::compile::schema_compiler;
use crate::compile::schema_compiler::Error;
use crate::schema::components::Components;
use crate::schema::request_body::RequestBody as SchemaRequestBody;
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

pub fn compile_json<'a, 'b>(
    body: &'a SchemaRequestBody,
    components: Option<&'a Components>,
    chain: &'b SchemaChain<'a, 'b>,
) -> Result<Option<DataTypeWithSchema<'a>>, Error<'a>> {
    body.content
        .get("application/json")
        .and_then(|json| json.schema.as_ref())
        .map(|json_schema| schema_compiler::compile(json_schema, components, chain, 0))
        .transpose()
}
