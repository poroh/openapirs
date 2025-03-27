// SPDX-License-Identifier: MIT
//
// Compilation of the openapi spec.
//
// This operation collects all operations
// required to be generated and all types those
// operations depends on.
//

use crate::schema;
use crate::schema::components::Components;
use crate::schema::parameter;
use crate::schema::parameter::Parameter;
use crate::schema::parameter::ParameterOrReference;
use crate::schema::path::Path;
use crate::schema::path::PathParseError;
use crate::schema::path_item::OperationType;
use crate::schema::path_item::PathItem;
use crate::schema::reference::Reference;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Compiled<'a> {
    pub operations: Vec<Operation<'a>>,
}

#[derive(Debug)]
pub struct Operation<'a> {
    pub op_type: OperationType,
    pub path: &'a Path,
    pub path_params: HashMap<&'a parameter::Name, &'a Parameter>,
    pub query_params: Vec<&'a Parameter>,
    pub header_params: Vec<&'a Parameter>,
    pub cookie_params: Vec<&'a Parameter>,
}

#[derive(Debug)]
pub enum Error<'a> {
    PathParseError(PathParseError),
    ParameterNotDefined(&'a Path, parameter::Name),
    NotDefinedAsPathParameter(&'a Path, parameter::Name),
    WrongParameterReference(&'a Path, &'a Reference),
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
                            compile_operation(
                                op_type,
                                &OpCompileData {
                                    path,
                                    item,
                                    op,
                                    components: &d.components,
                                },
                            )
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

fn compile_operation<'a>(
    op_type: OperationType,
    cdata: &OpCompileData<'a>,
) -> Result<Operation<'a>, Error<'a>> {
    let path_params = cdata
        .path
        .path_params_iter()
        .map(|pname| {
            let name = pname
                .map_err(Error::PathParseError)
                .map(|v| parameter::Name::new(v.into()))?;
            resolve_path_parameter(&name, cdata)
                .ok_or(Error::ParameterNotDefined(cdata.path, name.clone()))
                .and_then(|p| match p.place {
                    parameter::Place::Path(_) => Ok(p),
                    _ => Err(Error::NotDefinedAsPathParameter(cdata.path, name)),
                })
                .map(|v| (&v.name, v))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    Ok(Operation {
        op_type,
        path: cdata.path,
        path_params,
        query_params: compile_params_by_group(cdata, is_query_param)?,
        header_params: compile_params_by_group(cdata, is_header_param)?,
        cookie_params: compile_params_by_group(cdata, is_cookie_param)?,
    })
}

fn resolve_path_parameter<'a>(
    pname: &parameter::Name,
    cdata: &OpCompileData<'a>,
) -> Option<&'a Parameter> {
    let find_param = |ps: &'a Vec<ParameterOrReference>| {
        for p in ps.iter() {
            let candidate = match p {
                ParameterOrReference::Parameter(p) => Some(p),
                ParameterOrReference::Reference(r) => cdata.find_param_by_ref(r),
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
    cdata
        .op
        .parameters
        .as_ref()
        .and_then(find_param)
        .or_else(|| cdata.item.parameters.as_ref().and_then(find_param))
}

struct OpCompileData<'a> {
    path: &'a Path,
    item: &'a PathItem,
    op: &'a schema::operation::Operation,
    components: &'a Option<Components>,
}

impl<'a> OpCompileData<'a> {
    fn find_param_by_ref(&self, r: &Reference) -> Option<&'a Parameter> {
        r.sref.parameter_sref().as_ref().and_then(|sref| {
            self.components
                .as_ref()
                .and_then(|c| c.find_parameter(sref))
        })
    }
}

fn is_query_param(p: &Parameter) -> bool {
    matches!(
        p,
        Parameter {
            place: parameter::Place::Query(_),
            ..
        }
    )
}

fn is_header_param(p: &Parameter) -> bool {
    matches!(
        p,
        Parameter {
            place: parameter::Place::Header(_),
            ..
        }
    )
}

fn is_cookie_param(p: &Parameter) -> bool {
    matches!(
        p,
        Parameter {
            place: parameter::Place::Cookie(_),
            ..
        }
    )
}

type FilterFunc = fn(&Parameter) -> bool;

fn compile_params_by_group<'a>(
    cdata: &OpCompileData<'a>,
    filter: FilterFunc,
) -> Result<Vec<&'a Parameter>, Error<'a>> {
    let resolve_param = |p: &'a ParameterOrReference| match p {
        ParameterOrReference::Parameter(p) => Ok(p),
        ParameterOrReference::Reference(r) => cdata
            .find_param_by_ref(r)
            .ok_or(Error::WrongParameterReference(cdata.path, r)),
    };
    let filter_or_err = |v: &Result<&'a Parameter, _>| match v {
        Ok(p) => filter(p),
        Err(_) => true,
    };
    let op_params = cdata
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
    let path_params = cdata
        .item
        .parameters
        .as_ref()
        .map(|vec| {
            vec.iter()
                .map(resolve_param)
                .filter(filter_or_err)
                .filter(|v: &Result<&'a Parameter, _>| match v {
                    Err(_) => true,
                    Ok(Parameter { ref name, .. }) => !all_names.contains(&name),
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or(Ok(vec![]))?;

    Ok([op_params, path_params].concat())
}
