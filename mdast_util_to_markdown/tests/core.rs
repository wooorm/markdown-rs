use markdown::mdast::Definition;
use markdown::mdast::{Node, Paragraph, Root, Text, ThematicBreak};
use mdast_util_to_markdown::to_markdown_with_options as to_md_with_opts;
use mdast_util_to_markdown::{to_markdown as to, Options};
use pretty_assertions::assert_eq;

#[test]
fn core() {
    assert_eq!(
        to_md_with_opts(
            &Node::Root(Root {
                children: vec![
                    Node::Paragraph(Paragraph {
                        children: vec![Node::Text(Text {
                            value: String::from("a"),
                            position: None
                        })],
                        position: None
                    }),
                    Node::Definition(Definition {
                        position: None,
                        url: String::new(),
                        title: None,
                        identifier: String::from("b"),
                        label: None
                    }),
                    Node::Definition(Definition {
                        position: None,
                        url: String::new(),
                        title: None,
                        identifier: String::from("c"),
                        label: None
                    }),
                    Node::Paragraph(Paragraph {
                        children: vec![Node::Text(Text {
                            value: String::from("d"),
                            position: None
                        })],
                        position: None
                    }),
                ],
                position: None
            }),
            &Options {
                tight_definitions: true,
                ..Default::default()
            }
        )
        .unwrap(),
        "a\n\n[b]: <>\n[c]: <>\n\nd\n",
        "should support tight adjacent definitions when `tight_definitions: true`"
    );

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
