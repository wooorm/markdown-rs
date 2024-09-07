use markdown::mdast::{BlockQuote, Node, Paragraph, Text, ThematicBreak};
use mdast_util_to_markdown::to_markdown as to;

use pretty_assertions::assert_eq;

#[test]
fn block_quote() {
    assert_eq!(
        to(&Node::BlockQuote(BlockQuote {
            children: vec![],
            position: None,
        }))
        .unwrap(),
        ">\n",
        "should support a block quote"
    );

    assert_eq!(
        to(&Node::BlockQuote(BlockQuote {
            children: vec![Node::Text(Text {
                value: String::from("a"),
                position: None
            })],
            position: None,
        }))
        .unwrap(),
        "> a\n",
        "should support a block quote w/ a child"
    );

    assert_eq!(
        to(&Node::BlockQuote(BlockQuote {
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
            position: None,
        }))
        .unwrap(),
        "> a\n>\n> ***\n>\n> b\n",
        "should support a block quote"
    );
}
