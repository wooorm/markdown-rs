#![cfg(feature = "serde")]

use hast::{Comment, Doctype, Element, Node, PropertyValue, Raw, Root, Text};
use markdown::unist::Position;
use pretty_assertions::assert_eq;

#[test]
fn serde() -> Result<(), serde_json::Error> {
    let tree = Node::Root(Root {
        children: vec![
            Node::Doctype(Doctype { position: None }),
            Node::Element(Element {
                tag_name: "p".into(),
                properties: vec![
                    (
                        "className".into(),
                        PropertyValue::SpaceSeparated(vec!["a".into(), "b".into()]),
                    ),
                    ("hidden".into(), PropertyValue::Boolean(true)),
                    ("title".into(), PropertyValue::String("c".into())),
                ],
                children: vec![Node::Text(Text {
                    value: "d".into(),
                    position: Some(Position::new(1, 1, 0, 1, 2, 1)),
                })],
                position: None,
            }),
            Node::Comment(Comment {
                value: "e".into(),
                position: None,
            }),
            Node::Raw(Raw {
                value: "<f>".into(),
                position: None,
            }),
        ],
        position: None,
    });
    let json = r#"{"type":"root","children":[{"type":"doctype"},{"type":"element","tagName":"p","properties":{"className":["a","b"],"hidden":true,"title":"c"},"children":[{"type":"text","value":"d","position":{"start":{"line":1,"column":1,"offset":0},"end":{"line":1,"column":2,"offset":1}}}]},{"type":"comment","value":"e"},{"type":"raw","value":"<f>"}]}"#;

    assert_eq!(
        serde_json::to_string(&tree)?,
        json,
        "should serialize in the shape of hast’s JSON"
    );
    assert_eq!(
        serde_json::from_str::<Node>(json)?,
        tree,
        "should deserialize hast’s JSON"
    );
    assert_eq!(
        serde_json::from_str::<PropertyValue>(r#"["a","b"]"#)?,
        PropertyValue::SpaceSeparated(vec!["a".into(), "b".into()]),
        "should read a list as space-separated, as hast’s JSON does not say"
    );
    assert_eq!(
        serde_json::to_string(&PropertyValue::CommaSeparated(vec!["a".into()]))?,
        r#"["a"]"#,
        "should write a comma-separated list as an array"
    );
    assert_eq!(
        serde_json::from_str::<Node>(
            r#"{"type":"element","tagName":"ol","properties":{"start":2,"width":1.5,"coords":[1,2],"id":null},"children":[]}"#
        )?,
        Node::Element(Element {
            tag_name: "ol".into(),
            properties: vec![
                ("start".into(), PropertyValue::String("2".into())),
                ("width".into(), PropertyValue::String("1.5".into())),
                (
                    "coords".into(),
                    PropertyValue::SpaceSeparated(vec!["1".into(), "2".into()])
                ),
            ],
            children: vec![],
            position: None,
        }),
        "should read numbers from JS as strings, and leave out `null` properties"
    );
    assert!(
        serde_json::from_str::<Node>(
            r#"{"type":"element","tagName":"p","properties":[],"children":[]}"#
        )
        .unwrap_err()
        .to_string()
        .contains("an object of properties"),
        "should expect properties as an object"
    );

    Ok(())
}
