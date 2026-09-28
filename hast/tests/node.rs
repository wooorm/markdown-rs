use hast::{Comment, Doctype, Element, Node, Raw, Root, Text};
use markdown::unist::Position;
use pretty_assertions::assert_eq;

fn text(value: &'static str, position: Option<Position>) -> Node {
    Node::Text(Text {
        value: value.into(),
        position,
    })
}

#[test]
fn children() {
    let mut root = Node::Root(Root {
        children: vec![text("a", None)],
        position: None,
    });
    let mut element = Node::Element(Element {
        tag_name: "p".into(),
        properties: vec![],
        children: vec![text("b", None)],
        position: None,
    });

    assert_eq!(
        root.children(),
        Some(&vec![text("a", None)]),
        "should support children of a root"
    );
    assert_eq!(
        element.children(),
        Some(&vec![text("b", None)]),
        "should support children of an element"
    );
    assert_eq!(
        text("c", None).children(),
        None,
        "should support nodes without children"
    );

    root.children_mut().unwrap().push(text("d", None));
    element.children_mut().unwrap().clear();

    assert_eq!(
        root.children().map(Vec::len),
        Some(2),
        "should support changing children of a root"
    );
    assert_eq!(
        element.children().map(Vec::len),
        Some(0),
        "should support changing children of an element"
    );
    assert_eq!(
        text("e", None).children_mut(),
        None,
        "should support nodes without children, mutably"
    );
}

#[test]
fn position() {
    let position = Position::new(1, 1, 0, 1, 2, 1);
    let nodes = [
        Node::Root(Root {
            children: vec![],
            position: Some(position.clone()),
        }),
        Node::Element(Element {
            tag_name: "p".into(),
            properties: vec![],
            children: vec![],
            position: Some(position.clone()),
        }),
        Node::Doctype(Doctype {
            position: Some(position.clone()),
        }),
        Node::Comment(Comment {
            value: "a".into(),
            position: Some(position.clone()),
        }),
        text("a", Some(position.clone())),
        Node::Raw(Raw {
            value: "<a>".into(),
            position: Some(position.clone()),
        }),
    ];

    for node in &nodes {
        assert_eq!(
            node.position(),
            Some(&position),
            "should support the position of {:?}",
            node
        );
    }
}
