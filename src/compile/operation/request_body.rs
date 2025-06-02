// SPDX-License-Identifier: MIT
//
// Compiled request body
//

use crate::compile::data_type::DataTypeWithSchema;
use crate::compile::data_type::TypeOrSchemaRef;
use crate::compile::schema_compiler;
use crate::compile::schema_compiler::Error as SchemaCompileError;
use crate::compile::stack::Stack;
use crate::compile::RequestBodies;
use crate::compile::Schemas;
use crate::schema::components::Components;
use crate::schema::reference::Reference as SchemaReference;
use crate::schema::request_body::RequestBody as SchemaRequestBody;
use crate::schema::request_body::RequestBodyOrReference as SchemaRequestBodyOrReference;
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

#[derive(Debug)]
pub enum Error<'a> {
    JsonCompile(SchemaCompileError<'a>),
    WrongReference(&'a SchemaReference),
}

pub struct CompileData<'a, 'b> {
    pub components: &'a Option<Components>,
    pub schema_chain: &'b Stack<'a, 'b>,
    pub request_bodies: &'b RequestBodies<'a>,
}

pub enum CompileResult<'a> {
    // Body specified as reference and has been alredy compiled
    Existing(SRefRequestBody),
    // Body speicified as reference and compiled
    New((SRefRequestBody, RequestBody<'a>, Schemas<'a>)),
    // Body specified in-place
    DataType((RequestBody<'a>, Schemas<'a>)),
}

impl<'a> CompileResult<'a> {
    pub fn aggregate<'b>(
        self,
        request_bodies: &mut RequestBodies<'a>,
        chain: &mut Stack<'a, 'b>,
    ) -> RequestBodyOrReference<'a> {
        match self {
            CompileResult::Existing(sref) => RequestBodyOrReference::Reference(sref),
            CompileResult::New((sref, body, schemas)) => {
                request_bodies.insert(sref.clone(), body);
                chain.merge(schemas);
                RequestBodyOrReference::Reference(sref)
            }
            CompileResult::DataType((body, schemas)) => {
                chain.merge(schemas);
                RequestBodyOrReference::Body(body)
            }
        }
    }
}

pub fn compile_body<'a, 'b>(
    data: CompileData<'a, 'b>,
    sbody: &'a SchemaRequestBodyOrReference,
) -> Result<CompileResult<'a>, Error<'a>> {
    match sbody {
        SchemaRequestBodyOrReference::RequestBody(b) => {
            let mut chain = Stack::new(data.schema_chain);
            let json_type_or_ref = compile_json(b, data.components.as_ref(), &chain)
                .map_err(Error::JsonCompile)?
                .map(|v| {
                    chain.merge(v.schemas);
                    v.type_or_ref
                });
            let request_body = RequestBody { json_type_or_ref };
            Ok(CompileResult::DataType((request_body, chain.done())))
        }
        SchemaRequestBodyOrReference::Reference(r) => {
            let body_sref = r.sref.request_body_sref().ok_or(Error::WrongReference(r))?;
            if data.request_bodies.contains_key(&body_sref) {
                // Already compiled:
                Ok(CompileResult::Existing(body_sref))
            } else {
                let mut chain = Stack::new(data.schema_chain);
                let components = data.components.as_ref().ok_or(Error::WrongReference(r))?;
                let body_schema = components
                    .find_request_body(&body_sref)
                    .ok_or(Error::WrongReference(r))?;
                let json_type_or_ref = compile_json(body_schema, data.components.as_ref(), &chain)
                    .map_err(Error::JsonCompile)?
                    .map(|v| {
                        chain.merge(v.schemas);
                        v.type_or_ref
                    });
                let request_body = RequestBody { json_type_or_ref };
                Ok(CompileResult::New((body_sref, request_body, chain.done())))
            }
        }
    }
}

fn compile_json<'a, 'b>(
    body: &'a SchemaRequestBody,
    components: Option<&'a Components>,
    chain: &'b Stack<'a, 'b>,
) -> Result<Option<DataTypeWithSchema<'a>>, SchemaCompileError<'a>> {
    body.content
        .get("application/json")
        .and_then(|json| json.schema.as_ref())
        .map(|json_schema| schema_compiler::compile(json_schema, components, chain, 0))
        .transpose()
}
