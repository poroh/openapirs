// SPDX-License-Identifier: MIT
//
// Compiler of opeanpirs schema
//

use crate::compile::data_type::ActualType;
use crate::compile::data_type::CompiledArray;
use crate::compile::data_type::CompiledObject;
use crate::compile::data_type::CompiledType;
use crate::compile::data_type::DataType;
use crate::compile::data_type::NormalCompiledType;
use crate::compile::data_type::NullableCompiledType;
use crate::compile::data_type::TypeOrSchemaRef;
use crate::compile::schema_chain::CompiledSchemas;
use crate::compile::schema_chain::SchemaChain;
use crate::schema::components::Components;
use crate::schema::data_type::array::Array as SchemaArray;
use crate::schema::data_type::object::Object as SchemaObject;
use crate::schema::data_type::ActualType as SchemaActualType;
use crate::schema::data_type::DataType as SchemaDataType;
use crate::schema::data_type::MaybeNullableTypeSchema;
use crate::schema::data_type::NullableTypeSchema;
use crate::schema::data_type::TypeSchema;
use crate::schema::reference::Reference;
use crate::schema::request_body::RequestBody as SchemaRequestBody;
use crate::schema::sref::SRef;
use crate::schema::sref::SRefSchemas;
use crate::schema::sref::SRefSchemasObjectName;
use crate::schema::PropertyName;
use indexmap::indexmap;

const MAX_DEPTH: u32 = 1024;

