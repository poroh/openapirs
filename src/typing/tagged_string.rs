// SPDX-License-Identifier: MIT
//
// Strong-typed string
//

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

pub struct TaggedString<Tag> {
    v: String,
    _marker: std::marker::PhantomData<Tag>,
}

impl<Tag> TaggedString<Tag> {
    pub fn new(v: String) -> Self {
        Self {
            v,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<Tag> PartialEq for TaggedString<Tag> {
    fn eq(&self, other: &Self) -> bool {
        self.v.eq(&other.v)
    }
}

impl<Tag> Eq for TaggedString<Tag> {}

impl<Tag> std::hash::Hash for TaggedString<Tag> {
    fn hash<H>(&self, h: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.v.hash(h)
    }
}

impl<Tag> std::fmt::Debug for TaggedString<Tag> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.v.fmt(f)
    }
}

impl<'de, Tag> Deserialize<'de> for TaggedString<Tag> {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TaggedTypeVisitor<Tag> {
            _marker: std::marker::PhantomData<Tag>,
        }

        impl<'de, Tag> Visitor<'de> for TaggedTypeVisitor<Tag> {
            type Value = TaggedString<Tag>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("String type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(TaggedString::<Tag>::new(value.into()))
            }
        }

        de.deserialize_string(TaggedTypeVisitor::<Tag> {
            _marker: std::marker::PhantomData,
        })
    }
}
