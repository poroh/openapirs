// SPDX-License-Identifier: MIT
//
// Compilation of the openapi spec.
//
// This operation collects all operations
// required to be generated and all types those
// operations depends on.
//

pub mod data_type;
pub mod operation;
pub mod schema_compiler;
pub mod stack;

use crate::compile::data_type::DataType;
use crate::compile::operation::request_body::RequestBody;
use crate::compile::operation::response_body::ResponseBody;
use crate::compile::operation::Operation;
use crate::compile::stack::Stack;
use crate::schema;
use crate::schema::sref::SRefRequestBody;
use crate::schema::sref::SRefResponsesName;
use crate::schema::sref::SRefSchemasObjectName;

pub type RequestBodies<'a> = indexmap::IndexMap<SRefRequestBody, RequestBody<'a>>;
pub type ResponseBodies<'a> = indexmap::IndexMap<SRefResponsesName, ResponseBody<'a>>;
pub type Schemas<'a> = indexmap::IndexMap<SRefSchemasObjectName, DataType<'a>>;

#[derive(Debug)]
pub struct Compiled<'a> {
    pub request_bodies: RequestBodies<'a>,
    pub response_bodies: ResponseBodies<'a>,
    pub schemas: Schemas<'a>,
    pub operations: Vec<Operation<'a>>,
}

type CResult<'a, T> = Result<T, operation::Error<'a>>;

pub fn compile(d: &schema::Description) -> CResult<Compiled> {
    let mut schema_chain = Stack::default();
    let mut request_bodies = RequestBodies::default();
    let mut response_bodies = ResponseBodies::default();
    let operations = d
        .paths
        .as_ref()
        .map(|paths| {
            Ok(paths
                .iter()
                .map(|(path, item)| -> CResult<Vec<Operation>> {
                    item.operations_iter()
                        .map(|(op_type, op)| {
                            let cdata = operation::CompileData {
                                path,
                                item,
                                op,
                                components: &d.components,
                                schema_chain: &schema_chain,
                                request_bodies: &request_bodies,
                                response_bodies: &response_bodies,
                            };
                            let opr = cdata.compile_operation(op_type)?;
                            schema_chain.merge(opr.schemas);
                            request_bodies.extend(opr.request_bodies);
                            response_bodies.extend(opr.response_bodies);
                            Ok(opr.op)
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect())
        })
        .unwrap_or(Ok(vec![]))?;
    Ok(Compiled {
        response_bodies,
        request_bodies,
        schemas: schema_chain.done(),
        operations,
    })
}
