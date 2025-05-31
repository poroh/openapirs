// SPDX-License-Identifier: MIT
//
// OpenAPI Schema Path objects
//

use crate::schema::operation::Operation;
use crate::schema::parameter::ParameterOrReference;
use crate::schema::server::Server;
use crate::schema::SRef;
use crate::typing::TaggedString;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PathItem {
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub sref: Option<SRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<Summary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<ParameterOrReference>>,
}

pub type Summary = TaggedString<SummaryTag>;
pub enum SummaryTag {}

pub type Description = TaggedString<DescriptionTag>;
pub enum DescriptionTag {}

impl PathItem {
    pub fn operations_iter(&self) -> OperationIter {
        OperationIter {
            path_item: self,
            op_type: Some(&GET),
        }
    }
}

pub struct OperationIter<'a> {
    path_item: &'a PathItem,
    op_type: Option<&'static OperationType>,
}

#[derive(Debug)]
pub enum OperationType {
    Get,
    Put,
    Post,
    Delete,
    Options,
    Head,
    Patch,
    Trace,
}

static GET: OperationType = OperationType::Get;
static PUT: OperationType = OperationType::Put;
static POST: OperationType = OperationType::Post;
static DELETE: OperationType = OperationType::Delete;
static OPTIONS: OperationType = OperationType::Options;
static HEAD: OperationType = OperationType::Head;
static PATCH: OperationType = OperationType::Patch;
static TRACE: OperationType = OperationType::Trace;

impl<'a> Iterator for OperationIter<'a> {
    type Item = (&'static OperationType, &'a Operation);

    fn next(&mut self) -> Option<Self::Item> {
        self.op_type.and_then(|op| {
            match op {
                OperationType::Get => {
                    self.op_type = Some(&PUT);
                    &self.path_item.get
                }
                OperationType::Put => {
                    self.op_type = Some(&POST);
                    &self.path_item.put
                }
                OperationType::Post => {
                    self.op_type = Some(&DELETE);
                    &self.path_item.post
                }
                OperationType::Delete => {
                    self.op_type = Some(&OPTIONS);
                    &self.path_item.delete
                }
                OperationType::Options => {
                    self.op_type = Some(&HEAD);
                    &self.path_item.delete
                }
                OperationType::Head => {
                    self.op_type = Some(&PATCH);
                    &self.path_item.head
                }
                OperationType::Patch => {
                    self.op_type = Some(&TRACE);
                    &self.path_item.patch
                }
                OperationType::Trace => {
                    self.op_type = None;
                    &self.path_item.trace
                }
            }
            .as_ref()
            .map(|v| (op, v))
            .or_else(|| self.next())
        })
    }
}
