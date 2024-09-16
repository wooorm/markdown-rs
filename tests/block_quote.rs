use markdown::{
    mdast::{Blockquote, Node, Paragraph, Root, Text},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn block_quote() -> Result<(), message::Message> {
    assert_eq!(
        to_html("> # a\n> b\n> c"),
        "<blockquote>\n<h1>a</h1>\n<p>b\nc</p>\n</blockquote>",
        "should support block quotes"
    );

    assert_eq!(
        to_html("># a\n>b\n> c"),
        "<blockquote>\n<h1>a</h1>\n<p>b\nc</p>\n</blockquote>",
        "should support block quotes w/o space"
    );

    assert_eq!(
        to_html("   > # a\n   > b\n > c"),
        "<blockquote>\n<h1>a</h1>\n<p>b\nc</p>\n</blockquote>",
        "should support prefixing block quotes w/ spaces"
    );

    assert_eq!(
        to_html("    > # a\n    > b\n    > c"),
        "<pre><code>&gt; # a\n&gt; b\n&gt; c\n</code></pre>",
        "should not support block quotes w/ 4 spaces"
    );

    assert_eq!(
        to_html("> # a\n> b\nc"),
        "<blockquote>\n<h1>a</h1>\n<p>b\nc</p>\n</blockquote>",
        "should support lazy content lines"
    );

    assert_eq!(
        to_html("> a\nb\n> c"),
        "<blockquote>\n<p>a\nb\nc</p>\n</blockquote>",
        "should support lazy content lines inside block quotes"
    );

    assert_eq!(
        to_html("> a\n> ---"),
        "<blockquote>\n<h2>a</h2>\n</blockquote>",
        "should support setext headings underlines in block quotes"
    );

    assert_eq!(
        to_html("> a\n---"),
        "<blockquote>\n<p>a</p>\n</blockquote>\n<hr />",
        "should not support lazy setext headings underlines in block quotes"
    );

    assert_eq!(
        to_html("> - a\n> - b"),
        "<blockquote>\n<ul>\n<li>a</li>\n<li>b</li>\n</ul>\n</blockquote>",
        "should support lists in block quotes"
    );

    assert_eq!(
        to_html("> - a\n- b"),
        "<blockquote>\n<ul>\n<li>a</li>\n</ul>\n</blockquote>\n<ul>\n<li>b</li>\n</ul>",
        "should not support lazy lists in block quotes"
    );

    assert_eq!(
        to_html(">     a\n    b"),
        "<blockquote>\n<pre><code>a\n</code></pre>\n</blockquote>\n<pre><code>b\n</code></pre>",
        "should not support lazy indented code in block quotes"
    );

    assert_eq!(
        to_html("> ```\na\n```"),
        "<blockquote>\n<pre><code></code></pre>\n</blockquote>\n<p>a</p>\n<pre><code></code></pre>\n",
        "should not support lazy fenced code in block quotes (1)"
    );

    assert_eq!(
        to_html("> a\n```\nb"),
        "<blockquote>\n<p>a</p>\n</blockquote>\n<pre><code>b\n</code></pre>\n",
        "should not support lazy fenced code in block quotes (2)"
    );

    assert_eq!(
        to_html("> a\n    - b"),
        "<blockquote>\n<p>a\n- b</p>\n</blockquote>",
        "should not support lazy indented code (or lazy list) in block quotes"
    );

    assert_eq!(
        to_html("> [\na"),
        "<blockquote>\n<p>[\na</p>\n</blockquote>",
        "should support lazy, definition-like lines"
    );

    assert_eq!(
        to_html("> [a]: b\nc"),
        "<blockquote>\n<p>c</p>\n</blockquote>",
        "should support a definition, followed by a lazy paragraph"
    );

    assert_eq!(
        to_html(">"),
        "<blockquote>\n</blockquote>",
        "should support empty block quotes (1)"
    );

    assert_eq!(
        to_html(">\n>  \n> "),
        "<blockquote>\n</blockquote>",
        "should support empty block quotes (2)"
    );

    assert_eq!(
        to_html(">\n> a\n>  "),
        "<blockquote>\n<p>a</p>\n</blockquote>",
        "should support initial or final lazy empty block quote lines"
    );

    assert_eq!(
        to_html("> a\n\n> b"),
        "<blockquote>\n<p>a</p>\n</blockquote>\n<blockquote>\n<p>b</p>\n</blockquote>",
        "should support adjacent block quotes"
    );

    assert_eq!(
        to_html("> a\n> b"),
        "<blockquote>\n<p>a\nb</p>\n</blockquote>",
        "should support a paragraph in a block quote"
    );

    assert_eq!(
        to_html("> a\n>\n> b"),
        "<blockquote>\n<p>a</p>\n<p>b</p>\n</blockquote>",
        "should support adjacent paragraphs in block quotes"
    );

    assert_eq!(
        to_html("[a]\n\n> [a]: b"),
        "<p><a href=\"b\">a</a></p>\n<blockquote>\n</blockquote>",
        "should support a definition in a block quote (1)"
    );

    assert_eq!(
        to_html("> [a]: b\n\n[a]"),
        "<blockquote>\n</blockquote>\n<p><a href=\"b\">a</a></p>",
        "should support a definition in a block quote (2)"
    );

    assert_eq!(
        to_html("a\n> b"),
        "<p>a</p>\n<blockquote>\n<p>b</p>\n</blockquote>",
        "should support interrupting paragraphs w/ block quotes"
    );

    assert_eq!(
        to_html("> a\n***\n> b"),
        "<blockquote>\n<p>a</p>\n</blockquote>\n<hr />\n<blockquote>\n<p>b</p>\n</blockquote>",
        "should support interrupting block quotes w/ thematic breaks"
    );

    assert_eq!(
        to_html("> a\nb"),
        "<blockquote>\n<p>a\nb</p>\n</blockquote>",
        "should not support interrupting block quotes w/ paragraphs"
    );

    assert_eq!(
        to_html("> a\n\nb"),
        "<blockquote>\n<p>a</p>\n</blockquote>\n<p>b</p>",
        "should support interrupting block quotes w/ blank lines"
    );

    assert_eq!(
        to_html("> a\n>\nb"),
        "<blockquote>\n<p>a</p>\n</blockquote>\n<p>b</p>",
        "should not support interrupting a blank line in a block quotes w/ paragraphs"
    );

    assert_eq!(
        to_html("> > > a\nb"),
        "<blockquote>\n<blockquote>\n<blockquote>\n<p>a\nb</p>\n</blockquote>\n</blockquote>\n</blockquote>",
        "should not support interrupting many block quotes w/ paragraphs (1)"
    );

    assert_eq!(
        to_html(">>> a\n> b\n>>c"),
        "<blockquote>\n<blockquote>\n<blockquote>\n<p>a\nb\nc</p>\n</blockquote>\n</blockquote>\n</blockquote>",
        "should not support interrupting many block quotes w/ paragraphs (2)"
    );

    assert_eq!(
        to_html(">     a\n\n>    b"),
        "<blockquote>\n<pre><code>a\n</code></pre>\n</blockquote>\n<blockquote>\n<p>b</p>\n</blockquote>",
        "should support 5 spaces for indented code, not 4"
    );

    assert_eq!(
        to_html_with_options(
            "> # a\n> b\n> c",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        block_quote: false,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<p>&gt; # a\n&gt; b\n&gt; c</p>",
        "should support turning off block quotes"
    );

    assert_eq!(
        to_mdast("> a", &Default::default())?,
        Node::Root(Root {
            children: vec![Node::Blockquote(Blockquote {
                children: vec![Node::Paragraph(Paragraph {
                    children: vec![Node::Text(Text {
                        value: "a".into(),
                        position: Some(Position::new(1, 3, 2, 1, 4, 3))
                    }),],
                    position: Some(Position::new(1, 3, 2, 1, 4, 3))
                })],
                position: Some(Position::new(1, 1, 0, 1, 4, 3))
            })],
            position: Some(Position::new(1, 1, 0, 1, 4, 3))
        }),
        "should support block quotes as `BlockQuote`s in mdast"
    );

    Ok(())
}
