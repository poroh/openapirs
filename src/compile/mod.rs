// SPDX-License-Identifier: MIT
//
// Compilation of the openapi spec.
//
// This operation collects all operations
// required to be generated and all types those
// operations depends on.
//

pub mod data_type;
pub mod model;

use crate::schema;
use crate::schema::components::Components;
use crate::schema::parameter;
use crate::schema::parameter::Parameter as SchemaParameter;
use crate::schema::parameter::ParameterOrReference;
use crate::schema::path::Path;
use crate::schema::path::PathParseError;
use crate::schema::path_item::OperationType;
use crate::schema::path_item::PathItem;
use crate::schema::reference::Reference;
use crate::schema::request_body::RequestBody as SchemaRequestBody;
use crate::schema::request_body::RequestBodyOrReference;
use data_type::DataType;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Compiled<'a> {
    pub operations: Vec<Operation<'a>>,
}

#[derive(Debug)]
pub struct Parameter<'a> {
    pub schema_param: &'a SchemaParameter,
}

#[derive(Debug)]
pub struct RequestBody<'a> {
    pub schema_body: &'a SchemaRequestBody,
    pub json_model: Option<model::Name>,
}

#[derive(Debug)]
pub struct Operation<'a> {
    pub op_type: OperationType,
    pub path: &'a Path,
    pub path_params: HashMap<&'a parameter::Name, Parameter<'a>>,
    pub query_params: Vec<Parameter<'a>>,
    pub header_params: Vec<Parameter<'a>>,
    pub cookie_params: Vec<Parameter<'a>>,
    pub request_body: Option<RequestBody<'a>>,
}

#[derive(Debug)]
pub enum Error<'a> {
    PathParseError(PathParseError),
    ParameterNotDefined(&'a Path, parameter::Name),
    NotDefinedAsPathParameter(&'a Path, parameter::Name),
    WrongParameterReference(&'a Path, &'a Reference),
    WrongBodyReference(&'a Path, &'a Reference),
}

type CResult<'a, T> = Result<T, Error<'a>>;

pub fn compile(d: &schema::Description) -> CResult<Compiled> {
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
                            };
                            cdata.compile_operation(op_type)
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect())
        })
        .unwrap_or(Ok(vec![]))?;
    Ok(Compiled { operations })
}

struct OpCompileData<'a> {
    path: &'a Path,
    item: &'a PathItem,
    op: &'a schema::operation::Operation,
    components: &'a Option<Components>,
}

impl<'a> OpCompileData<'a> {
    fn find_param_by_ref(&self, r: &Reference) -> Option<&'a SchemaParameter> {
        r.sref.parameter_sref().as_ref().and_then(|sref| {
            self.components
                .as_ref()
                .and_then(|c| c.find_parameter(sref))
        })
    }

    fn find_request_body_by_ref(&self, r: &Reference) -> Option<&'a SchemaRequestBody> {
        r.sref.request_body_sref().as_ref().and_then(|sref| {
            self.components
                .as_ref()
                .and_then(|c| c.find_request_body(sref))
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

    fn compile_operation(&self, op_type: OperationType) -> Result<Operation<'a>, Error<'a>> {
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

        let request_body = self
            .op
            .request_body
            .as_ref()
            .map(|v| match v {
                RequestBodyOrReference::RequestBody(b) => Ok(b),
                RequestBodyOrReference::Reference(r) => self
                    .find_request_body_by_ref(r)
                    .ok_or(Error::WrongBodyReference(self.path, r)),
            })
            .transpose()?
            .map(|schema_body| {
                let json = DataType::resolve_body_json(schema_body, self.components.as_ref())?;
                Ok(RequestBody { schema_body, json })
            })
            .transpose()?;

        Ok(Operation {
            op_type,
            path: self.path,
            path_params,
            query_params: self.compile_params_by_group(is_query_param)?,
            header_params: self.compile_params_by_group(is_header_param)?,
            cookie_params: self.compile_params_by_group(is_cookie_param)?,
            request_body,
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
