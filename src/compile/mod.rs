// SPDX-License-Identifier: MIT
//
// Compilation of the openapi spec.
//
// This operation collects all operations
// required to be generated and all types those
// operations depends on.
//

pub mod compiler;
pub mod data_type;
pub mod schema_chain;

use crate::compile::data_type::TypeOrSchemaRef;
use crate::compile::schema_chain::SchemaChain;
use crate::schema;
use crate::schema::components::Components;
use crate::schema::http_status_code::HttpStatusCode;
use crate::schema::parameter;
use crate::schema::parameter::Parameter as SchemaParameter;
use crate::schema::parameter::ParameterOrReference;
use crate::schema::path::Path;
use crate::schema::path::PathParseError;
use crate::schema::path_item::OperationType;
use crate::schema::path_item::PathItem;
use crate::schema::reference::Reference;
use crate::schema::request_body::RequestBodyOrReference as SchemaRequestBodyOrReference;
use crate::schema::responses::ResponseOrReference as SchemaResponseOrReference;
use crate::schema::sref::SRefRequestBody;
use crate::schema::sref::SRefResponsesName;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::compile::schema_chain::CompiledSchemas;

pub type CompiledBodies<'a> = indexmap::IndexMap<SRefRequestBody, RequestBody<'a>>;
pub type CompiledResponses<'a> = indexmap::IndexMap<SRefResponsesName, RequestResponse<'a>>;

#[derive(Debug)]
pub struct Compiled<'a> {
    pub request_bodies: CompiledBodies<'a>,
    pub responses: CompiledResponses<'a>,
    pub schemas: CompiledSchemas<'a>,
    pub operations: Vec<Operation<'a>>,
}

#[derive(Debug)]
pub struct Parameter<'a> {
    pub schema_param: &'a SchemaParameter,
}

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
pub struct RequestResponse<'a> {
    pub json_type_or_ref: Option<TypeOrSchemaRef<'a>>,
}

#[derive(Debug)]
pub enum RequestResponseOrReference<'a> {
    Response(RequestResponse<'a>),
    Reference(SRefResponsesName),
}

#[derive(Debug)]
pub struct OperationResponses<'a> {
    pub default: Option<RequestResponseOrReference<'a>>,
    pub codes: indexmap::IndexMap<&'a HttpStatusCode, RequestResponseOrReference<'a>>,
}

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

