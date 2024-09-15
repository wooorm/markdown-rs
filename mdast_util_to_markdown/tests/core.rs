use markdown::mdast::{Node, Paragraph, Root, Text, ThematicBreak};
use mdast_util_to_markdown::to_markdown as to;

use pretty_assertions::assert_eq;

#[test]
fn core() {
    assert_eq!(
        to(&Node::Root(Root {
            children: vec![
                Node::Paragraph(Paragraph {
                    children: vec![Node::Text(Text {
                        value: String::from("a"),
                        position: None
                    })],
                    position: None
                }),
                Node::ThematicBreak(ThematicBreak { position: None }),
                Node::Paragraph(Paragraph {
                    children: vec![Node::Text(Text {
                        value: String::from("b"),
                        position: None
                    })],
                    position: None
                }),
            ],
            position: None
        }))
        .unwrap(),
        "a\n\n***\n\nb\n",
        "should support root"
    );
}
