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
}

#[derive(Debug)]
pub enum Error {
    PathParseError(PathParseError),
    ParameterNotDefined(Path, parameter::Name),
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
    let mut path_params = HashMap::new();
    for v in path.path_params_iter() {
        let name = parameter::Name::new(v.map_err(Error::PathParseError)?.into());
        op.parameters
            .as_ref()
            .and_then(|ps| {
                ps.iter().find(|p| match p {
                    parameter::ParameterOrReference::Parameter(p) => match p.place {
                        parameter::Place::Path(_) => {
                            path_params.insert(&p.name, p);
                            true
                        }
                        _ => false,
                    },
                    parameter::ParameterOrReference::Reference(_) => false,
                })
            })
            .ok_or(Error::ParameterNotDefined(path.clone(), name))?;
    }
    Ok(Operation {
        op_type,
        path,
        path_params,
    })
}