#[derive(Debug)]
pub enum Error<'a> {
    PathParseError(PathParseError),
    ParameterNotDefined(&'a Path, parameter::Name),
    NotDefinedAsPathParameter(&'a Path, parameter::Name),
    WrongParameterReference(&'a Path, &'a Reference),
    WrongBodyReference(&'a Path, &'a Reference),
    WrongResponseReference(&'a Path, &'a Reference),
    BodyCompilation(&'a Path, OperationType, compiler::Error<'a>),
    ResponseCompilation(&'a Path, OperationType, compiler::Error<'a>),
    ResponseCodeCompilation(&'a Path, OperationType, &'a HttpStatusCode, Box<Error<'a>>),
}

type CResult<'a, T> = Result<T, Error<'a>>;

pub fn compile(d: &schema::Description) -> CResult<Compiled> {
    let mut schema_chain = SchemaChain::default();
    let mut request_bodies = CompiledBodies::default();
    let mut responses = CompiledResponses::default();
    let operations = d
        .paths
        .as_ref()
        .map(|paths| {
            Ok(paths
                .iter()
                .map(|(path, item)| -> CResult<Vec<Operation>> {
                    item.operations_iter()
                        .map(|(op_type, op)| {
                            let cdata = OpCompileData {
                                path,
                                item,
                                op,
                                components: &d.components,
                                schema_chain: &schema_chain,
                                request_bodies: &request_bodies,
                                responses: &responses,
                            };
                            let opr = cdata.compile_operation(op_type)?;
                            schema_chain.merge(opr.schemas);
                            request_bodies.extend(opr.request_bodies);
                            responses.extend(opr.responses);
                            Ok(opr.op)
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect())
        })
        .unwrap_or(Ok(vec![]))?;
    Ok(Compiled {
        responses,
        request_bodies,
        schemas: schema_chain.done(),
        operations,
    })
}

struct OpCompileData<'a, 'b> {
    path: &'a Path,
    item: &'a PathItem,
    op: &'a schema::operation::Operation,
    components: &'a Option<Components>,
    schema_chain: &'b SchemaChain<'a, 'b>,
    request_bodies: &'b CompiledBodies<'a>,
    responses: &'b CompiledResponses<'a>,
}

impl<'a, 'b> OpCompileData<'a, 'b> {
    fn find_param_by_ref(&self, r: &Reference) -> Option<&'a SchemaParameter> {
        r.sref.parameter_sref().as_ref().and_then(|sref| {
            self.components
                .as_ref()
                .and_then(|c| c.find_parameter(sref))
        })
    }

    fn resolve_path_parameter(&self, pname: &parameter::Name) -> Option<&'a SchemaParameter> {
        let find_param = |ps: &'a Vec<ParameterOrReference>| {
            for p in ps.iter() {
                let candidate = match p {
                    ParameterOrReference::Parameter(p) => Some(p),
                    ParameterOrReference::Reference(r) => self.find_param_by_ref(r),
                };
                if let Some(candidate) = candidate {
                    if &candidate.name == pname {
                        return Some(candidate);
                    }
                }
            }
            None
        };
        // Find parameter inside operation and after for whole path
        self.op
            .parameters
            .as_ref()
            .and_then(find_param)
            .or_else(|| self.item.parameters.as_ref().and_then(find_param))
    }

    fn compile_operation(&self, op_type: OperationType) -> Result<OpCompileResult<'a>, Error<'a>> {
        let mut chain = SchemaChain::new(self.schema_chain);
        let mut request_bodies = CompiledBodies::default();
        let mut responses = CompiledResponses::default();
        let path_params = self
            .path
            .path_params_iter()
            .map(|pname| {
                let name = pname
                    .map_err(Error::PathParseError)
                    .map(|v| parameter::Name::new(v.into()))?;
                self.resolve_path_parameter(&name)
                    .ok_or(Error::ParameterNotDefined(self.path, name.clone()))
                    .and_then(|p| match p.place {
                        parameter::Place::Path(_) => Ok(p),
                        _ => Err(Error::NotDefinedAsPathParameter(self.path, name)),
                    })
                    .map(|v| (&v.name, Parameter { schema_param: v }))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        let body_compile_result = self
            .op
            .request_body
            .as_ref()
            .map(|body| self.compile_body(op_type.clone(), body))
            .transpose()?;

        let request_body_or_ref = body_compile_result.map(|v| match v {
            BodyCompileResult::ExistingBody(sref) => RequestBodyOrReference::Reference(sref),
            BodyCompileResult::NewBody((sref, body, schemas)) => {
                request_bodies.insert(sref.clone(), body);
                chain.merge(schemas);
                RequestBodyOrReference::Reference(sref)
            }
            BodyCompileResult::DataType((body, schemas)) => {
                chain.merge(schemas);
                RequestBodyOrReference::Body(body)
            }
        });

        let request_responses = self
            .op
            .responses
            .as_ref()
            .map(|resps| {
                let default = resps
                    .default
                    .as_ref()
                    .map(|resp| self.compile_response(op_type.clone(), resp))
                    .transpose()?
                    .map(|resp| match resp {
                        ResponseCompileResult::ExistingResponse(sref) => {
                            RequestResponseOrReference::Reference(sref)
                        }
                        ResponseCompileResult::NewResponse((sref, resp, schemas)) => {
                            responses.insert(sref.clone(), resp);
                            chain.merge(schemas);
                            RequestResponseOrReference::Reference(sref)
                        }
                        ResponseCompileResult::DataType((resp, schemas)) => {
                            chain.merge(schemas);
                            RequestResponseOrReference::Response(resp)
                        }
                    });
                let codes = resps
                    .codes
                    .iter()
                    .map(|(code, resp)| {
                        Ok((
                            code,
                            self.compile_response(op_type.clone(), resp)
                                .map(|resp| match resp {
                                    ResponseCompileResult::ExistingResponse(sref) => {
                                        RequestResponseOrReference::Reference(sref)
                                    }
                                    ResponseCompileResult::NewResponse((sref, resp, schemas)) => {
                                        responses.insert(sref.clone(), resp);
                                        chain.merge(schemas);
                                        RequestResponseOrReference::Reference(sref)
                                    }
                                    ResponseCompileResult::DataType((resp, schemas)) => {
                                        chain.merge(schemas);
                                        RequestResponseOrReference::Response(resp)
                                    }
                                })
                                .map_err(|err| {
                                    Error::ResponseCodeCompilation(
                                        self.path,
                                        op_type.clone(),
                                        code,
                                        Box::new(err),
                                    )
                                })?,
                        ))
                    })
                    .collect::<Result<indexmap::IndexMap<_, _>, _>>()?;
                Ok(OperationResponses { default, codes })
            })
            .transpose()?;

        Ok(OpCompileResult {
            op: Operation {
                op_type,
                path: self.path,
                path_params,
                query_params: self.compile_params_by_group(is_query_param)?,
                header_params: self.compile_params_by_group(is_header_param)?,
                cookie_params: self.compile_params_by_group(is_cookie_param)?,
                request_body_or_ref,
                request_responses,
            },
            schemas: chain.done(),
            request_bodies,
            responses,
        })
    }

    fn compile_params_by_group(
        &self,
        filter: fn(&SchemaParameter) -> bool,
    ) -> Result<Vec<Parameter<'a>>, Error<'a>> {
        let resolve_param = |p: &'a ParameterOrReference| match p {
            ParameterOrReference::Parameter(p) => Ok(p),
            ParameterOrReference::Reference(r) => self
                .find_param_by_ref(r)
                .ok_or(Error::WrongParameterReference(self.path, r)),
        };
        let filter_or_err = |v: &Result<&'a SchemaParameter, _>| match v {
            Ok(p) => filter(p),
            Err(_) => true,
        };
        let op_params = self
            .op
            .parameters
            .as_ref()
            .map(|vec| {
                vec.iter()
                    .map(resolve_param)
                    .filter(filter_or_err)
                    .collect::<Result<Vec<_>, _>>()
            })
            .unwrap_or(Ok(vec![]))?;

        // Append all parameters from path that are not overriden by operation.
        let all_names = op_params.iter().map(|p| &p.name).collect::<HashSet<_>>();
        let path_params = self
            .item
            .parameters
            .as_ref()
            .map(|vec| {
                vec.iter()
                    .map(resolve_param)
                    .filter(filter_or_err)
                    .filter(|v: &Result<&'a SchemaParameter, _>| match v {
                        Err(_) => true,
                        Ok(SchemaParameter { ref name, .. }) => !all_names.contains(&name),
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .unwrap_or(Ok(vec![]))?;

        Ok([op_params, path_params]
            .concat()
            .into_iter()
            .map(|p| Parameter { schema_param: p })
            .collect::<Vec<_>>())
    }

    fn compile_body(
        &self,
        op_type: OperationType,
        sbody: &'a SchemaRequestBodyOrReference,
    ) -> Result<BodyCompileResult<'a>, Error<'a>> {
        match sbody {
            SchemaRequestBodyOrReference::RequestBody(b) => {
                let mut chain = SchemaChain::new(self.schema_chain);
                let json_type_or_ref =
                    compiler::compile_body_json(b, self.components.as_ref(), &chain)
                        .map_err(|err| Error::BodyCompilation(self.path, op_type.clone(), err))?
                        .map(|v| {
                            chain.merge(v.schemas);
                            v.type_or_ref
                        });
                let request_body = RequestBody { json_type_or_ref };
                Ok(BodyCompileResult::DataType((request_body, chain.done())))
            }
            SchemaRequestBodyOrReference::Reference(r) => {
                let body_sref = r
                    .sref
                    .request_body_sref()
                    .ok_or(Error::WrongBodyReference(self.path, r))?;
                if self.request_bodies.contains_key(&body_sref) {
                    // Already compiled:
                    Ok(BodyCompileResult::ExistingBody(body_sref))
                } else {
                    let mut chain = SchemaChain::new(self.schema_chain);
                    let components = self
                        .components
                        .as_ref()
                        .ok_or(Error::WrongBodyReference(self.path, r))?;
                    let body_schema = components
                        .find_request_body(&body_sref)
                        .ok_or(Error::WrongBodyReference(self.path, r))?;
                    let json_type_or_ref =
                        compiler::compile_body_json(body_schema, self.components.as_ref(), &chain)
                            .map_err(|err| Error::BodyCompilation(self.path, op_type.clone(), err))?
                            .map(|v| {
                                chain.merge(v.schemas);
                                v.type_or_ref
                            });
                    let request_body = RequestBody { json_type_or_ref };
                    Ok(BodyCompileResult::NewBody((
                        body_sref,
                        request_body,
                        chain.done(),
                    )))
                }
            }
        }
    }

    fn compile_response(
        &self,
        op_type: OperationType,
        sresp: &'a SchemaResponseOrReference,
    ) -> Result<ResponseCompileResult<'a>, Error<'a>> {
        match sresp {
            SchemaResponseOrReference::Response(b) => {
                let mut chain = SchemaChain::new(self.schema_chain);
                let json_type_or_ref =
                    compiler::compile_response_json(b, self.components.as_ref(), &chain)
                        .map_err(|err| Error::ResponseCompilation(self.path, op_type.clone(), err))?
                        .map(|v| {
                            chain.merge(v.schemas);
                            v.type_or_ref
                        });
                let reps = RequestResponse { json_type_or_ref };
                Ok(ResponseCompileResult::DataType((reps, chain.done())))
            }
            SchemaResponseOrReference::Reference(r) => {
                let resp_sref = r
                    .sref
                    .responses_sref()
                    .ok_or(Error::WrongResponseReference(self.path, r))?;
                if self.responses.contains_key(&resp_sref) {
                    // Already compiled:
                    Ok(ResponseCompileResult::ExistingResponse(resp_sref))
                } else {
                    let mut chain = SchemaChain::new(self.schema_chain);
                    let components = self
                        .components
                        .as_ref()
                        .ok_or(Error::WrongResponseReference(self.path, r))?;
                    let resp_schema = components
                        .find_response(&resp_sref)
                        .ok_or(Error::WrongResponseReference(self.path, r))?;
                    let json_type_or_ref = compiler::compile_response_json(
                        resp_schema,
                        self.components.as_ref(),
                        &chain,
                    )
                    .map_err(|err| Error::BodyCompilation(self.path, op_type.clone(), err))?
                    .map(|v| {
                        chain.merge(v.schemas);
                        v.type_or_ref
                    });
                    let response = RequestResponse { json_type_or_ref };
                    Ok(ResponseCompileResult::NewResponse((
                        resp_sref,
                        response,
                        chain.done(),
                    )))
                }
            }
        }
    }
}

enum BodyCompileResult<'a> {
    // Body specified as reference and has been alredy compiled
    ExistingBody(SRefRequestBody),
    // Body speicified as reference and compiled
    NewBody((SRefRequestBody, RequestBody<'a>, CompiledSchemas<'a>)),
    // Body specified in-place
    DataType((RequestBody<'a>, CompiledSchemas<'a>)),
}

enum ResponseCompileResult<'a> {
    // Body specified as reference and has been alredy compiled
    ExistingResponse(SRefResponsesName),
    // Body speicified as reference and compiled
    NewResponse((SRefResponsesName, RequestResponse<'a>, CompiledSchemas<'a>)),
    // Body specified in-place
    DataType((RequestResponse<'a>, CompiledSchemas<'a>)),
}

struct OpCompileResult<'a> {
    op: Operation<'a>,
    schemas: CompiledSchemas<'a>,
    request_bodies: CompiledBodies<'a>,
    responses: CompiledResponses<'a>,
}

fn is_query_param(p: &SchemaParameter) -> bool {
    matches!(
        p,
        SchemaParameter {
            place: parameter::Place::Query(_),
            ..
        }
    )
}

fn is_header_param(p: &SchemaParameter) -> bool {
    matches!(
        p,
        SchemaParameter {
            place: parameter::Place::Header(_),
            ..
        }
    )
}

fn is_cookie_param(p: &SchemaParameter) -> bool {
    matches!(
        p,
        SchemaParameter {
            place: parameter::Place::Cookie(_),
            ..
        }
    )
}
