use markdown::mdast::Definition;
use markdown::mdast::{
    Blockquote, Break, Code, Heading, InlineCode, Node, Paragraph, Text, ThematicBreak,
};

use mdast_util_to_markdown::to_markdown as to;
use mdast_util_to_markdown::to_markdown_with_options as to_md_with_opts;

use mdast_util_to_markdown::Options;
use pretty_assertions::assert_eq;

#[test]
fn block_quote() {
    assert_eq!(
        to(&Node::Blockquote(Blockquote {
            children: vec![],
            position: None,
        }))
        .unwrap(),
        ">\n",
        "should support a block quote"
    );

    assert_eq!(
        to(&Node::Blockquote(Blockquote {
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
        to(&Node::Blockquote(Blockquote {
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
        "should support a block quote w/ children"
    );

    assert_eq!(
        to(&Node::Blockquote(Blockquote {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![Node::Text(Text {
                    value: String::from("a\nb"),
                    position: None
                })],
                position: None
            }),],
            position: None,
        }))
        .unwrap(),
        "> a\n> b\n",
        "should support text w/ a line ending in a block quote"
    );

    assert_eq!(
        to(&Node::Blockquote(Blockquote {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: String::from("a"),
                        position: None
                    }),
                    Node::Text(Text {
                        value: String::from("b"),
                        position: None
                    })
                ],
                position: None
            }),],
            position: None,
        }))
        .unwrap(),
        "> ab\n",
        "should support adjacent texts in a block quote"
    );

    assert_eq!(
        to(&Node::Blockquote(Blockquote {
            children: vec![
                Node::Paragraph(Paragraph {
                    children: vec![Node::Text(Text {
                        value: String::from("a\nb"),
                        position: None
                    })],
                    position: None
                }),
                Node::Blockquote(Blockquote {
                    children: vec![
                        Node::Paragraph(Paragraph {
                            children: vec![
                                Node::Text(Text {
                                    value: String::from("a\n"),
                                    position: None
                                }),
                                Node::InlineCode(InlineCode {
                                    value: String::from("b\nc"),
                                    position: None
                                }),
                                Node::Text(Text {
                                    value: String::from("\nd"),
                                    position: None
                                }),
                            ],
                            position: None
                        }),
                        Node::Heading(Heading {
                            children: vec![Node::Text(Text {
                                value: String::from("a b"),
                                position: None
                            })],
                            position: None,
                            depth: 1
                        })
                    ],
                    position: None
                }),
            ],
            position: None,
        }))
        .unwrap(),
        "> a\n> b\n>\n> > a\n> > `b\n> > c`\n> > d\n> >\n> > # a b\n",
        "should support a block quote in a block quote"
    );

    assert_eq!(
        to(&Node::Blockquote(Blockquote {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: String::from("a"),
                        position: None
                    }),
                    Node::Break(Break { position: None }),
                    Node::Text(Text {
                        value: String::from("b"),
                        position: None
                    })
                ],
                position: None
            }),],
            position: None,
        }))
        .unwrap(),
        "> a\\\n> b\n",
        "should support a break in a block quote"
    );

    assert_eq!(
        to_md_with_opts(
            &Node::Blockquote(Blockquote {
                children: vec![Node::Code(Code {
                    value: String::from("a\nb\n\nc"),
                    position: None,
                    lang: None,
                    meta: None
                })],
                position: None,
            }),
            &Options {
                fences: false,
                ..Default::default()
            }
        )
        .unwrap(),
        ">     a\n>     b\n>\n>     c\n",
        "should support code (flow, indented) in a block quote"
    );

    assert_eq!(
        to(&Node::Blockquote(Blockquote {
            children: vec![Node::Code(Code {
                value: String::from("c\nd\n\ne"),
                position: None,
                lang: String::from("a\nb").into(),
                meta: None
            })],
            position: None,
        }))
        .unwrap(),
        "> ```a&#xA;b\n> c\n> d\n>\n> e\n> ```\n",
        "should support code (flow, fenced) in a block quote"
    );

    assert_eq!(
        to(&Node::Blockquote(Blockquote {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: String::from("a\n"),
                        position: None
                    }),
                    Node::InlineCode(InlineCode {
                        value: String::from("b\nc"),
                        position: None
                    }),
                    Node::Text(Text {
                        value: String::from("\nd"),
                        position: None
                    })
                ],
                position: None
            })],
            position: None,
        }))
        .unwrap(),
        "> a\n> `b\n> c`\n> d\n",
        "should support code (text) in a block quote"
    );

    assert_eq!(
        to(&Node::Blockquote(Blockquote {
            children: vec![
                Node::Definition(Definition {
                    position: None,
                    title: Some("e\nf".into()),
                    url: "c\nd".into(),
                    identifier: "a\nb".into(),
                    label: None
                }),
                Node::Paragraph(Paragraph {
                    children: vec![Node::Text(Text {
                        value: String::from("a\nb"),
                        position: None
                    })],
                    position: None
                })
            ],
            position: None,
        }))
        .unwrap(),
        "> [a\n> b]: <c&#xA;d> \"e\n> f\"\n>\n> a\n> b\n",
        "should support a definition in a block quote"
    );
}
