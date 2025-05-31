// SPDX-License-Identifier: MIT
//
// Compiled schema chain.  Chain means that we have current-level
// compiled schemas and optionally parent level compiled schema.
//
// The purpose of this design is to avoid passing mutable collection
// of schemas to the level below in recursion but still pass
// everything that is compiled on previous stages.
//

use crate::compile::data_type::DataType;
use crate::schema::sref::SRefSchemasObjectName;

pub type Schemas<'a> = indexmap::IndexMap<SRefSchemasObjectName, DataType<'a>>;

// Lifetime 'a is lifetime of parsed schema object.
// Lifetime 'b is lifetime of schemas.
#[derive(Default)]
pub struct SchemaChain<'a, 'b> {
    pub sref: Option<&'b SRefSchemasObjectName>,
    pub parent: Option<&'b SchemaChain<'a, 'b>>,
    pub current: Schemas<'a>,
}

impl<'a, 'b> SchemaChain<'a, 'b> {
    pub fn new(parent: &'b SchemaChain<'a, 'b>) -> Self {
        Self {
            sref: None,
            parent: Some(parent),
            current: indexmap::IndexMap::default(),
        }
    }

    pub fn new_ref(parent: &'b SchemaChain<'a, 'b>, sref: &'b SRefSchemasObjectName) -> Self {
        Self {
            sref: Some(sref),
            parent: Some(parent),
            current: indexmap::IndexMap::default(),
        }
    }

    pub fn contains(&self, v: &SRefSchemasObjectName) -> bool {
        self.sref == Some(v)
            || self.current.contains_key(v)
            || self.parent.map(|p| p.contains(v)).unwrap_or(false)
    }

    pub fn merge(&mut self, v: Schemas<'a>) {
        self.current.extend(v);
    }

    pub fn done(self) -> Schemas<'a> {
        self.current
    }
}
