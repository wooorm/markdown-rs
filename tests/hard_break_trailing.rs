extern crate micromark;
use micromark::{
    mdast::{Break, Node, Paragraph, Root, Text},
    micromark, micromark_to_mdast, micromark_with_options,
    unist::Position,
    Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn hard_break_trailing() -> Result<(), String> {
    assert_eq!(
        micromark("foo  \nbaz"),
        "<p>foo<br />\nbaz</p>",
        "should support two trailing spaces to form a hard break"
    );

    assert_eq!(
        micromark("foo       \nbaz"),
        "<p>foo<br />\nbaz</p>",
        "should support multiple trailing spaces"
    );

    assert_eq!(
        micromark("foo  \n     bar"),
        "<p>foo<br />\nbar</p>",
        "should support leading spaces after a trailing hard break"
    );

    assert_eq!(
        micromark("*foo  \nbar*"),
        "<p><em>foo<br />\nbar</em></p>",
        "should support trailing hard breaks in emphasis"
    );

    assert_eq!(
        micromark("`code  \ntext`"),
        "<p><code>code   text</code></p>",
        "should not support trailing hard breaks in code"
    );

    assert_eq!(
        micromark("foo  "),
        "<p>foo</p>",
        "should not support trailing hard breaks at the end of a paragraph"
    );

    assert_eq!(
        micromark("### foo  "),
        "<h3>foo</h3>",
        "should not support trailing hard breaks at the end of a heading"
    );

    assert_eq!(
        micromark("aaa  \t\nbb"),
        "<p>aaa\nbb</p>",
        "should support a mixed line suffix (1)"
    );

    assert_eq!(
        micromark("aaa\t  \nbb"),
        "<p>aaa\nbb</p>",
        "should support a mixed line suffix (2)"
    );

    assert_eq!(
        micromark("aaa  \t  \nbb"),
        "<p>aaa\nbb</p>",
        "should support a mixed line suffix (3)"
    );

    assert_eq!(
        micromark("aaa\0  \nbb"),
        "<p>aaa�<br />\nbb</p>",
        "should support a hard break after a replacement character"
    );

    assert_eq!(
        micromark("aaa\0\t\nbb"),
        "<p>aaa�\nbb</p>",
        "should support a line suffix after a replacement character"
    );

    assert_eq!(
        micromark("*a*  \nbb"),
        "<p><em>a</em><br />\nbb</p>",
        "should support a hard break after a span"
    );

    assert_eq!(
        micromark("*a*\t\nbb"),
        "<p><em>a</em>\nbb</p>",
        "should support a line suffix after a span"
    );

    assert_eq!(
        micromark("*a*  \t\nbb"),
        "<p><em>a</em>\nbb</p>",
        "should support a mixed line suffix after a span (1)"
    );

    assert_eq!(
        micromark("*a*\t  \nbb"),
        "<p><em>a</em>\nbb</p>",
        "should support a mixed line suffix after a span (2)"
    );

    assert_eq!(
        micromark("*a*  \t  \nbb"),
        "<p><em>a</em>\nbb</p>",
        "should support a mixed line suffix after a span (3)"
    );

    assert_eq!(
        micromark_with_options(
            "a  \nb",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        hard_break_trailing: false,
                        ..Constructs::default()
                    },
                    ..ParseOptions::default()
                },
                ..Options::default()
            }
        )?,
        "<p>a\nb</p>",
        "should support turning off hard break (trailing)"
    );

    assert_eq!(
        micromark_to_mdast("a  \nb.", &ParseOptions::default())?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: "a".to_string(),
                        position: Some(Position::new(1, 1, 0, 1, 2, 1))
                    }),
                    Node::Break(Break {
                        position: Some(Position::new(1, 2, 1, 2, 1, 4))
                    }),
                    Node::Text(Text {
                        value: "b.".to_string(),
                        position: Some(Position::new(2, 1, 4, 2, 3, 6))
                    }),
                ],
                position: Some(Position::new(1, 1, 0, 2, 3, 6))
            })],
            position: Some(Position::new(1, 1, 0, 2, 3, 6))
        }),
        "should support hard break (trailing) as `Break`s in mdast"
    );

    Ok(())
}
