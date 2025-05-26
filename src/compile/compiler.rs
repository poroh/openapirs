// SPDX-License-Identifier: MIT
//
// Compiler of opeanpirs schema
//


use crate::compile::data_type::CompiledObject;
use crate::compile::data_type::TypeOrRef;
use crate::compile::data_type::ActualType;
use crate::compile::data_type::DataType;
use crate::compile::data_type::CompiledType;
use crate::compile::data_type::NormalCompiledType;
use crate::compile::data_type::NullableCompiledType;
use crate::compile::schema_chain::SchemaChain;
use crate::compile::schema_chain::CompiledSchemas;
use crate::schema::PropertyName;
use crate::schema::components::Components;
use crate::schema::data_type::TypeSchema;
use crate::schema::data_type::NullableTypeSchema;
use crate::schema::data_type::ActualType as SchemaActualType;
use crate::schema::data_type::object::Object as SchemaObject;
use crate::schema::data_type::DataType as SchemaDataType;
use crate::schema::data_type::MaybeNullableTypeSchema;
use crate::schema::request_body::RequestBody as SchemaRequestBody;
use crate::schema::sref::SRef;
use crate::schema::sref::SRefSchemas;
use indexmap::indexmap;

const MAX_DEPTH: u32 = 1024;

#[derive(Debug)]
pub enum Error<'a> {
    UnexpecetedReferenceType(&'a SRef),
    SchemasNotDefinedButReferenced,
    SchemaRefernceNotFound(SRefSchemas),
    SchemaCompilation(SRefSchemas, Box<Error<'a>>),
    PropertyCompilation(&'a PropertyName, Box<Error<'a>>),
    MaxDepthReached(u32),
}

pub fn compile_body_json<'a, 'b>(
    body: &'a SchemaRequestBody,
    components: Option<&'a Components>,
    chain: &'b SchemaChain<'a, 'b>,
) -> Result<Option<DataTypeWithSchema<'a>>, Error<'a>> {
    body.content
        .get("application/json")
        .and_then(|json| json.schema.as_ref())
        .map(|json_schema| compile(json_schema, components, chain, 0))
        .transpose()
}

pub fn compile<'a, 'b>(
    sdt: &'a SchemaDataType,
    components: Option<&'a Components>,
    chain: &'b SchemaChain<'a, 'b>,
    depth: u32,
) -> Result<DataTypeWithSchema<'a>, Error<'a>> {
    if depth > MAX_DEPTH {
        return Err(Error::MaxDepthReached(depth));
    }
    match sdt {
        SchemaDataType::Reference(r) => {
            let schemas_ref = r
                .sref
                .schemas_sref()
                .ok_or(Error::UnexpecetedReferenceType(&r.sref))?;
            if chain.contains(&schemas_ref) {
                // If schema has been already compiled just refer to it
                Ok(DataTypeWithSchema {
                    type_or_ref: TypeOrRef::Reference(schemas_ref),
                    schemas: CompiledSchemas::default(),
                })
            } else {
                // If schema has not been compiled:
                let schemas = components
                    .and_then(|components| components.schemas.as_ref())
                    .ok_or(Error::SchemasNotDefinedButReferenced)?;
                let schema = schemas
                    .get(&schemas_ref)
                    .ok_or(Error::SchemaRefernceNotFound(schemas_ref.clone()))?;
                let compiled_schema = compile(schema, components, chain, depth + 1)
                    .map_err(|err| {
                        Error::SchemaCompilation(schemas_ref.clone(), Box::new(err))
                    })?;
                match compiled_schema.type_or_ref {
                    TypeOrRef::ActualType(dt) => Ok(DataTypeWithSchema {
                        type_or_ref: TypeOrRef::Reference(schemas_ref.clone()),
                        schemas: indexmap! {
                            schemas_ref => dt
                        },
                    }),
                    TypeOrRef::Reference(sref) => {
                        // reference to reference. In this case we just
                        // follow further reference
                        Ok(DataTypeWithSchema {
                            type_or_ref: TypeOrRef::Reference(sref),
                            schemas: compiled_schema.schemas,
                        })
                    }
                }

            }
        }
        SchemaDataType::ActualType(at) => match &at.type_schema {
            MaybeNullableTypeSchema::Nullable(dt) => {
                compile_nullable_actual_type(at, &dt.schema)
            }
            MaybeNullableTypeSchema::Normal(dt) => {
                compile_normal_actual_type(at, dt)
            }
            MaybeNullableTypeSchema::Object(obj) => {
                compile_object(obj, components, chain, depth + 1)
            }
            MaybeNullableTypeSchema::Array(_) => {
                todo!()
            }
        },
        SchemaDataType::OneOf(_) => todo!(),
        SchemaDataType::AllOf(_) => todo!(),
        SchemaDataType::AnyOf(_) => todo!(),
        SchemaDataType::Empty(_) => {
            todo!()
        }
        SchemaDataType::UnknownType(_) => todo!(),
    }
}

pub fn compile_nullable_actual_type<'a>(at: &'a SchemaActualType, dt: &'a NullableTypeSchema) -> Result<DataTypeWithSchema<'a>, Error<'a>> {
    Ok(DataTypeWithSchema {
        schemas: CompiledSchemas::default(),
        type_or_ref: TypeOrRef::ActualType(DataType::ActualType(ActualType {
            readonly: at.readonly,
            writeonly: at.writeonly,
            compiled_type: match dt {
                NullableTypeSchema::Null => {
                    todo!()
                }
                NullableTypeSchema::Boolean(v) => {
                    CompiledType::Nullable(NullableCompiledType::Boolean(v))
                }
                NullableTypeSchema::Integer(v) => {
                    CompiledType::Nullable(NullableCompiledType::Integer(v))
                }
                NullableTypeSchema::String(v) => {
                    CompiledType::Nullable(NullableCompiledType::String(v))
                }
                NullableTypeSchema::Number(v) => {
                    CompiledType::Nullable(NullableCompiledType::Number(v))
                }
                NullableTypeSchema::Object(_) => {
                    todo!()
                }
                NullableTypeSchema::Array(_) => {
                    todo!()
                }
            }
        }))
    })
}

