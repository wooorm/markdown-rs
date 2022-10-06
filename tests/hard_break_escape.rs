extern crate micromark;
use micromark::{
    mdast::{Break, Node, Paragraph, Root, Text},
    micromark, micromark_to_mdast, micromark_with_options,
    unist::Position,
    Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn hard_break_escape() -> Result<(), String> {
    assert_eq!(
        micromark("foo\\\nbaz"),
        "<p>foo<br />\nbaz</p>",
        "should support a backslash to form a hard break"
    );

    assert_eq!(
        micromark("foo\\\n     bar"),
        "<p>foo<br />\nbar</p>",
        "should support leading spaces after an escape hard break"
    );

    assert_eq!(
        micromark("*foo\\\nbar*"),
        "<p><em>foo<br />\nbar</em></p>",
        "should support escape hard breaks in emphasis"
    );

    assert_eq!(
        micromark("``code\\\ntext``"),
        "<p><code>code\\ text</code></p>",
        "should not support escape hard breaks in code"
    );

    assert_eq!(
        micromark("foo\\"),
        "<p>foo\\</p>",
        "should not support escape hard breaks at the end of a paragraph"
    );

    assert_eq!(
        micromark("### foo\\"),
        "<h3>foo\\</h3>",
        "should not support escape hard breaks at the end of a heading"
    );

    assert_eq!(
        micromark_with_options(
            "a\\\nb",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        hard_break_escape: false,
                        ..Constructs::default()
                    },
                    ..ParseOptions::default()
                },
                ..Options::default()
            }
        )?,
        "<p>a\\\nb</p>",
        "should support turning off hard break (escape)"
    );

    assert_eq!(
        micromark_to_mdast("a\\\nb.", &ParseOptions::default())?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: "a".to_string(),
                        position: Some(Position::new(1, 1, 0, 1, 2, 1))
                    }),
                    Node::Break(Break {
                        position: Some(Position::new(1, 2, 1, 2, 1, 3))
                    }),
                    Node::Text(Text {
                        value: "b.".to_string(),
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