#[derive(Debug)]
pub enum Error<'a> {
    UnexpecetedReferenceType(&'a SRef),
    SchemasNotDefinedButReferenced,
    SchemaRefernceNotFound(SRefSchemasObjectName),
    SchemaCompilation(SRefSchemasObjectName, Box<Error<'a>>),
    PropertyCompilation(&'a PropertyName, Box<Error<'a>>),
    ReferenceError(crate::schema::sref::Error),
    MaxDepthReached(u32),
    NoItemsInArray,
    ArrayItemCompilation(Box<Error<'a>>),
    ReferenceToUncompatibleObject(SRefSchemas),
    PropertiesNotFoundInReferencedObject(SRefSchemasObjectName),
    PropertyNotFoundInReferencedObject((SRefSchemasObjectName, PropertyName)),
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
        SchemaDataType::Reference(r) => compile_ref(r, components, chain, depth),
        SchemaDataType::ActualType(at) => match &at.type_schema {
            MaybeNullableTypeSchema::Nullable(dt) => {
                compile_nullable_actual_type(at, &dt.schema, components, chain, depth + 1)
            }
            MaybeNullableTypeSchema::Normal(dt) => compile_normal_actual_type(at, dt),
            MaybeNullableTypeSchema::Object(obj) => {
                compile_normal_object(obj, components, chain, depth + 1)
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

pub fn compile_ref<'a, 'b>(
    r: &'a Reference,
    components: Option<&'a Components>,
    chain: &'b SchemaChain<'a, 'b>,
    depth: u32,
) -> Result<DataTypeWithSchema<'a>, Error<'a>> {
    let schemas_ref = r
        .sref
        .schemas_sref()
        .map_err(Error::ReferenceError)?
        .ok_or(Error::UnexpecetedReferenceType(&r.sref))?;
    match schemas_ref {
        SRefSchemas::Normal(schemas_name) => {
            if chain.contains(&schemas_name) {
                // If schema has been already compiled just refer to it
                Ok(DataTypeWithSchema {
                    type_or_ref: TypeOrSchemaRef::Reference(schemas_name),
                    schemas: CompiledSchemas::default(),
                })
            } else {
                // If schema has not been compiled:
                let schema = components
                    .ok_or(Error::SchemasNotDefinedButReferenced)?
                    .find_schema_by_name(&schemas_name)
                    .ok_or(Error::SchemaRefernceNotFound(schemas_name.clone()))?;
                let compiled_schema = compile(schema, components, chain, depth + 1)
                    .map_err(|err| Error::SchemaCompilation(schemas_name.clone(), Box::new(err)))?;
                match compiled_schema.type_or_ref {
                    TypeOrSchemaRef::DataType(dt) => Ok(DataTypeWithSchema {
                        type_or_ref: TypeOrSchemaRef::Reference(schemas_name.clone()),
                        schemas: indexmap! {
                            schemas_name => dt
                        },
                    }),
                    TypeOrSchemaRef::Reference(sref) => {
                        // reference to reference. In this case we just
                        // follow further reference
                        Ok(DataTypeWithSchema {
                            type_or_ref: TypeOrSchemaRef::Reference(sref),
                            schemas: compiled_schema.schemas,
                        })
                    }
                }
            }
        }
        SRefSchemas::ObjProperty((ref schemas_name, ref pname)) => {
            // Rare: reference directly to property
            let schema = components
                .ok_or(Error::SchemasNotDefinedButReferenced)?
                .find_schema_by_name(schemas_name)
                .ok_or(Error::SchemaRefernceNotFound(schemas_name.clone()))?;
            match schema {
                SchemaDataType::ActualType(at) => {
                    let obj = match at.type_schema {
                        MaybeNullableTypeSchema::Nullable(ref dt) => match dt.schema {
                            NullableTypeSchema::Object(ref obj) => Ok(obj),
                            _ => Err(Error::ReferenceToUncompatibleObject(schemas_ref.clone())),
                        },
                        MaybeNullableTypeSchema::Normal(_) => {
                            Err(Error::ReferenceToUncompatibleObject(schemas_ref.clone()))
                        }
                        MaybeNullableTypeSchema::Object(ref obj) => Ok(obj),
                        MaybeNullableTypeSchema::Array(_) => {
                            Err(Error::ReferenceToUncompatibleObject(schemas_ref.clone()))
                        }
                    }?;
                    let dt = obj
                        .properties
                        .as_ref()
                        .ok_or(Error::PropertiesNotFoundInReferencedObject(
                            schemas_name.clone(),
                        ))
                        .and_then(|properties| {
                            properties
                                .get(pname)
                                .ok_or(Error::PropertyNotFoundInReferencedObject((
                                    schemas_name.clone(),
                                    pname.clone(),
                                )))
                        })?;
                    compile(dt, components, chain, depth + 1)
                }
                _ => Err(Error::ReferenceToUncompatibleObject(schemas_ref)),
            }
        }
    }
}

pub fn compile_nullable_actual_type<'a, 'b>(
    at: &'a SchemaActualType,
    dt: &'a NullableTypeSchema,
    components: Option<&'a Components>,
    parent_chain: &'b SchemaChain<'a, 'b>,
    depth: u32,
) -> Result<DataTypeWithSchema<'a>, Error<'a>> {
    Ok(match dt {
        NullableTypeSchema::Null => {
            todo!()
        }
        NullableTypeSchema::Boolean(v) => DataTypeWithSchema::actual_type(
            at,
            CompiledType::Nullable(NullableCompiledType::Boolean(v)),
        ),
        NullableTypeSchema::Integer(v) => DataTypeWithSchema::actual_type(
            at,
            CompiledType::Nullable(NullableCompiledType::Integer(v)),
        ),
        NullableTypeSchema::String(v) => DataTypeWithSchema::actual_type(
            at,
            CompiledType::Nullable(NullableCompiledType::String(v)),
        ),
        NullableTypeSchema::Number(v) => DataTypeWithSchema::actual_type(
            at,
            CompiledType::Nullable(NullableCompiledType::Number(v)),
        ),
        NullableTypeSchema::Object(v) => {
            compile_nullable_object(v, components, parent_chain, depth)?
        }
        NullableTypeSchema::Array(v) => compile_nullable_array(v, components, parent_chain, depth)?,
    })
}

pub fn compile_normal_actual_type<'a>(
    at: &'a SchemaActualType,
    dt: &'a TypeSchema,
) -> Result<DataTypeWithSchema<'a>, Error<'a>> {
    Ok(DataTypeWithSchema {
        schemas: CompiledSchemas::default(),
        type_or_ref: TypeOrSchemaRef::DataType(DataType::ActualType(ActualType {
            readonly: at.readonly,
            writeonly: at.writeonly,
            compiled_type: match dt {
                TypeSchema::Null => {
                    todo!()
                }
                TypeSchema::Boolean(v) => CompiledType::Normal(NormalCompiledType::Boolean(v)),
                TypeSchema::Integer(v) => CompiledType::Normal(NormalCompiledType::Integer(v)),
                TypeSchema::String(v) => CompiledType::Normal(NormalCompiledType::String(v)),
                TypeSchema::Number(v) => CompiledType::Normal(NormalCompiledType::Number(v)),
            },
        })),
    })
}

fn compile_object<'a, 'b>(
    sobj: &'a SchemaObject,
    components: Option<&'a Components>,
    parent_chain: &'b SchemaChain<'a, 'b>,
    depth: u32,
) -> Result<(CompiledObject<'a>, CompiledSchemas<'a>), Error<'a>> {
    let mut result = CompiledObject::default();
    let mut chain = SchemaChain::new(parent_chain);
    if let Some(properties) = sobj.properties.as_ref() {
        for (propname, sprop) in properties.iter() {
            let cresult = compile(sprop, components, &chain, depth + 1)
                .map_err(|err| Error::PropertyCompilation(propname, Box::new(err)))?;
            chain.merge(cresult.schemas);
            result
                .properties
                .insert(propname.clone(), cresult.type_or_ref);
        }
    }
    Ok((result, chain.done()))
}

pub fn compile_normal_object<'a, 'b>(
    sobj: &'a SchemaObject,
    components: Option<&'a Components>,
    parent_chain: &'b SchemaChain<'a, 'b>,
    depth: u32,
) -> Result<DataTypeWithSchema<'a>, Error<'a>> {
    let (obj, schemas) = compile_object(sobj, components, parent_chain, depth)?;
    Ok(DataTypeWithSchema {
        schemas,
        type_or_ref: TypeOrSchemaRef::DataType(DataType::ActualType(ActualType {
            compiled_type: CompiledType::Normal(NormalCompiledType::Object(obj)),
            readonly: false,
            writeonly: false,
        })),
    })
}

pub fn compile_nullable_object<'a, 'b>(
    sobj: &'a SchemaObject,
    components: Option<&'a Components>,
    parent_chain: &'b SchemaChain<'a, 'b>,
    depth: u32,
) -> Result<DataTypeWithSchema<'a>, Error<'a>> {
    let (obj, schemas) = compile_object(sobj, components, parent_chain, depth)?;
    Ok(DataTypeWithSchema {
        schemas,
        type_or_ref: TypeOrSchemaRef::DataType(DataType::ActualType(ActualType {
            compiled_type: CompiledType::Nullable(NullableCompiledType::Object(obj)),
            readonly: false,
            writeonly: false,
        })),
    })
}

fn compile_array<'a, 'b>(
    sarr: &'a SchemaArray,
    components: Option<&'a Components>,
    parent_chain: &'b SchemaChain<'a, 'b>,
    depth: u32,
) -> Result<(CompiledArray<'a>, CompiledSchemas<'a>), Error<'a>> {
    let mut chain = SchemaChain::new(parent_chain);
    let sitems = sarr.items.as_ref().ok_or(Error::NoItemsInArray)?;
    let cresult = compile(sitems, components, &chain, depth + 1)
        .map_err(|err| Error::ArrayItemCompilation(Box::new(err)))?;
    chain.merge(cresult.schemas);
    Ok((
        CompiledArray {
            items: Box::new(cresult.type_or_ref),
        },
        chain.done(),
    ))
}

pub fn compile_nullable_array<'a, 'b>(
    sarr: &'a SchemaArray,
    components: Option<&'a Components>,
    parent_chain: &'b SchemaChain<'a, 'b>,
    depth: u32,
) -> Result<DataTypeWithSchema<'a>, Error<'a>> {
    let (arr, schemas) = compile_array(sarr, components, parent_chain, depth)?;
    Ok(DataTypeWithSchema {
        schemas,
        type_or_ref: TypeOrSchemaRef::DataType(DataType::ActualType(ActualType {
            compiled_type: CompiledType::Nullable(NullableCompiledType::Array(arr)),
            readonly: false,
            writeonly: false,
        })),
    })
}

#[derive(Debug)]
pub struct DataTypeWithSchema<'a> {
    pub schemas: CompiledSchemas<'a>,
    pub type_or_ref: TypeOrSchemaRef<'a>,
}

impl<'a> DataTypeWithSchema<'a> {
    pub fn actual_type(at: &'a SchemaActualType, compiled_type: CompiledType<'a>) -> Self {
        Self {
            schemas: CompiledSchemas::default(),
            type_or_ref: TypeOrSchemaRef::DataType(DataType::ActualType(ActualType {
                readonly: at.readonly,
                writeonly: at.writeonly,
                compiled_type,
            })),
        }
    }
}