pub fn compile_normal_actual_type<'a>(at: &'a SchemaActualType, dt: &'a TypeSchema) -> Result<DataTypeWithSchema<'a>, Error<'a>> {
    Ok(DataTypeWithSchema {
        schemas: CompiledSchemas::default(),
        type_or_ref: TypeOrRef::ActualType(DataType::ActualType(ActualType {
            readonly: at.readonly,
            writeonly: at.writeonly,
            compiled_type: match dt {
                TypeSchema::Null => {
                    todo!()
                }
                TypeSchema::Boolean(v) => {
                    CompiledType::Normal(NormalCompiledType::Boolean(v))
                }
                TypeSchema::Integer(v) => {
                    CompiledType::Normal(NormalCompiledType::Integer(v))
                }
                TypeSchema::String(v) => {
                    CompiledType::Normal(NormalCompiledType::String(v))
                }
                TypeSchema::Number(v) => {
                    CompiledType::Normal(NormalCompiledType::Number(v))
                }
            }
        }))
    })
}

pub fn compile_object<'a, 'b>(
    sobj: &'a SchemaObject,
    components: Option<&'a Components>,
    parent_chain: &'b SchemaChain<'a, 'b>,
    depth: u32,
) -> Result<DataTypeWithSchema<'a>, Error<'a>> {
    let mut result = CompiledObject::default();
    let mut chain = SchemaChain::new(parent_chain);
    if let Some(properties) = sobj.properties.as_ref() { 
        for (propname, sprop) in properties.iter() {
            let cresult = compile(sprop, components, &chain, depth + 1).map_err(|err| {
                Error::PropertyCompilation(propname, Box::new(err))
            })?;
            chain.merge(cresult.schemas);
            result.properties.insert(propname.clone(), cresult.type_or_ref);
        }
    }
    Ok(DataTypeWithSchema {
        schemas: chain.done(),
        type_or_ref: TypeOrRef::ActualType(DataType::ActualType(ActualType {
            compiled_type: CompiledType::Normal(NormalCompiledType::Object(result)),
            readonly: false,
            writeonly: false,
        }))
    })
}

#[derive(Debug)]
pub struct DataTypeWithSchema<'a> {
    pub schemas: CompiledSchemas<'a>,
    pub type_or_ref: TypeOrRef<'a>,
}
