use markdown::{
    mdast::{Math, Node, Root},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn math_flow() -> Result<(), message::Message> {
    let math = Options {
        parse: ParseOptions {
            constructs: Constructs {
                math_text: true,
                math_flow: true,
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        to_html("$$\na\n$$"),
        "<p>$$\na\n$$</p>",
        "should not support math (flow) by default"
    );

    assert_eq!(
        to_html_with_options("$$\na\n$$", &math)?,
        "<pre><code class=\"language-math math-display\">a\n</code></pre>",
        "should support math (flow) if enabled"
    );

    assert_eq!(
        to_html_with_options("$$\n<\n >\n$$", &math)?,
        "<pre><code class=\"language-math math-display\">&lt;\n &gt;\n</code></pre>",
        "should support math (flow)"
    );

    assert_eq!(
        to_html_with_options("$\nfoo\n$", &math)?,
        "<p><code class=\"language-math math-inline\">foo</code></p>",
        "should not support math (flow) w/ less than two markers"
    );

    assert_eq!(
        to_html_with_options("$$$\naaa\n$$\n$$$$", &math)?,
        "<pre><code class=\"language-math math-display\">aaa\n$$\n</code></pre>",
        "should support a closing sequence longer, but not shorter than, the opening"
    );

    assert_eq!(
        to_html_with_options("$$", &math)?,
        "<pre><code class=\"language-math math-display\"></code></pre>\n",
        "should support an eof right after an opening sequence"
    );

    assert_eq!(
        to_html_with_options("$$$\n\n$$\naaa\n", &math)?,
        "<pre><code class=\"language-math math-display\">\n$$\naaa\n</code></pre>\n",
        "should support an eof somewhere in content"
    );

    assert_eq!(
        to_html_with_options("> $$\n> aaa\n\nbbb", &math)?,
        "<blockquote>\n<pre><code class=\"language-math math-display\">aaa\n</code></pre>\n</blockquote>\n<p>bbb</p>",
        "should support no closing sequence in a block quote"
    );

    assert_eq!(
        to_html_with_options("$$\n\n  \n$$", &math)?,
        "<pre><code class=\"language-math math-display\">\n  \n</code></pre>",
        "should support blank lines in math (flow)"
    );

    assert_eq!(
        to_html_with_options("$$\n$$", &math)?,
        "<pre><code class=\"language-math math-display\"></code></pre>",
        "should support empty math (flow)"
    );

    assert_eq!(
      to_html_with_options(" $$\n aaa\naaa\n$$", &math)?,
      "<pre><code class=\"language-math math-display\">aaa\naaa\n</code></pre>",
      "should remove up to one space from the content if the opening sequence is indented w/ 1 space"
    );

    assert_eq!(
      to_html_with_options("  $$\naaa\n  aaa\naaa\n  $$", &math)?,
      "<pre><code class=\"language-math math-display\">aaa\naaa\naaa\n</code></pre>",
      "should remove up to two space from the content if the opening sequence is indented w/ 2 spaces"
    );

    assert_eq!(
      to_html_with_options("   $$\n   aaa\n    aaa\n  aaa\n   $$", &math)?,
      "<pre><code class=\"language-math math-display\">aaa\n aaa\naaa\n</code></pre>",
      "should remove up to three space from the content if the opening sequence is indented w/ 3 spaces"
    );

    assert_eq!(
        to_html_with_options("    $$\n    aaa\n    $$", &math)?,
        "<pre><code>$$\naaa\n$$\n</code></pre>",
        "should not support indenteding the opening sequence w/ 4 spaces"
    );

    assert_eq!(
        to_html_with_options("$$\naaa\n  $$", &math)?,
        "<pre><code class=\"language-math math-display\">aaa\n</code></pre>",
        "should support an indented closing sequence"
    );

    assert_eq!(
        to_html_with_options("   $$\naaa\n  $$", &math)?,
        "<pre><code class=\"language-math math-display\">aaa\n</code></pre>",
        "should support a differently indented closing sequence than the opening sequence"
    );

    assert_eq!(
        to_html_with_options("$$\naaa\n    $$\n", &math)?,
        "<pre><code class=\"language-math math-display\">aaa\n    $$\n</code></pre>\n",
        "should not support an indented closing sequence w/ 4 spaces"
    );

    assert_eq!(
        to_html_with_options("$$ $$\naaa", &math)?,
        "<p><code class=\"language-math math-inline\"> </code>\naaa</p>",
        "should not support dollars in the opening fence after the opening sequence"
    );

    assert_eq!(
        to_html_with_options("$$$\naaa\n$$$ $$\n", &math)?,
        "<pre><code class=\"language-math math-display\">aaa\n$$$ $$\n</code></pre>\n",
        "should not support spaces in the closing sequence"
    );

    assert_eq!(
        to_html_with_options("foo\n$$\nbar\n$$\nbaz", &math)?,
        "<p>foo</p>\n<pre><code class=\"language-math math-display\">bar\n</code></pre>\n<p>baz</p>",
        "should support interrupting paragraphs"
    );

    assert_eq!(
        to_html_with_options("foo\n---\n$$\nbar\n$$\n# baz", &math)?,
        "<h2>foo</h2>\n<pre><code class=\"language-math math-display\">bar\n</code></pre>\n<h1>baz</h1>",
        "should support interrupting other content"
    );

    assert_eq!(
        to_html_with_options("$$ruby\ndef foo(x)\n  return 3\nend\n$$", &math)?,
        "<pre><code class=\"language-math math-display\">def foo(x)\n  return 3\nend\n</code></pre>",
        "should not support an “info” string (1)"
    );

    assert_eq!(
        to_html_with_options("$$$;\n$$$", &math)?,
        "<pre><code class=\"language-math math-display\"></code></pre>",
        "should not support an “info” string (2)"
    );

    assert_eq!(
        to_html_with_options("$$    ruby startline=3 `%@#`\ndef foo(x)\n  return 3\nend\n$$$$", &math)?,
        "<pre><code class=\"language-math math-display\">def foo(x)\n  return 3\nend\n</code></pre>",
        "should not support an “info” string (3)"
    );

    assert_eq!(
        to_html_with_options("$$ aa $$\nfoo", &math)?,
        "<p><code class=\"language-math math-inline\">aa</code>\nfoo</p>",
        "should not support dollars in the meta string"
    );

    assert_eq!(
        to_html_with_options("$$\n$$ aaa\n$$", &math)?,
        "<pre><code class=\"language-math math-display\">$$ aaa\n</code></pre>",
        "should not support meta string on closing sequences"
    );

    // Our own:
    assert_eq!(
        to_html_with_options("$$  ", &math)?,
        "<pre><code class=\"language-math math-display\"></code></pre>\n",
        "should support an eof after whitespace, after the start fence sequence"
    );

    assert_eq!(
        to_html_with_options("$$  js\nalert(1)\n$$", &math)?,
        "<pre><code class=\"language-math math-display\">alert(1)\n</code></pre>",
        "should support whitespace between the sequence and the meta string"
    );

    assert_eq!(
        to_html_with_options("$$js", &math)?,
        "<pre><code class=\"language-math math-display\"></code></pre>\n",
        "should support an eof after the meta string"
    );

    assert_eq!(
        to_html_with_options("$$  js \nalert(1)\n$$", &math)?,
        "<pre><code class=\"language-math math-display\">alert(1)\n</code></pre>",
        "should support whitespace after the meta string"
    );

    assert_eq!(
        to_html_with_options("$$\n  ", &math)?,
        "<pre><code class=\"language-math math-display\">  \n</code></pre>\n",
        "should support an eof after whitespace in content"
    );

    assert_eq!(
        to_html_with_options("  $$\n ", &math)?,
        "<pre><code class=\"language-math math-display\"></code></pre>\n",
        "should support an eof in the prefix, in content"
    );

    assert_eq!(
        to_html_with_options("$$j\\+s&copy;", &math)?,
        "<pre><code class=\"language-math math-display\"></code></pre>\n",
        "should support character escapes and character references in meta strings"
    );

    assert_eq!(
        to_html_with_options("$$a\\&b\0c", &math)?,
        "<pre><code class=\"language-math math-display\"></code></pre>\n",
        "should support dangerous characters in meta strings"
    );

    assert_eq!(
      to_html_with_options("   $$\naaa\n    $$", &math)?,
      "<pre><code class=\"language-math math-display\">aaa\n $$\n</code></pre>\n",
      "should not support a closing sequence w/ too much indent, regardless of opening sequence (1)"
    );

    assert_eq!(
        to_html_with_options("> $$\n>\n>\n>\n\na", &math)?,
        "<blockquote>\n<pre><code class=\"language-math math-display\">\n\n\n</code></pre>\n</blockquote>\n<p>a</p>",
        "should not support a closing sequence w/ too much indent, regardless of opening sequence (2)"
    );

    assert_eq!(
        to_html_with_options("> $$a\nb", &math)?,
        "<blockquote>\n<pre><code class=\"language-math math-display\"></code></pre>\n</blockquote>\n<p>b</p>",
        "should not support lazyness (1)"
    );

    assert_eq!(
        to_html_with_options("> a\n$$b", &math)?,
        "<blockquote>\n<p>a</p>\n</blockquote>\n<pre><code class=\"language-math math-display\"></code></pre>\n",
        "should not support lazyness (2)"
    );

    assert_eq!(
        to_html_with_options("> $$a\n$$", &math)?,
        "<blockquote>\n<pre><code class=\"language-math math-display\"></code></pre>\n</blockquote>\n<pre><code class=\"language-math math-display\"></code></pre>\n",
        "should not support lazyness (3)"
    );

    assert_eq!(
        to_mdast("$$extra\nabc\ndef\n$$", &math.parse)?,
        Node::Root(Root {
            children: vec![Node::Math(Math {
                meta: Some("extra".into()),
                value: "abc\ndef".into(),
                position: Some(Position::new(1, 1, 0, 4, 3, 18))
            })],
            position: Some(Position::new(1, 1, 0, 4, 3, 18))
        }),
        "should support math (flow) as `Math`s in mdast"
    );

    Ok(())
}
