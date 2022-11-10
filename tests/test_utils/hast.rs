//! HTML syntax tree: [hast][].
//!
//! [hast]: https://github.com/syntax-tree/hast

#![allow(dead_code)]
// ^-- To do: externalize.

extern crate alloc;
use alloc::{
    fmt,
    string::{String, ToString},
    vec::Vec,
};
pub use markdown::mdast::{AttributeContent, AttributeValue, MdxJsxAttribute, Stop};
use markdown::unist::Position;

/// Nodes.
#[derive(Clone, PartialEq, Eq)]
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
            Node::Doctype(_) => "".into(),
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
#[derive(Clone, Debug, PartialEq, Eq)]
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    pub tag_name: String,
    pub properties: Vec<(String, PropertyValue)>,
    // Parent.
    pub children: Vec<Node>,
    /// Positional info.
    pub position: Option<Position>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyValue {
    Boolean(bool),
    String(String),
    CommaSeparated(Vec<String>),
    SpaceSeparated(Vec<String>),
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MdxJsxElement {
    // JSX element.
    /// Name.
    ///
    /// Fragments have no name.
    pub name: Option<String>,
    /// Attributes.
    pub attributes: Vec<AttributeContent>,
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
    /// Positional info.
    pub position: Option<Position>,
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

    // Custom data on where each slice of `value` came from.
    pub stops: Vec<Stop>,
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

    // Custom data on where each slice of `value` came from.
    pub stops: Vec<Stop>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{format, string::ToString, vec};
    use markdown::unist::Position;

    // Literals.

    #[test]
    fn text() {
        let mut node = Node::Text(Text {
            value: "a".into(),
            position: None,
        });

        assert_eq!(
            format!("{:?}", node),
            "Text { value: \"a\", position: None }",
            "should support `Debug`"
        );
        assert_eq!(node.to_string(), "a", "should support `ToString`");
        assert_eq!(node.children_mut(), None, "should support `children_mut`");
        assert_eq!(node.children(), None, "should support `children`");
        assert_eq!(node.position(), None, "should support `position`");
        assert_eq!(node.position_mut(), None, "should support `position`");
        node.position_set(Some(Position::new(1, 1, 0, 1, 2, 1)));
        assert_eq!(
            format!("{:?}", node),
            "Text { value: \"a\", position: Some(1:1-1:2 (0-1)) }",
            "should support `position_set`"
        );
    }

    #[test]
    fn comment() {
        let mut node = Node::Comment(Comment {
            value: "a".into(),
            position: None,
        });

        assert_eq!(
            format!("{:?}", node),
            "Comment { value: \"a\", position: None }",
            "should support `Debug`"
        );
        assert_eq!(node.to_string(), "a", "should support `ToString`");
        assert_eq!(node.children_mut(), None, "should support `children_mut`");
        assert_eq!(node.children(), None, "should support `children`");
        assert_eq!(node.position(), None, "should support `position`");
        assert_eq!(node.position_mut(), None, "should support `position`");
        node.position_set(Some(Position::new(1, 1, 0, 1, 2, 1)));
        assert_eq!(
            format!("{:?}", node),
            "Comment { value: \"a\", position: Some(1:1-1:2 (0-1)) }",
            "should support `position_set`"
        );
    }

    #[test]
    fn mdx_expression() {
        let mut node = Node::MdxExpression(MdxExpression {
            value: "a".into(),
            stops: vec![],
            position: None,
        });

        assert_eq!(
            format!("{:?}", node),
            "MdxExpression { value: \"a\", position: None, stops: [] }",
            "should support `Debug`"
        );
        assert_eq!(node.to_string(), "a", "should support `ToString`");
        assert_eq!(node.children_mut(), None, "should support `children_mut`");
        assert_eq!(node.children(), None, "should support `children`");
        assert_eq!(node.position(), None, "should support `position`");
        assert_eq!(node.position_mut(), None, "should support `position`");
        node.position_set(Some(Position::new(1, 1, 0, 1, 2, 1)));
        assert_eq!(
            format!("{:?}", node),
            "MdxExpression { value: \"a\", position: Some(1:1-1:2 (0-1)), stops: [] }",
            "should support `position_set`"
        );
    }

    #[test]
    fn mdxjs_esm() {
        let mut node = Node::MdxjsEsm(MdxjsEsm {
            value: "a".into(),
            stops: vec![],
            position: None,
        });

        assert_eq!(
            format!("{:?}", node),
            "MdxjsEsm { value: \"a\", position: None, stops: [] }",
            "should support `Debug`"
        );
        assert_eq!(node.to_string(), "a", "should support `ToString`");
        assert_eq!(node.children_mut(), None, "should support `children_mut`");
        assert_eq!(node.children(), None, "should support `children`");
        assert_eq!(node.position(), None, "should support `position`");
        assert_eq!(node.position_mut(), None, "should support `position`");
        node.position_set(Some(Position::new(1, 1, 0, 1, 2, 1)));
        assert_eq!(
            format!("{:?}", node),
            "MdxjsEsm { value: \"a\", position: Some(1:1-1:2 (0-1)), stops: [] }",
            "should support `position_set`"
        );
    }

    // Voids.

    #[test]
    fn doctype() {
        let mut node = Node::Doctype(Doctype { position: None });

        assert_eq!(
            format!("{:?}", node),
            "Doctype { position: None }",
            "should support `Debug`"
        );
        assert_eq!(node.to_string(), "", "should support `ToString`");
        assert_eq!(node.children_mut(), None, "should support `children_mut`");
        assert_eq!(node.children(), None, "should support `children`");
        assert_eq!(node.position(), None, "should support `position`");
        assert_eq!(node.position_mut(), None, "should support `position`");
        node.position_set(Some(Position::new(1, 1, 0, 1, 2, 1)));
        assert_eq!(
            format!("{:?}", node),
            "Doctype { position: Some(1:1-1:2 (0-1)) }",
            "should support `position_set`"
        );
    }

    // Parents.

    #[test]
    fn root() {
        let mut node = Node::Root(Root {
            position: None,
            children: vec![],
        });

        assert_eq!(
            format!("{:?}", node),
            "Root { children: [], position: None }",
            "should support `Debug`"
        );
        assert_eq!(node.to_string(), "", "should support `ToString`");
        assert_eq!(
            node.children_mut(),
            Some(&mut vec![]),
            "should support `children_mut`"
        );
        assert_eq!(node.children(), Some(&vec![]), "should support `children`");
        assert_eq!(node.position(), None, "should support `position`");
        assert_eq!(node.position_mut(), None, "should support `position`");
        node.position_set(Some(Position::new(1, 1, 0, 1, 2, 1)));
        assert_eq!(
            format!("{:?}", node),
            "Root { children: [], position: Some(1:1-1:2 (0-1)) }",
            "should support `position_set`"
        );
    }

    #[test]
    fn element() {
        let mut node = Node::Element(Element {
            tag_name: "a".into(),
            properties: vec![],
            position: None,
            children: vec![],
        });

        assert_eq!(
            format!("{:?}", node),
            "Element { tag_name: \"a\", properties: [], children: [], position: None }",
            "should support `Debug`"
        );
        assert_eq!(node.to_string(), "", "should support `ToString`");
        assert_eq!(
            node.children_mut(),
            Some(&mut vec![]),
            "should support `children_mut`"
        );
        assert_eq!(node.children(), Some(&vec![]), "should support `children`");
        assert_eq!(node.position(), None, "should support `position`");
        assert_eq!(node.position_mut(), None, "should support `position`");
        node.position_set(Some(Position::new(1, 1, 0, 1, 2, 1)));
        assert_eq!(
            format!("{:?}", node),
            "Element { tag_name: \"a\", properties: [], children: [], position: Some(1:1-1:2 (0-1)) }",
            "should support `position_set`"
        );
    }

    #[test]
    fn mdx_jsx_element() {
        let mut node = Node::MdxJsxElement(MdxJsxElement {
            name: None,
            attributes: vec![],
            position: None,
            children: vec![],
        });

        assert_eq!(
            format!("{:?}", node),
            "MdxJsxElement { name: None, attributes: [], children: [], position: None }",
            "should support `Debug`"
        );
        assert_eq!(node.to_string(), "", "should support `ToString`");
        assert_eq!(
            node.children_mut(),
            Some(&mut vec![]),
            "should support `children_mut`"
        );
        assert_eq!(node.children(), Some(&vec![]), "should support `children`");
        assert_eq!(node.position(), None, "should support `position`");
        assert_eq!(node.position_mut(), None, "should support `position`");
        node.position_set(Some(Position::new(1, 1, 0, 1, 2, 1)));
        assert_eq!(
            format!("{:?}", node),
            "MdxJsxElement { name: None, attributes: [], children: [], position: Some(1:1-1:2 (0-1)) }",
            "should support `position_set`"
        );
    }
}
