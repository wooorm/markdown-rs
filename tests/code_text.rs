use markdown::{
    mdast::{InlineCode, Node, Paragraph, Root, Text},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    CompileOptions, Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn code_text() -> Result<(), message::Message> {
    let danger = Options {
        compile: CompileOptions {
            allow_dangerous_html: true,
            allow_dangerous_protocol: true,
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        to_html("`foo`"),
        "<p><code>foo</code></p>",
        "should support code"
    );

    assert_eq!(
        to_html("`` foo ` bar ``"),
        "<p><code>foo ` bar</code></p>",
        "should support code w/ more accents"
    );

    assert_eq!(
        to_html("` `` `"),
        "<p><code>``</code></p>",
        "should support code w/ fences inside, and padding"
    );

    assert_eq!(
        to_html("`  ``  `"),
        "<p><code> `` </code></p>",
        "should support code w/ extra padding"
    );

    assert_eq!(
        to_html("` a`"),
        "<p><code> a</code></p>",
        "should support code w/ unbalanced padding"
    );

    assert_eq!(
        to_html("`\u{a0}b\u{a0}`"),
        "<p><code>\u{a0}b\u{a0}</code></p>",
        "should support code w/ non-padding whitespace"
    );

    assert_eq!(
        to_html("` `\n`  `"),
        "<p><code> </code>\n<code>  </code></p>",
        "should support code w/o data"
    );

    assert_eq!(
        to_html("``\nfoo\nbar  \nbaz\n``"),
        "<p><code>foo bar   baz</code></p>",
        "should support code w/o line endings (1)"
    );

    assert_eq!(
        to_html("``\nfoo \n``"),
        "<p><code>foo </code></p>",
        "should support code w/o line endings (2)"
    );

    assert_eq!(
        to_html("`foo   bar \nbaz`"),
        "<p><code>foo   bar  baz</code></p>",
        "should not support whitespace collapsing"
    );

    assert_eq!(
        to_html("`foo\\`bar`"),
        "<p><code>foo\\</code>bar`</p>",
        "should not support character escapes"
    );

    assert_eq!(
        to_html("``foo`bar``"),
        "<p><code>foo`bar</code></p>",
        "should support more accents"
    );

    assert_eq!(
        to_html("` foo `` bar `"),
        "<p><code>foo `` bar</code></p>",
        "should support less accents"
    );

    assert_eq!(
        to_html("*foo`*`"),
        "<p>*foo<code>*</code></p>",
        "should precede over emphasis"
    );

    assert_eq!(
        to_html("[not a `link](/foo`)"),
        "<p>[not a <code>link](/foo</code>)</p>",
        "should precede over links"
    );

    assert_eq!(
        to_html("`<a href=\"`\">`"),
        "<p><code>&lt;a href=&quot;</code>&quot;&gt;`</p>",
        "should have same precedence as HTML (1)"
    );

    assert_eq!(
        to_html_with_options("<a href=\"`\">`", &danger)?,
        "<p><a href=\"`\">`</p>",
        "should have same precedence as HTML (2)"
    );

    assert_eq!(
        to_html("`<http://foo.bar.`baz>`"),
        "<p><code>&lt;http://foo.bar.</code>baz&gt;`</p>",
        "should have same precedence as autolinks (1)"
    );

    assert_eq!(
        to_html("<http://foo.bar.`baz>`"),
        "<p><a href=\"http://foo.bar.%60baz\">http://foo.bar.`baz</a>`</p>",
        "should have same precedence as autolinks (2)"
    );

    assert_eq!(
        to_html("```foo``"),
        "<p>```foo``</p>",
        "should not support more accents before a fence"
    );

    assert_eq!(
        to_html("`foo"),
        "<p>`foo</p>",
        "should not support no closing fence (1)"
    );

    assert_eq!(
        to_html("`foo``bar``"),
        "<p>`foo<code>bar</code></p>",
        "should not support no closing fence (2)"
    );

    // Extra:
    assert_eq!(
        to_html("`foo\t\tbar`"),
        "<p><code>foo\t\tbar</code></p>",
        "should support tabs in code"
    );

    assert_eq!(
        to_html("\\``x`"),
        "<p>`<code>x</code></p>",
        "should support an escaped initial grave accent"
    );

    assert_eq!(
        to_html_with_options(
            "`a`",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        code_text: false,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<p>`a`</p>",
        "should support turning off code (text)"
    );

    assert_eq!(
        to_mdast("a `alpha` b.", &Default::default())?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: "a ".into(),
                        position: Some(Position::new(1, 1, 0, 1, 3, 2))
                    }),
                    Node::InlineCode(InlineCode {
                        value: "alpha".into(),
                        position: Some(Position::new(1, 3, 2, 1, 10, 9))
                    }),
                    Node::Text(Text {
                        value: " b.".into(),
                        position: Some(Position::new(1, 10, 9, 1, 13, 12))
                    })
                ],
                position: Some(Position::new(1, 1, 0, 1, 13, 12))
            })],
            position: Some(Position::new(1, 1, 0, 1, 13, 12))
        }),
        "should support code (text) as `InlineCode`s in mdast"
    );

    assert_eq!(
        to_mdast("`  alpha `", &Default::default())?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![Node::InlineCode(InlineCode {
                    value: " alpha".into(),
                    position: Some(Position::new(1, 1, 0, 1, 11, 10))
                }),],
                position: Some(Position::new(1, 1, 0, 1, 11, 10))
            })],
            position: Some(Position::new(1, 1, 0, 1, 11, 10))
        }),
        "should strip one space from each side of `InlineCode` if the value starts and ends with space"
    );

    assert_eq!(
        to_mdast("`   `", &Default::default())?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![Node::InlineCode(InlineCode {
                    value: "   ".into(),
                    position: Some(Position::new(1, 1, 0, 1, 6, 5))
                }),],
                position: Some(Position::new(1, 1, 0, 1, 6, 5))
            })],
            position: Some(Position::new(1, 1, 0, 1, 6, 5))
        }),
        "should not strip any whitespace if `InlineCode` is all whitespace"
    );

    Ok(())
}
