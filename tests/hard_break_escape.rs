use markdown::{
    mdast::{Break, Node, Paragraph, Root, Text},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn hard_break_escape() -> Result<(), message::Message> {
    assert_eq!(
        to_html("foo\\\nbaz"),
        "<p>foo<br />\nbaz</p>",
        "should support a backslash to form a hard break"
    );

    assert_eq!(
        to_html("foo\\\n     bar"),
        "<p>foo<br />\nbar</p>",
        "should support leading spaces after an escape hard break"
    );

    assert_eq!(
        to_html("*foo\\\nbar*"),
        "<p><em>foo<br />\nbar</em></p>",
        "should support escape hard breaks in emphasis"
    );

    assert_eq!(
        to_html("``code\\\ntext``"),
        "<p><code>code\\ text</code></p>",
        "should not support escape hard breaks in code"
    );

    assert_eq!(
        to_html("foo\\"),
        "<p>foo\\</p>",
        "should not support escape hard breaks at the end of a paragraph"
    );

    assert_eq!(
        to_html("### foo\\"),
        "<h3>foo\\</h3>",
        "should not support escape hard breaks at the end of a heading"
    );

    assert_eq!(
        to_html_with_options(
            "a\\\nb",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        hard_break_escape: false,
                        ..Constructs::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<p>a\\\nb</p>",
        "should support turning off hard break (escape)"
    );

    assert_eq!(
        to_mdast("a\\\nb.", &Default::default())?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: "a".into(),
                        position: Some(Position::new(1, 1, 0, 1, 2, 1))
                    }),
                    Node::Break(Break {
                        position: Some(Position::new(1, 2, 1, 2, 1, 3))
                    }),
                    Node::Text(Text {
                        value: "b.".into(),
                        position: Some(Position::new(2, 1, 3, 2, 3, 5))
                    }),
                ],
                position: Some(Position::new(1, 1, 0, 2, 3, 5))
            })],
            position: Some(Position::new(1, 1, 0, 2, 3, 5))
        }),
        "should support hard break (escape) as `Break`s in mdast"
    );

    Ok(())
}
