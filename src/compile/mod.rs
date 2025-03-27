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
}

#[derive(Debug)]
pub enum Error<'a> {
    PathParseError(PathParseError),
    ParameterNotDefined(&'a Path, parameter::Name),
    NotDefinedAsPathParameter(&'a Path, parameter::Name),
    WrongParameterReference(&'a Path, &'a Reference),
}

pub fn compile(d: &schema::Description) -> Result<Compiled, Error> {
    let operations = d
        .paths
        .as_ref()
        .map(|paths| {
            Ok(paths
                .iter()
                .map(|(path, item)| -> Result<Vec<Operation>, Error> {
                    item.operations_iter()
                        .map(|(op_type, op)| {
                            compile_operation(path, item, op_type, op, &d.components)
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

pub fn compile_operation<'a>(
    path: &'a schema::path::Path,
    item: &'a PathItem,
    op_type: OperationType,
    op: &'a schema::operation::Operation,
    components: &'a Option<Components>,
) -> Result<Operation<'a>, Error<'a>> {
    let path_params = path
        .path_params_iter()
        .map(|pname| {
            let name = pname
                .map_err(Error::PathParseError)
                .map(|v| parameter::Name::new(v.into()))?;
            resolve_parameter(op, &name, item, components)
                .ok_or(Error::ParameterNotDefined(path, name.clone()))
                .and_then(|p| match p.place {
                    parameter::Place::Path(_) => Ok(p),
                    _ => Err(Error::NotDefinedAsPathParameter(path, name)),
                })
                .map(|v| (&v.name, v))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    let resolve_param = |p: &'a ParameterOrReference| match p {
        ParameterOrReference::Parameter(p) => Ok(p),
        ParameterOrReference::Reference(sref) => sref
            .sref
            .parameter_sref()
            .and_then(|sref| components.as_ref().and_then(|c| c.find_parameter(&sref)))
            .ok_or(Error::WrongParameterReference(path, sref)),
    };
    let filter_err_or_query = |v: &Result<&'a Parameter, _>| {
        matches!(
            v,
            Ok(Parameter {
                place: parameter::Place::Query(_),
                ..
            }) | Err(_)
        )
    };
    let op_query_params = op
        .parameters
        .as_ref()
        .map(|vec| {
            vec.iter()
                .map(resolve_param)
                .filter(filter_err_or_query)
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or(Ok(vec![]))?;

    // Append all parameters from path that are not overriden by operation.
    let all_query_names = op_query_params
        .iter()
        .map(|p| &p.name)
        .collect::<HashSet<_>>();
    let path_query_params = item
        .parameters
        .as_ref()
        .map(|vec| {
            vec.iter()
                .map(resolve_param)
                .filter(filter_err_or_query)
                .filter(|v: &Result<&'a Parameter, _>| match v {
                    Err(_) => true,
                    Ok(Parameter { ref name, .. }) => !all_query_names.contains(&name),
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or(Ok(vec![]))?;

    Ok(Operation {
        op_type,
        path,
        path_params,
        query_params: [op_query_params, path_query_params].concat(),
    })
}

pub fn resolve_parameter<'a>(
    op: &'a schema::operation::Operation,
    pname: &parameter::Name,
    item: &'a PathItem,
    components: &'a Option<Components>,
) -> Option<&'a Parameter> {
    let find_param = |ps: &'a Vec<ParameterOrReference>| {
        for p in ps.iter() {
            let candidate = match p {
                ParameterOrReference::Parameter(p) => Some(p),
                ParameterOrReference::Reference(sref) => sref
                    .sref
                    .parameter_sref()
                    .as_ref()
                    .and_then(|sref| components.as_ref().and_then(|c| c.find_parameter(sref))),
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
    op.parameters
        .as_ref()
        .and_then(find_param)
        .or_else(|| item.parameters.as_ref().and_then(find_param))
}
