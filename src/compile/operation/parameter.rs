// SPDX-License-Identifier: MIT
//
// Compiled parameter
//

use crate::schema::components::Components;
use crate::schema::parameter::Name as SchemaParameterName;
use crate::schema::parameter::Parameter as SchemaParameter;
use crate::schema::parameter::ParameterOrReference;
use crate::schema::parameter::Place as SchemaParameterPlace;
use crate::schema::reference::Reference as SchemaReference;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Parameter<'a> {
    pub schema_param: &'a SchemaParameter,
}

#[derive(Debug)]
pub enum Error<'a> {
    WrongParameterReference(&'a SchemaReference),
    PathParameterNotDefined,
    NotDefinedAsPathParameter,
}

pub struct CompileData<'a> {
    pub op_parameters: &'a Option<Vec<ParameterOrReference>>,
    pub item_parameters: &'a Option<Vec<ParameterOrReference>>,
    pub components: &'a Option<Components>,
}

impl<'a> CompileData<'a> {
    pub fn compile_path_parameter<'b>(
        &self,
        name: &'b SchemaParameterName,
    ) -> Result<(&'a SchemaParameterName, Parameter<'a>), Error<'a>> {
        self.resolve_path_parameter(name)
            .ok_or(Error::PathParameterNotDefined)
            .and_then(|p| match p.place {
                SchemaParameterPlace::Path(_) => Ok(p),
                _ => Err(Error::NotDefinedAsPathParameter),
            })
            .map(|v| (&v.name, Parameter { schema_param: v }))
    }
    pub fn find_param_by_ref(&self, r: &SchemaReference) -> Option<&'a SchemaParameter> {
        r.sref.parameter_sref().as_ref().and_then(|sref| {
            self.components
                .as_ref()
                .and_then(|c| c.find_parameter(sref))
        })
    }

    pub fn resolve_path_parameter<'b>(
        &self,
        pname: &'b SchemaParameterName,
    ) -> Option<&'a SchemaParameter> {
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
        self.op_parameters
            .as_ref()
            .and_then(find_param)
            .or_else(|| self.item_parameters.as_ref().and_then(find_param))
    }

    pub fn compile_params_by_group(
        &self,
        filter: fn(&SchemaParameter) -> bool,
    ) -> Result<Vec<Parameter<'a>>, Error<'a>> {
        let resolve_param = |p: &'a ParameterOrReference| match p {
            ParameterOrReference::Parameter(p) => Ok(p),
            ParameterOrReference::Reference(r) => self
                .find_param_by_ref(r)
                .ok_or(Error::WrongParameterReference(r)),
        };
        let filter_or_err = |v: &Result<&'a SchemaParameter, _>| match v {
            Ok(p) => filter(p),
            Err(_) => true,
        };
        let op_params = self
            .op_parameters
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
            .item_parameters
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
