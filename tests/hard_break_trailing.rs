use markdown::{
    mdast::{Break, Node, Paragraph, Root, Text},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn hard_break_trailing() -> Result<(), message::Message> {
    assert_eq!(
        to_html("foo  \nbaz"),
        "<p>foo<br />\nbaz</p>",
        "should support two trailing spaces to form a hard break"
    );

    assert_eq!(
        to_html("foo       \nbaz"),
        "<p>foo<br />\nbaz</p>",
        "should support multiple trailing spaces"
    );

    assert_eq!(
        to_html("foo  \n     bar"),
        "<p>foo<br />\nbar</p>",
        "should support leading spaces after a trailing hard break"
    );

    assert_eq!(
        to_html("*foo  \nbar*"),
        "<p><em>foo<br />\nbar</em></p>",
        "should support trailing hard breaks in emphasis"
    );

    assert_eq!(
        to_html("`code  \ntext`"),
        "<p><code>code   text</code></p>",
        "should not support trailing hard breaks in code"
    );

    assert_eq!(
        to_html("foo  "),
        "<p>foo</p>",
        "should not support trailing hard breaks at the end of a paragraph"
    );

    assert_eq!(
        to_html("### foo  "),
        "<h3>foo</h3>",
        "should not support trailing hard breaks at the end of a heading"
    );

    assert_eq!(
        to_html("aaa  \t\nbb"),
        "<p>aaa\nbb</p>",
        "should support a mixed line suffix (1)"
    );

    assert_eq!(
        to_html("aaa\t  \nbb"),
        "<p>aaa\nbb</p>",
        "should support a mixed line suffix (2)"
    );

    assert_eq!(
        to_html("aaa  \t  \nbb"),
        "<p>aaa\nbb</p>",
        "should support a mixed line suffix (3)"
    );

    assert_eq!(
        to_html("aaa\0  \nbb"),
        "<p>aaa�<br />\nbb</p>",
        "should support a hard break after a replacement character"
    );

    assert_eq!(
        to_html("aaa\0\t\nbb"),
        "<p>aaa�\nbb</p>",
        "should support a line suffix after a replacement character"
    );

    assert_eq!(
        to_html("*a*  \nbb"),
        "<p><em>a</em><br />\nbb</p>",
        "should support a hard break after a span"
    );

    assert_eq!(
        to_html("*a*\t\nbb"),
        "<p><em>a</em>\nbb</p>",
        "should support a line suffix after a span"
    );

    assert_eq!(
        to_html("*a*  \t\nbb"),
        "<p><em>a</em>\nbb</p>",
        "should support a mixed line suffix after a span (1)"
    );

    assert_eq!(
        to_html("*a*\t  \nbb"),
        "<p><em>a</em>\nbb</p>",
        "should support a mixed line suffix after a span (2)"
    );

    assert_eq!(
        to_html("*a*  \t  \nbb"),
        "<p><em>a</em>\nbb</p>",
        "should support a mixed line suffix after a span (3)"
    );

    assert_eq!(
        to_html_with_options(
            "a  \nb",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        hard_break_trailing: false,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<p>a\nb</p>",
        "should support turning off hard break (trailing)"
    );

    assert_eq!(
        to_mdast("a  \nb.", &Default::default())?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: "a".into(),
                        position: Some(Position::new(1, 1, 0, 1, 2, 1))
                    }),
                    Node::Break(Break {
                        position: Some(Position::new(1, 2, 1, 2, 1, 4))
                    }),
                    Node::Text(Text {
                        value: "b.".into(),
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
