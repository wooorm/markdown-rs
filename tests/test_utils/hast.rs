#![allow(dead_code)]

// ^-- fix later

extern crate alloc;
extern crate micromark;
use alloc::{
    fmt,
    string::{String, ToString},
    vec::Vec,
};
use micromark::{mdast::AttributeContent, unist::Position};

/// Nodes.
#[derive(Clone, PartialEq)]
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

    // MDX being passed through.
    /// MDX: JSX element.
    MdxJsxElement(MdxJsxElement),
    /// MDX.js ESM.
    MdxjsEsm(MdxjsEsm),
    // MDX: expression.
    MdxExpression(MdxExpression),
}

impl fmt::Debug for Node {
    // Debug the wrapped struct.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Root(x) => write!(f, "{:?}", x),
            Node::Element(x) => write!(f, "{:?}", x),
            Node::Doctype(x) => write!(f, "{:?}", x),
            Node::Comment(x) => write!(f, "{:?}", x),
            Node::Text(x) => write!(f, "{:?}", x),
            Node::MdxJsxElement(x) => write!(f, "{:?}", x),
            Node::MdxExpression(x) => write!(f, "{:?}", x),
            Node::MdxjsEsm(x) => write!(f, "{:?}", x),
        }
    }
}

fn children_to_string(children: &[Node]) -> String {
    children.iter().map(ToString::to_string).collect()
}

impl ToString for Node {
    fn to_string(&self) -> String {
        match self {
            // Parents.
            Node::Root(x) => children_to_string(&x.children),
            Node::Element(x) => children_to_string(&x.children),
            Node::MdxJsxElement(x) => children_to_string(&x.children),
            // Literals.
            Node::Comment(x) => x.value.clone(),
            Node::Text(x) => x.value.clone(),
            Node::MdxExpression(x) => x.value.clone(),
            Node::MdxjsEsm(x) => x.value.clone(),
            // Voids.
            Node::Doctype(_) => "".to_string(),
        }
    }
}

impl Node {
    #[must_use]
    pub fn children(&self) -> Option<&Vec<Node>> {
        match self {
            // Parent.
            Node::Root(x) => Some(&x.children),
            Node::Element(x) => Some(&x.children),
            Node::MdxJsxElement(x) => Some(&x.children),
            // Non-parent.
            _ => None,
        }
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<Node>> {
        match self {
            // Parent.
            Node::Root(x) => Some(&mut x.children),
            Node::Element(x) => Some(&mut x.children),
            Node::MdxJsxElement(x) => Some(&mut x.children),
            // Non-parent.
            _ => None,
        }
    }

    pub fn position(&self) -> Option<&Position> {
        match self {
            Node::Root(x) => x.position.as_ref(),
            Node::Element(x) => x.position.as_ref(),
            Node::Doctype(x) => x.position.as_ref(),
            Node::Comment(x) => x.position.as_ref(),
            Node::Text(x) => x.position.as_ref(),
            Node::MdxJsxElement(x) => x.position.as_ref(),
            Node::MdxExpression(x) => x.position.as_ref(),
            Node::MdxjsEsm(x) => x.position.as_ref(),
        }
    }

    pub fn position_mut(&mut self) -> Option<&mut Position> {
        match self {
            Node::Root(x) => x.position.as_mut(),
            Node::Element(x) => x.position.as_mut(),
            Node::Doctype(x) => x.position.as_mut(),
            Node::Comment(x) => x.position.as_mut(),
            Node::Text(x) => x.position.as_mut(),
            Node::MdxJsxElement(x) => x.position.as_mut(),
            Node::MdxExpression(x) => x.position.as_mut(),
            Node::MdxjsEsm(x) => x.position.as_mut(),
        }
    }

    pub fn position_set(&mut self, position: Option<Position>) {
        match self {
            Node::Root(x) => x.position = position,
            Node::Element(x) => x.position = position,
            Node::Doctype(x) => x.position = position,
            Node::Comment(x) => x.position = position,
            Node::Text(x) => x.position = position,
            Node::MdxJsxElement(x) => x.position = position,
            Node::MdxExpression(x) => x.position = position,
            Node::MdxjsEsm(x) => x.position = position,
        }
    }
}

/// Document.
///
/// ```html
/// > | a
///     ^
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Root {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
    /// Positional info.
    pub position: Option<Position>,
}

/// Document type.
///
/// ```html
/// > | <!doctype html>
///     ^^^^^^^^^^^^^^^
/// ```
// To do: clone.
#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    pub tag_name: String,
    pub properties: Vec<(String, PropertyValue)>,
    // Parent.
    pub children: Vec<Node>,
    /// Positional info.
    pub position: Option<Position>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PropertyItem {
    Number(f32),
    String(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PropertyValue {
    Number(f32),
    Boolean(bool),
    String(String),
    CommaSeparated(Vec<PropertyItem>),
    SpaceSeparated(Vec<PropertyItem>),
}

/// Document type.
///
/// ```html
/// > | <!doctype html>
///     ^^^^^^^^^^^^^^^
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Doctype {
    // Void.
    /// Positional info.
    pub position: Option<Position>,
}

/// Comment.
///
/// ```html
/// > | <!-- a -->
///     ^^^^^^^^^^
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comment {
    // Text.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,
}

/// Text.
///
/// ```html
/// > | a
///     ^
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Text {
    // Text.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,
}

/// MDX: JSX element.
///
/// ```markdown
/// > | <a />
///     ^^^^^
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MdxJsxElement {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
    /// Positional info.
    pub position: Option<Position>,
    // JSX element.
    /// Name.
    ///
    /// Fragments have no name.
    pub name: Option<String>,
    /// Attributes.
    pub attributes: Vec<AttributeContent>,
}

/// MDX: expression.
///
/// ```markdown
/// > | {a}
///     ^^^
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MdxExpression {
    // Literal.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,
}

/// MDX: ESM.
///
/// ```markdown
/// > | import a from 'b'
///     ^^^^^^^^^^^^^^^^^
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MdxjsEsm {
    // Literal.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,
}
