use markdown::mdast::{ListItem, Node, Text};
use markdown::mdast::{Paragraph, ThematicBreak};
use mdast_util_to_markdown::{to_markdown as to, IndentOptions};

use mdast_util_to_markdown::to_markdown_with_options as to_md_with_opts;
use mdast_util_to_markdown::Options;
use pretty_assertions::assert_eq;

#[test]
fn list_item() {
    assert_eq!(
        to(&Node::ListItem(ListItem {
            children: vec![],
            position: None,
            spread: false,
            checked: None
        }))
        .unwrap(),
        "*\n",
        "should support a list item"
    );

    assert_eq!(
        to_md_with_opts(
            &Node::ListItem(ListItem {
                children: Vec::new(),
                position: None,
                spread: false,
                checked: None
            }),
            &Options {
                bullet: '+',
                ..Default::default()
            }
        )
        .unwrap(),
        "+\n",
        "should serialize an item w/ a plus as bullet when `bullet: \" + \"`"
    );

    assert_eq!(
        to(&Node::ListItem(ListItem {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![Node::Text(Text {
                    value: String::from("a"),
                    position: None
                })],
                position: None
            })],
            position: None,
            spread: false,
            checked: None
        }))
        .unwrap(),
        "* a\n",
        "should support a list item w/ a child"
    );

    assert_eq!(
        to(&Node::ListItem(ListItem {
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
            spread: false,
            checked: None
        }))
        .unwrap(),
        "* a\n  ***\n  b\n",
        "should support a list item w/ children"
    );

    assert_eq!(
        to_md_with_opts(
            &Node::ListItem(ListItem {
                children: vec![
                    Node::Paragraph(Paragraph {
                        children: vec![Node::Text(Text {
                            value: String::from("a"),
                            position: None
                        })],
                        position: None
                    }),
                    Node::ThematicBreak(ThematicBreak { position: None })
                ],
                position: None,
                spread: false,
                checked: None
            }),
            &Options {
                list_item_indent: IndentOptions::One,
                ..Default::default()
            }
        )
        .unwrap(),
        "* a\n  ***\n",
        "should use one space after the bullet for `listItemIndent: \"one\"`"
    );

    assert_eq!(
        to_md_with_opts(
            &Node::ListItem(ListItem {
                children: vec![Node::Paragraph(Paragraph {
                    children: vec![Node::Text(Text {
                        value: String::from("a"),
                        position: None
                    })],
                    position: None
                }),],
                position: None,
                spread: false,
                checked: None
            }),
            &Options {
                list_item_indent: IndentOptions::Mixed,
                ..Default::default()
            }
        )
        .unwrap(),
        "* a\n",
        "should use one space after the bullet for `listItemIndent: \"mixed\"`, when the item is not spread"
    );

    assert_eq!(
        to_md_with_opts(
            &Node::ListItem(ListItem {
                children: vec![Node::Paragraph(Paragraph {
                    children: vec![Node::Text(Text {
                        value: String::from("a"),
                        position: None
                    })],
                    position: None
                }),
                Node::ThematicBreak(ThematicBreak { position: None })],
                position: None,
                spread: true,
                checked: None
            }),
            &Options {
                list_item_indent: IndentOptions::Mixed,
                ..Default::default()
            }
        )
        .unwrap(),
        "*   a\n\n    ***\n",
        "should use a tab stop of spaces after the bullet for `listItemIndent: \"mixed\"`, when the item is spread"
    );

    assert_eq!(
        to(&Node::ListItem(ListItem {
            children: vec![
                Node::Paragraph(Paragraph {
                    children: vec![Node::Text(Text {
                        value: String::from("a"),
                        position: None
                    })],
                    position: None
                }),
                Node::ThematicBreak(ThematicBreak { position: None }),
            ],
            position: None,
            spread: false,
            checked: None
        }))
        .unwrap(),
        "* a\n  ***\n",
        "should not use blank lines between child blocks for items w/ `spread: false`"
    );
}
