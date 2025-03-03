// SPDX-License-Identifier: MIT
//
// Strong typing helpers
//

pub mod tagged_string;
pub type TaggedString<T> = tagged_string::TaggedString<T>;

pub mod tagged_uri;
pub type TaggedURI<T> = tagged_uri::TaggedURI<T>;

pub mod always_true;
pub type AlwaysTrue = always_true::AlwaysTrue;
