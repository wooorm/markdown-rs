//! HTML syntax tree: [hast][].
//!
//! Port of `hast.rs` from `mdxjs-rs`, with these changes: no MDX nodes; a
//! `Raw` node; `Cow` strings for tag names, property names, and text, so
//! constants do not allocate; and serde behind the `serde` feature, in the
//! shape of hast’s JSON.
//! It leaves out `position_mut`, `position_set`, `ToString`, and the custom
//! `Debug` of `mdxjs-rs`.
//!
//! [hast]: https://github.com/syntax-tree/hast

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{borrow::Cow, string::String, vec::Vec};
use markdown::unist::Position;

/// Nodes.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", rename_all = "camelCase")
)]
pub enum Node {
    /// Root.
    Root(Root),
    /// Element.
    Element(Element),
    /// Document type.
    Doctype(Doctype),
    /// Comment.
    Comment(Comment),
    /// Text.
    Text(Text),
    /// Raw HTML, passed through by the serializer when dangerous HTML is
    /// allowed, and encoded as text otherwise.
    Raw(Raw),
}

impl Node {
    /// Get children of a hast node.
    #[must_use]
    pub fn children(&self) -> Option<&Vec<Node>> {
        match self {
            Node::Root(x) => Some(&x.children),
            Node::Element(x) => Some(&x.children),
            _ => None,
        }
    }

    /// Get children of a hast node, mutably.
    pub fn children_mut(&mut self) -> Option<&mut Vec<Node>> {
        match self {
            Node::Root(x) => Some(&mut x.children),
            Node::Element(x) => Some(&mut x.children),
            _ => None,
        }
    }

    /// Get the position of a hast node.
    #[must_use]
    pub fn position(&self) -> Option<&Position> {
        match self {
            Node::Root(x) => x.position.as_ref(),
            Node::Element(x) => x.position.as_ref(),
            Node::Doctype(x) => x.position.as_ref(),
            Node::Comment(x) => x.position.as_ref(),
            Node::Text(x) => x.position.as_ref(),
            Node::Raw(x) => x.position.as_ref(),
        }
    }
}

/// Document.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Root {
    /// Content model.
    pub children: Vec<Node>,
    /// Positional info.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub position: Option<Position>,
}

/// Element.
///
/// ```html
/// > | <a href="b">c</a>
///     ^^^^^^^^^^^^^^^^^
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Element {
    /// Tag name.
    pub tag_name: Cow<'static, str>,
    /// Properties, keyed by hast property name (such as `className`).
    #[cfg_attr(feature = "serde", serde(with = "properties"))]
    pub properties: Vec<(Cow<'static, str>, PropertyValue)>,
    /// Children.
    pub children: Vec<Node>,
    /// Positional info.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub position: Option<Position>,
}

/// Property value.
///
/// In hast’s JSON, a list is an array: which separator it uses comes from the
/// property, not the tree. Deserialized arrays are space-separated, numbers
/// become strings, and a `null` property is left out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyValue {
    /// A boolean.
    Boolean(bool),
    /// A string.
    String(String),
    /// A comma-separated list of strings.
    CommaSeparated(Vec<String>),
    /// A space-separated list of strings.
    SpaceSeparated(Vec<String>),
}

/// Document type.
///
/// ```html
/// > | <!doctype html>
///     ^^^^^^^^^^^^^^^
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Doctype {
    /// Positional info.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub position: Option<Position>,
}

/// Comment.
///
/// ```html
/// > | <!-- a -->
///     ^^^^^^^^^^
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Comment {
    /// Content model.
    pub value: String,
    /// Positional info.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub position: Option<Position>,
}

/// Text.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Text {
    /// Content model.
    pub value: Cow<'static, str>,
    /// Positional info.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub position: Option<Position>,
}

/// Raw HTML.
///
/// ```html
/// > | <div>
///     ^^^^^
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Raw {
    /// Content model.
    pub value: String,
    /// Positional info.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub position: Option<Position>,
}

#[cfg(feature = "serde")]
impl serde::Serialize for PropertyValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            PropertyValue::Boolean(value) => serializer.serialize_bool(*value),
            PropertyValue::String(value) => serializer.serialize_str(value),
            PropertyValue::CommaSeparated(values) | PropertyValue::SpaceSeparated(values) => {
                values.serialize(serializer)
            }
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for PropertyValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use alloc::string::ToString;

        // Numbers, such as `start` or `colSpan` from JS, become strings.
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum Scalar {
            String(String),
            Integer(i64),
            Float(f64),
        }

        impl Scalar {
            fn into_string(self) -> String {
                match self {
                    Scalar::String(value) => value,
                    Scalar::Integer(value) => value.to_string(),
                    Scalar::Float(value) => value.to_string(),
                }
            }
        }

        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum Value {
            Boolean(bool),
            Scalar(Scalar),
            List(Vec<Scalar>),
        }

        Ok(match Value::deserialize(deserializer)? {
            Value::Boolean(value) => PropertyValue::Boolean(value),
            Value::Scalar(value) => PropertyValue::String(value.into_string()),
            Value::List(values) => {
                PropertyValue::SpaceSeparated(values.into_iter().map(Scalar::into_string).collect())
            }
        })
    }
}

/// Properties as a JSON object, in order.
#[cfg(feature = "serde")]
mod properties {
    use super::PropertyValue;
    use alloc::{borrow::Cow, string::String, vec::Vec};
    use serde::ser::SerializeMap;

    type Properties = Vec<(Cow<'static, str>, PropertyValue)>;

    pub fn serialize<S: serde::Serializer>(
        properties: &Properties,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(properties.len()))?;
        for (name, value) in properties {
            map.serialize_entry(name, value)?;
        }
        map.end()
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Properties, D::Error> {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Properties;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("an object of properties")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Properties, A::Error> {
                let mut properties = Vec::new();
                while let Some((name, value)) = map.next_entry::<String, Option<PropertyValue>>()? {
                    // In hast, `null` means the property is not set.
                    if let Some(value) = value {
                        properties.push((name.into(), value));
                    }
                }
                Ok(properties)
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}
