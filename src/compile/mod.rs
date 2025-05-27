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
use crate::schema::sref::SRefRequestBody;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::compile::schema_chain::CompiledSchemas;

pub type CompiledBodies<'a> = indexmap::IndexMap<SRefRequestBody, RequestBody<'a>>;

#[derive(Debug)]
pub struct Compiled<'a> {
    pub request_bodies: CompiledBodies<'a>,
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
pub struct Responses<'a> {
    pub default: Option<Response<'a>>,
    pub codes: indexmap::IndexMap<&'a HttpStatusCode, Response<'a>>,
}

#[derive(Debug)]
pub struct Response<'a> {
    pub json_type_or_ref: Option<TypeOrSchemaRef<'a>>,
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
    // TODO:
    // pub responses: Option<Responses<'a>>,
}

#[derive(Debug)]
pub enum Error<'a> {
    PathParseError(PathParseError),
    ParameterNotDefined(&'a Path, parameter::Name),
    NotDefinedAsPathParameter(&'a Path, parameter::Name),
    WrongParameterReference(&'a Path, &'a Reference),
    WrongBodyReference(&'a Path, &'a Reference),
    BodyCompilation(&'a Path, OperationType, compiler::Error<'a>),
}

type CResult<'a, T> = Result<T, Error<'a>>;

pub fn compile(d: &schema::Description) -> CResult<Compiled> {
    let mut schema_chain = SchemaChain::default();
    let mut request_bodies = CompiledBodies::default();
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
                            };
                            let opr = cdata.compile_operation(op_type)?;
                            schema_chain.merge(opr.schemas);
                            request_bodies.extend(opr.request_bodies);
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

        // TODO:
        // let responses = self
        //     .op
        //     .responses
        //     .as_ref()
        //     .map(|resps| {
        //         println!("{resps:?}");
        //         todo!()
        //     })
        //     .transpose()?;

        Ok(OpCompileResult {
            op: Operation {
                op_type,
                path: self.path,
                path_params,
                query_params: self.compile_params_by_group(is_query_param)?,
                header_params: self.compile_params_by_group(is_header_param)?,
                cookie_params: self.compile_params_by_group(is_cookie_param)?,
                request_body_or_ref,
            },
            schemas: chain.done(),
            request_bodies,
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
}

enum BodyCompileResult<'a> {
    // Body specified as reference and has been alredy compiled
    ExistingBody(SRefRequestBody),
    // Body speicified as reference and compiled
    NewBody((SRefRequestBody, RequestBody<'a>, CompiledSchemas<'a>)),
    // Body specified in-place
    DataType((RequestBody<'a>, CompiledSchemas<'a>)),
}

struct OpCompileResult<'a> {
    op: Operation<'a>,
    schemas: CompiledSchemas<'a>,
    request_bodies: CompiledBodies<'a>,
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
