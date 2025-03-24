// SPDX-License-Identifier: MIT
//
// OpenAPI Schema Path objects
//

use crate::schema::operation::Operation;
use crate::schema::parameter::ParameterOrReference;
use crate::schema::server::Server;
use crate::schema::Sref;
use crate::typing::TaggedString;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PathItem {
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub sref: Option<Sref>,
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
            op_type: Some(OperationType::Get),
        }
    }
}

pub struct OperationIter<'a> {
    path_item: &'a PathItem,
    op_type: Option<OperationType>,
}

#[derive(Clone, Debug)]
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

impl<'a> Iterator for OperationIter<'a> {
    type Item = (OperationType, &'a Operation);

    fn next(&mut self) -> Option<Self::Item> {
        self.op_type.clone().and_then(|op| {
            match op {
                OperationType::Get => {
                    self.op_type = Some(OperationType::Put);
                    &self.path_item.get
                }
                OperationType::Put => {
                    self.op_type = Some(OperationType::Post);
                    &self.path_item.put
                }
                OperationType::Post => {
                    self.op_type = Some(OperationType::Delete);
                    &self.path_item.post
                }
                OperationType::Delete => {
                    self.op_type = Some(OperationType::Options);
                    &self.path_item.delete
                }
                OperationType::Options => {
                    self.op_type = Some(OperationType::Head);
                    &self.path_item.delete
                }
                OperationType::Head => {
                    self.op_type = Some(OperationType::Patch);
                    &self.path_item.head
                }
                OperationType::Patch => {
                    self.op_type = Some(OperationType::Trace);
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
