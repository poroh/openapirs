// SPDX-License-Identifier: MIT
//
// Compiled operation
//

pub mod parameter;
pub mod request_body;
pub mod response_body;

use crate::compile::schema_chain::SchemaChain;
use crate::compile::schema_chain::Schemas;
use crate::compile::schema_compiler;
use crate::compile::RequestBodies;
use crate::compile::ResponseBodies;
use crate::schema::components::Components;
use crate::schema::http_status_code::HttpStatusCode;
use crate::schema::operation::Operation as SchemaOperation;
use crate::schema::parameter::Name as SchemaParameterName;
use crate::schema::parameter::Parameter as SchemaParameter;
use crate::schema::parameter::ParameterOrReference;
use crate::schema::parameter::Place as SchemaParameterPlace;
use crate::schema::path::Path;
use crate::schema::path::PathParseError;
use crate::schema::path_item::OperationType;
use crate::schema::path_item::PathItem;
use crate::schema::reference::Reference as SchemaReference;
use crate::schema::request_body::RequestBodyOrReference as SchemaRequestBodyOrReference;
use crate::schema::responses::ResponseOrReference as SchemaResponseOrReference;
use parameter::Parameter;
use request_body::CompileResult as BodyCompileResult;
use request_body::RequestBodyOrReference;
use response_body::CompileResult as ResponseCompileResult;
use response_body::ResponseBodyOrReference;
use std::collections::HashMap;
use std::collections::HashSet;

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

#[derive(Debug)]
pub enum Error<'a> {
    PathParseError(PathParseError),
    ParameterNotDefined(&'a Path, SchemaParameterName),
    NotDefinedAsPathParameter(&'a Path, SchemaParameterName),
    RequestBodyCompile(&'a Path, OperationType, request_body::Error<'a>),
    ResponseBodyCompile(&'a Path, OperationType, response_body::Error<'a>),
    WrongParameterReference(&'a Path, &'a SchemaReference),
    WrongResponseReference(&'a Path, &'a SchemaReference),
    ResponseCompilation(&'a Path, OperationType, schema_compiler::Error<'a>),
    ResponseCodeCompilation(&'a Path, OperationType, &'a HttpStatusCode, Box<Error<'a>>),
}

pub struct OpCompileResult<'a> {
    pub op: Operation<'a>,
    pub schemas: Schemas<'a>,
    pub request_bodies: RequestBodies<'a>,
    pub response_bodies: ResponseBodies<'a>,
}

pub struct OpCompileData<'a, 'b> {
    pub path: &'a Path,
    pub item: &'a PathItem,
    pub op: &'a SchemaOperation,
    pub components: &'a Option<Components>,
    pub schema_chain: &'b SchemaChain<'a, 'b>,
    pub request_bodies: &'b RequestBodies<'a>,
    pub response_bodies: &'b ResponseBodies<'a>,
}

impl<'a, 'b> OpCompileData<'a, 'b> {
    fn find_param_by_ref(&self, r: &SchemaReference) -> Option<&'a SchemaParameter> {
        r.sref.parameter_sref().as_ref().and_then(|sref| {
            self.components
                .as_ref()
                .and_then(|c| c.find_parameter(sref))
        })
    }

    fn resolve_path_parameter(&self, pname: &SchemaParameterName) -> Option<&'a SchemaParameter> {
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

    pub fn compile_operation(
        &self,
        op_type: OperationType,
    ) -> Result<OpCompileResult<'a>, Error<'a>> {
        let mut chain = SchemaChain::new(self.schema_chain);
        let mut request_bodies = RequestBodies::default();
        let mut response_bodies = ResponseBodies::default();
        let path_params = self
            .path
            .path_params_iter()
            .map(|pname| {
                let name = pname
                    .map_err(Error::PathParseError)
                    .map(|v| SchemaParameterName::new(v.into()))?;
                self.resolve_path_parameter(&name)
                    .ok_or(Error::ParameterNotDefined(self.path, name.clone()))
                    .and_then(|p| match p.place {
                        SchemaParameterPlace::Path(_) => Ok(p),
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
            BodyCompileResult::Existing(sref) => RequestBodyOrReference::Reference(sref),
            BodyCompileResult::New((sref, body, schemas)) => {
                request_bodies.insert(sref.clone(), body);
                chain.merge(schemas);
                RequestBodyOrReference::Reference(sref)
            }
            BodyCompileResult::DataType((body, schemas)) => {
                chain.merge(schemas);
                RequestBodyOrReference::Body(body)
            }
        });

        let responses = self
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
                        ResponseCompileResult::Existing(sref) => {
                            ResponseBodyOrReference::Reference(sref)
                        }
                        ResponseCompileResult::New((sref, resp, schemas)) => {
                            response_bodies.insert(sref.clone(), resp);
                            chain.merge(schemas);
                            ResponseBodyOrReference::Reference(sref)
                        }
                        ResponseCompileResult::DataType((resp, schemas)) => {
                            chain.merge(schemas);
                            ResponseBodyOrReference::Body(resp)
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
                                    ResponseCompileResult::Existing(sref) => {
                                        ResponseBodyOrReference::Reference(sref)
                                    }
                                    ResponseCompileResult::New((sref, resp, schemas)) => {
                                        response_bodies.insert(sref.clone(), resp);
                                        chain.merge(schemas);
                                        ResponseBodyOrReference::Reference(sref)
                                    }
                                    ResponseCompileResult::DataType((resp, schemas)) => {
                                        chain.merge(schemas);
                                        ResponseBodyOrReference::Body(resp)
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
                Ok(Responses { default, codes })
            })
            .transpose()?;

        Ok(OpCompileResult {
            op: Operation {
                op_type,
                path: self.path,
                path_params,
                query_params: self.compile_params_by_group(SchemaParameter::is_query)?,
                header_params: self.compile_params_by_group(SchemaParameter::is_header)?,
                cookie_params: self.compile_params_by_group(SchemaParameter::is_cookie)?,
                request_body_or_ref,
                request_responses: responses.unwrap_or_default(),
            },
            schemas: chain.done(),
            request_bodies,
            response_bodies,
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
        let cdata = request_body::CompileData {
            components: self.components,
            schema_chain: self.schema_chain,
            request_bodies: self.request_bodies,
        };
        request_body::compile_body(cdata, sbody)
            .map_err(|err| Error::RequestBodyCompile(self.path, op_type.clone(), err))
    }

    fn compile_response(
        &self,
        op_type: OperationType,
        sresp: &'a SchemaResponseOrReference,
    ) -> Result<ResponseCompileResult<'a>, Error<'a>> {
        let cdata = response_body::CompileData {
            components: self.components,
            schema_chain: self.schema_chain,
            response_bodies: self.response_bodies,
        };
        response_body::compile_response(cdata, sresp)
            .map_err(|err| Error::ResponseBodyCompile(self.path, op_type.clone(), err))
    }
}
