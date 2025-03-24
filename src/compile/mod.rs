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
use crate::schema::path::Path;
use crate::schema::path::PathParseError;
use crate::schema::path_item::OperationType;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Compiled<'a> {
    pub operations: Vec<Operation<'a>>,
}

#[derive(Debug)]
pub struct Operation<'a> {
    pub op_type: OperationType,
    pub path: &'a Path,
    pub path_params: HashMap<&'a parameter::Name, &'a Parameter>,
    //pub query_params: HashMap<&'a parameter::Name, &'a Parameter>,
}

#[derive(Debug)]
pub enum Error {
    PathParseError(PathParseError),
    ParameterNotDefined(Path, parameter::Name),
    NotDefinedAsPathParameter(Path, parameter::Name),
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
                        .map(|(op_type, op)| compile_operation(path, op_type, op, &d.components))
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
    op_type: OperationType,
    op: &'a schema::operation::Operation,
    components: &'a Option<Components>,
) -> Result<Operation<'a>, Error> {
    let path_params = path
        .path_params_iter()
        .map(|pname| {
            let name = pname
                .map_err(Error::PathParseError)
                .map(|v| parameter::Name::new(v.into()))?;
            resolve_parameter(op, &name, components)
                .ok_or(Error::ParameterNotDefined(path.clone(), name.clone()))
                .and_then(|p| match p.place {
                    parameter::Place::Path(_) => Ok(p),
                    _ => Err(Error::NotDefinedAsPathParameter(path.clone(), name.clone())),
                })
                .map(|v| (&v.name, v))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    Ok(Operation {
        op_type,
        path,
        path_params,
        // query_params,
    })
}

pub fn resolve_parameter<'a>(
    op: &'a schema::operation::Operation,
    pname: &parameter::Name,
    components: &'a Option<Components>,
) -> Option<&'a Parameter> {
    op.parameters.as_ref().and_then(|ps| {
        for p in ps.iter() {
            let candidate = match p {
                parameter::ParameterOrReference::Parameter(p) => Some(p),
                parameter::ParameterOrReference::Reference(sref) => sref
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
    })
}
