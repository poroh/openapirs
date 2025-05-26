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

pub type CompiledSchemas<'a> = indexmap::IndexMap<SRefSchemasObjectName, DataType<'a>>;

// Lifetime 'a is lifetime of parsed schema object.
// Lifetime 'b is lifetime of schemas.
#[derive(Default)]
pub struct SchemaChain<'a, 'b> {
    pub parent: Option<&'b SchemaChain<'a, 'b>>,
    pub current: CompiledSchemas<'a>,
}

impl<'a, 'b> SchemaChain<'a, 'b> {
    pub fn new(parent: &'b SchemaChain<'a, 'b>) -> Self {
        Self {
            parent: Some(parent),
            current: indexmap::IndexMap::default(),
        }
    }

    pub fn contains(&self, v: &SRefSchemasObjectName) -> bool {
        self.current.contains_key(v) || self.parent.map(|p| p.contains(v)).unwrap_or(false)
    }

    pub fn merge(&mut self, v: CompiledSchemas<'a>) {
        self.current.extend(v);
    }

    pub fn done(self) -> CompiledSchemas<'a> {
        self.current
    }
}
