// SPDX-License-Identifier: MIT
//
// Compiled response body
//

use crate::compile::data_type::DataTypeWithSchema;
use crate::compile::data_type::TypeOrSchemaRef;
use crate::compile::schema_chain::SchemaChain;
use crate::compile::schema_chain::Schemas;
use crate::compile::schema_compiler;
use crate::compile::schema_compiler::Error as SchemaCompileError;
use crate::compile::ResponseBodies;
use crate::schema::components::Components;
use crate::schema::reference::Reference as SchemaReference;
use crate::schema::response::Response as SchemaResponse;
use crate::schema::responses::ResponseOrReference as SchemaResponseOrReference;
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
) -> Result<Option<DataTypeWithSchema<'a>>, SchemaCompileError<'a>> {
    resp.content
        .as_ref()
        .and_then(|content| content.get("application/json"))
        .and_then(|json| json.schema.as_ref())
        .map(|json_schema| schema_compiler::compile(json_schema, components, chain, 0))
        .transpose()
}

pub struct CompileData<'a, 'b> {
    pub components: &'a Option<Components>,
    pub schema_chain: &'b SchemaChain<'a, 'b>,
    pub response_bodies: &'b ResponseBodies<'a>,
}

pub enum CompileResult<'a> {
    // Body specified as reference and has been alredy compiled
    Existing(SRefResponsesName),
    // Body speicified as reference and compiled
    New((SRefResponsesName, ResponseBody<'a>, Schemas<'a>)),
    // Body specified in-place
    DataType((ResponseBody<'a>, Schemas<'a>)),
}

#[derive(Debug)]
pub enum Error<'a> {
    JsonCompile(SchemaCompileError<'a>),
    WrongReference(&'a SchemaReference),
}

pub fn compile_response<'a, 'b>(
    cdata: CompileData<'a, 'b>,
    sresp: &'a SchemaResponseOrReference,
) -> Result<CompileResult<'a>, Error<'a>> {
    match sresp {
        SchemaResponseOrReference::Response(b) => {
            let mut chain = SchemaChain::new(cdata.schema_chain);
            let json_type_or_ref = compile_json(b, cdata.components.as_ref(), &chain)
                .map_err(Error::JsonCompile)?
                .map(|v| {
                    chain.merge(v.schemas);
                    v.type_or_ref
                });
            let reps = ResponseBody { json_type_or_ref };
            Ok(CompileResult::DataType((reps, chain.done())))
        }
        SchemaResponseOrReference::Reference(r) => {
            let resp_sref = r.sref.responses_sref().ok_or(Error::WrongReference(r))?;
            if cdata.response_bodies.contains_key(&resp_sref) {
                // Already compiled:
                Ok(CompileResult::Existing(resp_sref))
            } else {
                let mut chain = SchemaChain::new(cdata.schema_chain);
                let components = cdata.components.as_ref().ok_or(Error::WrongReference(r))?;
                let resp_schema = components
                    .find_response(&resp_sref)
                    .ok_or(Error::WrongReference(r))?;
                let json_type_or_ref = compile_json(resp_schema, cdata.components.as_ref(), &chain)
                    .map_err(Error::JsonCompile)?
                    .map(|v| {
                        chain.merge(v.schemas);
                        v.type_or_ref
                    });
                let response = ResponseBody { json_type_or_ref };
                Ok(CompileResult::New((resp_sref, response, chain.done())))
            }
        }
    }
}
