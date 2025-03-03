// SPDX-License-Identifier: MIT
//
// Server Object
//

use crate::typing::TaggedString;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Server {
    pub url: ServerURI,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descriptor: Option<ServerDescription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<ServerVariables>,
}

// TODO: String with variable substitution:
pub type ServerURI = TaggedString<ServerURITag>;
pub enum ServerURITag {}

pub type ServerDescription = TaggedString<ServerDescriptionTag>;
pub enum ServerDescriptionTag {}

pub type ServerVariables = indexmap::IndexMap<ServerVariableName, ServerVariable>;

pub type ServerVariableName = TaggedString<ServerVariableNameTag>;
pub enum ServerVariableNameTag {}

#[derive(Deserialize, Debug)]
pub struct ServerVariable {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<ServerVariableValue>>,
    pub default: ServerVariableValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<ServerVariableDescription>,
}

pub type ServerVariableDescription = TaggedString<ServerVariableDescriptionTag>;
pub enum ServerVariableDescriptionTag {}

pub type ServerVariableValue = TaggedString<ServerVariableValueTag>;
pub enum ServerVariableValueTag {}
