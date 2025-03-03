// SPDX-License-Identifier: MIT
//
// Strong-typed URL
//

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

pub struct TaggedURI<Tag> {
    v: String,
    _marker: std::marker::PhantomData<Tag>,
}

impl<'a, Tag> TaggedURI<Tag> {
    fn from_uri(uri: uriparse::URI<'_>) -> Self {
        Self {
            v: uri.to_string(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<Tag> std::fmt::Debug for TaggedURI<Tag> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.v.fmt(f)
    }
}

impl<'de, Tag> Deserialize<'de> for TaggedURI<Tag> {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TaggedTypeVisitor<Tag> {
            _marker: std::marker::PhantomData<Tag>,
        }

        impl<'de, Tag> Visitor<'de> for TaggedTypeVisitor<Tag> {
            type Value = TaggedURI<Tag>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("MUST be in the form of a URI")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let uri = uriparse::URI::try_from(value).map_err(de::Error::custom)?;
                Ok(TaggedURI::<Tag>::from_uri(uri))
            }
        }

        de.deserialize_string(TaggedTypeVisitor::<Tag> {
            _marker: std::marker::PhantomData,
        })
    }
}
