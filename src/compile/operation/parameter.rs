// SPDX-License-Identifier: MIT
//
// Compiled parameter
//

use crate::schema::parameter::Parameter as SchemaParameter;

#[derive(Debug)]
pub struct Parameter<'a> {
    pub schema_param: &'a SchemaParameter,
}
