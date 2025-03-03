// SPDX-License-Identifier: MIT
//
// Content Type Value
//

use crate::typing::TaggedString;

// TODO: correct content type
pub type ContentType = TaggedString<ContentTypeTag>;
pub enum ContentTypeTag {}
