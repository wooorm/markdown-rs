use markdown::mdast::{Math, Node, Paragraph, Text};
use mdast_util_to_markdown::{
    to_markdown as to, to_markdown_with_options as to_md_with_opts, Options,
};
use pretty_assertions::assert_eq;

#[test]
fn math() {
    assert_eq!(
        to(&Node::Math(Math {
            value: String::from("a"),
            position: None,
            meta: None
        }))
        .unwrap(),
        "$$\na\n$$\n",
        "should serialize math (flow)"
    );

    assert_eq!(
        to(&Node::Math(Math {
            value: String::new(),
            position: None,
            meta: None
        }))
        .unwrap(),
        "$$\n$$\n",
        "should serialize math (flow) w/o `value`"
    );

    assert_eq!(
        to(&Node::Math(Math {
            value: String::new(),
            position: None,
            meta: String::from("a").into()
        }))
        .unwrap(),
        "$$a\n$$\n",
        "should serialize math (flow) w/ `meta`"
    );

    assert_eq!(
        to(&Node::Math(Math {
            value: String::from("$$"),
            position: None,
            meta: None
        }))
        .unwrap(),
        "$$$\n$$\n$$$\n",
        "should serialize math (flow) w/ more dollars than occur together in `value`"
    );

    assert_eq!(
        to(&Node::Paragraph(Paragraph {
            children: vec![Node::Text(Text {
                value: String::from("a $ b"),
                position: None
            })],
            position: None
        }))
        .unwrap(),
        "a \\$ b\n",
        "should escape `$` in phrasing"
    );

    assert_eq!(
        to_md_with_opts(
            &Node::Paragraph(Paragraph {
                children: vec![Node::Text(Text {
                    value: String::from("a $ b"),
                    position: None
                })],
                position: None
            }),
            &Options {
                single_dollar_text_math: false,
                ..Default::default()
            }
        )
        .unwrap(),
        "a $ b\n",
        "should not escape a single dollar in phrasing w/ `single_dollar_text_math: false`'"
    );

    assert_eq!(
        to_md_with_opts(
            &Node::Paragraph(Paragraph {
                children: vec![Node::Text(Text {
                    value: String::from("a $$ b"),
                    position: None
                })],
                position: None
            }),
            &Options {
                single_dollar_text_math: false,
                ..Default::default()
            }
        )
        .unwrap(),
        "a \\$$ b\n",
        "should escape two dollars in phrasing w/ `single_dollar_text_math: false`"
    );
}
