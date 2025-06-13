// SPDX-License-Identifier: MIT
//
// Name of the model
//

use crate::schema::path::Path;
use crate::schema::sref::SRefSchemasObjectName;
use crate::schema::PropertyName;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Name<'a> {
    Schemas(&'a SRefSchemasObjectName),
    Property(&'a PropertyName, Box<&'a Name<'a>>),
    RequestBody(&'a Path),
}
