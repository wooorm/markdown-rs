extern crate micromark;
use micromark::{
    mdast::{InlineMath, Node, Paragraph, Position, Root, Text},
    micromark, micromark_to_mdast, micromark_with_options, Constructs, Options,
};
use pretty_assertions::assert_eq;

#[test]
fn math_text() -> Result<(), String> {
    let math = Options {
        constructs: Constructs {
            math_text: true,
            math_flow: true,
            ..Constructs::default()
        },
        ..Options::default()
    };

    assert_eq!(
        micromark("$a$"),
        "<p>$a$</p>",
        "should not support math (text) by default"
    );

    assert_eq!(
        micromark_with_options("$foo$ $$bar$$", &math)?,
        "<p><code class=\"language-math math-inline\">foo</code> <code class=\"language-math math-inline\">bar</code></p>",
        "should support math (text) if enabled"
    );

    assert_eq!(
        micromark_with_options(
            "$foo$ $$bar$$",
            &Options {
                math_text_single_dollar: false,
                constructs: Constructs {
                    math_text: true,
                    math_flow: true,
                    ..Constructs::default()
                },
                ..Options::default()
            }
        )?,
        "<p>$foo$ <code class=\"language-math math-inline\">bar</code></p>",
        "should not support math (text) w/ a single dollar, w/ `math_text_single_dollar: false`"
    );

    assert_eq!(
        micromark_with_options("$$ foo $ bar $$", &math)?,
        "<p><code class=\"language-math math-inline\">foo $ bar</code></p>",
        "should support math (text) w/ more dollars"
    );

    assert_eq!(
        micromark_with_options("$ $$ $", &math)?,
        "<p><code class=\"language-math math-inline\">$$</code></p>",
        "should support math (text) w/ fences inside, and padding"
    );

    assert_eq!(
        micromark_with_options("$  $$  $", &math)?,
        "<p><code class=\"language-math math-inline\"> $$ </code></p>",
        "should support math (text) w/ extra padding"
    );

    assert_eq!(
        micromark_with_options("$ a$", &math)?,
        "<p><code class=\"language-math math-inline\"> a</code></p>",
        "should support math (text) w/ unbalanced padding"
    );

    assert_eq!(
        micromark_with_options("$\u{a0}b\u{a0}$", &math)?,
        "<p><code class=\"language-math math-inline\">\u{a0}b\u{a0}</code></p>",
        "should support math (text) w/ non-padding whitespace"
    );

    assert_eq!(
        micromark_with_options("$ $\n$  $", &math)?,
        "<p><code class=\"language-math math-inline\"> </code>\n<code class=\"language-math math-inline\">  </code></p>",
        "should support math (text) w/o data"
    );

    assert_eq!(
        micromark_with_options("$\nfoo\nbar  \nbaz\n$", &math)?,
        "<p><code class=\"language-math math-inline\">foo bar   baz</code></p>",
        "should support math (text) w/o line endings (1)"
    );

    assert_eq!(
        micromark_with_options("$\nfoo \n$", &math)?,
        "<p><code class=\"language-math math-inline\">foo </code></p>",
        "should support math (text) w/o line endings (2)"
    );

    assert_eq!(
        micromark_with_options("$foo   bar \nbaz$", &math)?,
        "<p><code class=\"language-math math-inline\">foo   bar  baz</code></p>",
        "should not support whitespace collapsing"
    );

    assert_eq!(
        micromark_with_options("$foo\\$bar$", &math)?,
        "<p><code class=\"language-math math-inline\">foo\\</code>bar$</p>",
        "should not support character escapes"
    );

    assert_eq!(
        micromark_with_options("$$foo$bar$$", &math)?,
        "<p><code class=\"language-math math-inline\">foo$bar</code></p>",
        "should support more dollars"
    );

    assert_eq!(
        micromark_with_options("$ foo $$ bar $", &math)?,
        "<p><code class=\"language-math math-inline\">foo $$ bar</code></p>",
        "should support less dollars"
    );

    assert_eq!(
        micromark_with_options("*foo$*$", &math)?,
        "<p>*foo<code class=\"language-math math-inline\">*</code></p>",
        "should precede over emphasis"
    );

    assert_eq!(
        micromark_with_options("[not a $link](/foo$)", &math)?,
        "<p>[not a <code class=\"language-math math-inline\">link](/foo</code>)</p>",
        "should precede over links"
    );

    assert_eq!(
        micromark_with_options("$<a href=\"$\">$", &math)?,
        "<p><code class=\"language-math math-inline\">&lt;a href=&quot;</code>&quot;&gt;$</p>",
        "should have same precedence as HTML (1)"
    );

    assert_eq!(
        micromark_with_options(
            "<a href=\"$\">$",
            &Options {
                allow_dangerous_html: true,
                allow_dangerous_protocol: true,
                constructs: Constructs {
                    math_text: true,
                    math_flow: true,
                    ..Constructs::default()
                },
                ..Options::default()
            }
        )?,
        "<p><a href=\"$\">$</p>",
        "should have same precedence as HTML (2)"
    );

    assert_eq!(
        micromark_with_options("$<http://foo.bar.$baz>$", &math)?,
        "<p><code class=\"language-math math-inline\">&lt;http://foo.bar.</code>baz&gt;$</p>",
        "should have same precedence as autolinks (1)"
    );

    assert_eq!(
        micromark_with_options("<http://foo.bar.$baz>$", &math)?,
        "<p><a href=\"http://foo.bar.$baz\">http://foo.bar.$baz</a>$</p>",
        "should have same precedence as autolinks (2)"
    );

    assert_eq!(
        micromark_with_options("$$$foo$$", &math)?,
        "<p>$$$foo$$</p>",
        "should not support more dollars before a fence"
    );

    assert_eq!(
        micromark_with_options("$foo", &math)?,
        "<p>$foo</p>",
        "should not support no closing fence (1)"
    );

    assert_eq!(
        micromark_with_options("$foo$$bar$$", &math)?,
        "<p>$foo<code class=\"language-math math-inline\">bar</code></p>",
        "should not support no closing fence (2)"
    );

    assert_eq!(
        micromark_with_options("$foo\t\tbar$", &math)?,
        "<p><code class=\"language-math math-inline\">foo\t\tbar</code></p>",
        "should support tabs in code"
    );

    assert_eq!(
        micromark_with_options("\\$$x$", &math)?,
        "<p>$<code class=\"language-math math-inline\">x</code></p>",
        "should support an escaped initial dollar"
    );

    assert_eq!(
        micromark_to_mdast("a $alpha$ b.", &math)?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: "a ".to_string(),
                        position: Some(Position::new(1, 1, 0, 1, 3, 2))
                    }),
                    Node::InlineMath(InlineMath {
                        value: "alpha".to_string(),
                        position: Some(Position::new(1, 3, 2, 1, 10, 9))
                    }),
                    Node::Text(Text {
                        value: " b.".to_string(),
                        position: Some(Position::new(1, 10, 9, 1, 13, 12))
                    })
                ],
                position: Some(Position::new(1, 1, 0, 1, 13, 12))
            })],
            position: Some(Position::new(1, 1, 0, 1, 13, 12))
        }),
        "should support math (text) as `InlineMath`s in mdast"
    );

    Ok(())
}
