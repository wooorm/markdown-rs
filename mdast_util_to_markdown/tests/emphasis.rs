use markdown::mdast::Emphasis;
use markdown::mdast::{Node, Text};
use mdast_util_to_markdown::to_markdown as to;
use mdast_util_to_markdown::to_markdown_with_options as to_md_with_opts;

use mdast_util_to_markdown::Options;
use pretty_assertions::assert_eq;

#[test]
fn emphasis() {
    assert_eq!(
        to(&Node::Emphasis(Emphasis {
            children: Vec::new(),
            position: None
        }))
        .unwrap(),
        "**\n",
        "should support an empty emphasis"
    );

    assert_eq!(
        to(&Node::Emphasis(Emphasis {
            children: vec![Node::Text(Text {
                value: String::from("a"),
                position: None,
            })],
            position: None
        }))
        .unwrap(),
        "*a*\n",
        "should support an emphasis w/ children"
    );

    assert_eq!(
        to_md_with_opts(
            &Node::Emphasis(Emphasis {
                children: vec![Node::Text(Text {
                    value: String::from("a"),
                    position: None,
                })],
                position: None
            }),
            &Options {
                emphasis: '_',
                ..Default::default()
            }
        )
        .unwrap(),
        "_a_\n",
        "should support an emphasis w/ underscores when `emphasis: \"_\"`"
    );
}
