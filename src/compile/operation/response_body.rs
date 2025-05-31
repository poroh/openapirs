// SPDX-License-Identifier: MIT
//
// Compiled response body
//

use crate::compile::data_type::DataTypeWithSchema;
use crate::compile::data_type::TypeOrSchemaRef;
use crate::compile::schema_chain::SchemaChain;
use crate::compile::schema_compiler;
use crate::compile::schema_compiler::Error;
use crate::schema::components::Components;
use crate::schema::response::Response as SchemaResponse;
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

pub fn compile_json<'a, 'b>(
    resp: &'a SchemaResponse,
    components: Option<&'a Components>,
    chain: &'b SchemaChain<'a, 'b>,
) -> Result<Option<DataTypeWithSchema<'a>>, Error<'a>> {
    resp.content
        .as_ref()
        .and_then(|content| content.get("application/json"))
        .and_then(|json| json.schema.as_ref())
        .map(|json_schema| schema_compiler::compile(json_schema, components, chain, 0))
        .transpose()
}
