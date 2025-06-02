// SPDX-License-Identifier: MIT
//
// Compiled operation
//

pub mod parameter;
pub mod request_body;
pub mod response_body;

use crate::compile::stack::Stack;
use crate::compile::RequestBodies;
use crate::compile::ResponseBodies;
use crate::compile::Schemas;
use crate::schema::components::Components;
use crate::schema::http_status_code::HttpStatusCode;
use crate::schema::operation::Operation as SchemaOperation;
use crate::schema::parameter::Name as SchemaParameterName;
use crate::schema::parameter::Parameter as SchemaParameter;
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

#[derive(Debug)]
pub struct Operation<'a> {
    pub op_type: &'static OperationType,
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
    PathParameter(&'a Path, SchemaParameterName, parameter::Error<'a>),
    QueryParameter(&'a Path, parameter::Error<'a>),
    HeaderParameter(&'a Path, parameter::Error<'a>),
    CookieParameter(&'a Path, parameter::Error<'a>),
    PathParseError(&'a Path, PathParseError),
    RequestBodyCompile(&'a Path, &'static OperationType, request_body::Error<'a>),
    ResponseBodyCompile(&'a Path, &'static OperationType, response_body::Error<'a>),
    WrongParameterReference(&'a Path, &'a SchemaReference),
    ResponseCodeCompilation(
        &'a Path,
        &'static OperationType,
        &'a HttpStatusCode,
        Box<Error<'a>>,
    ),
}

pub struct CompileResult<'a> {
    pub op: Operation<'a>,
    pub schemas: Schemas<'a>,
    pub request_bodies: RequestBodies<'a>,
    pub response_bodies: ResponseBodies<'a>,
}

pub struct CompileData<'a, 'b> {
    pub path: &'a Path,
    pub item: &'a PathItem,
    pub op: &'a SchemaOperation,
    pub components: &'a Option<Components>,
    pub schema_chain: &'b Stack<'a, 'b>,
    pub request_bodies: &'b RequestBodies<'a>,
    pub response_bodies: &'b ResponseBodies<'a>,
}

impl<'a, 'b> CompileData<'a, 'b> {
    pub fn compile_operation(
        &self,
        op_type: &'static OperationType,
    ) -> Result<CompileResult<'a>, Error<'a>> {
        let mut chain = Stack::new(self.schema_chain);
        let mut request_bodies = RequestBodies::default();
        let mut response_bodies = ResponseBodies::default();

        let parameter_compile = parameter::CompileData {
            op_parameters: &self.op.parameters,
            item_parameters: &self.item.parameters,
            components: self.components,
        };
        let path_params = self
            .path
            .path_params_iter()
            .map(|pname| {
                let name = pname
                    .map_err(|err| Error::PathParseError(self.path, err))
                    .map(|v| SchemaParameterName::new(v.into()))?;
                parameter_compile
                    .compile_path_parameter(&name)
                    .map_err(|err| Error::PathParameter(self.path, name.clone(), err))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        let request_body_or_ref = self
            .op
            .request_body
            .as_ref()
            .map(|body| self.compile_body(op_type, body))
            .transpose()?
            .map(|cbody| cbody.aggregate(&mut request_bodies, &mut chain));

        let responses = self
            .op
            .responses
            .as_ref()
            .map(|resps| {
                let default = resps
                    .default
                    .as_ref()
                    .map(|resp| self.compile_response(op_type, resp))
                    .transpose()?
                    .map(|resp| resp.aggregate(&mut response_bodies, &mut chain));
                let codes = resps
                    .codes
                    .iter()
                    .map(|(code, resp)| {
                        Ok((
                            code,
                            self.compile_response(op_type, resp)
                                .map(|resp| resp.aggregate(&mut response_bodies, &mut chain))
                                .map_err(|err| {
                                    Error::ResponseCodeCompilation(
                                        self.path,
                                        op_type,
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

        Ok(CompileResult {
            op: Operation {
                op_type,
                path: self.path,
                path_params,
                query_params: parameter_compile
                    .compile_params_by_group(SchemaParameter::is_query)
                    .map_err(|err| Error::QueryParameter(self.path, err))?,
                header_params: parameter_compile
                    .compile_params_by_group(SchemaParameter::is_header)
                    .map_err(|err| Error::HeaderParameter(self.path, err))?,
                cookie_params: parameter_compile
                    .compile_params_by_group(SchemaParameter::is_cookie)
                    .map_err(|err| Error::CookieParameter(self.path, err))?,
                request_body_or_ref,
                request_responses: responses.unwrap_or_default(),
            },
            schemas: chain.done(),
            request_bodies,
            response_bodies,
        })
    }

    fn compile_body(
        &self,
        op_type: &'static OperationType,
        sbody: &'a SchemaRequestBodyOrReference,
    ) -> Result<BodyCompileResult<'a>, Error<'a>> {
        let cdata = request_body::CompileData {
            components: self.components,
            schema_chain: self.schema_chain,
            request_bodies: self.request_bodies,
        };
        request_body::compile_body(cdata, sbody)
            .map_err(|err| Error::RequestBodyCompile(self.path, op_type, err))
    }

    fn compile_response(
        &self,
        op_type: &'static OperationType,
        sresp: &'a SchemaResponseOrReference,
    ) -> Result<ResponseCompileResult<'a>, Error<'a>> {
        let cdata = response_body::CompileData {
            components: self.components,
            schema_chain: self.schema_chain,
            response_bodies: self.response_bodies,
        };
        response_body::compile_response(cdata, sresp)
            .map_err(|err| Error::ResponseBodyCompile(self.path, op_type, err))
    }
}
