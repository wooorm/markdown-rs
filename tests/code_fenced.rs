use markdown::{
    mdast::{Code, Node, Root},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn code_fenced() -> Result<(), message::Message> {
    assert_eq!(
        to_html("```\n<\n >\n```"),
        "<pre><code>&lt;\n &gt;\n</code></pre>",
        "should support fenced code w/ grave accents"
    );

    assert_eq!(
        to_html("~~~\n<\n >\n~~~"),
        "<pre><code>&lt;\n &gt;\n</code></pre>",
        "should support fenced code w/ tildes"
    );

    assert_eq!(
        to_html("``\nfoo\n``"),
        "<p><code>foo</code></p>",
        "should not support fenced code w/ less than three markers"
    );

    assert_eq!(
        to_html("```\naaa\n~~~\n```"),
        "<pre><code>aaa\n~~~\n</code></pre>",
        "should not support a tilde closing sequence for a grave accent opening sequence"
    );

    assert_eq!(
        to_html("~~~\naaa\n```\n~~~"),
        "<pre><code>aaa\n```\n</code></pre>",
        "should not support a grave accent closing sequence for a tilde opening sequence"
    );

    assert_eq!(
        to_html("````\naaa\n```\n``````"),
        "<pre><code>aaa\n```\n</code></pre>",
        "should support a closing sequence longer, but not shorter than, the opening"
    );

    assert_eq!(
        to_html("~~~~\naaa\n~~~\n~~~~"),
        "<pre><code>aaa\n~~~\n</code></pre>",
        "should support a closing sequence equal to, but not shorter than, the opening"
    );

    assert_eq!(
        to_html("```"),
        "<pre><code></code></pre>\n",
        "should support an eof right after an opening sequence"
    );

    assert_eq!(
        to_html("`````\n\n```\naaa\n"),
        "<pre><code>\n```\naaa\n</code></pre>\n",
        "should support an eof somewhere in content"
    );

    assert_eq!(
        to_html("> ```\n> aaa\n\nbbb"),
        "<blockquote>\n<pre><code>aaa\n</code></pre>\n</blockquote>\n<p>bbb</p>",
        "should support no closing sequence in a block quote"
    );

    assert_eq!(
        to_html("```\n\n  \n```"),
        "<pre><code>\n  \n</code></pre>",
        "should support blank lines in fenced code"
    );

    assert_eq!(
        to_html("```\n```"),
        "<pre><code></code></pre>",
        "should support empty fenced code"
    );

    assert_eq!(
      to_html(" ```\n aaa\naaa\n```"),
      "<pre><code>aaa\naaa\n</code></pre>",
      "should remove up to one space from the content if the opening sequence is indented w/ 1 space"
    );

    assert_eq!(
      to_html("  ```\naaa\n  aaa\naaa\n  ```"),
      "<pre><code>aaa\naaa\naaa\n</code></pre>",
      "should remove up to two space from the content if the opening sequence is indented w/ 2 spaces"
    );

    assert_eq!(
      to_html("   ```\n   aaa\n    aaa\n  aaa\n   ```"),
      "<pre><code>aaa\n aaa\naaa\n</code></pre>",
      "should remove up to three space from the content if the opening sequence is indented w/ 3 spaces"
    );

    assert_eq!(
        to_html("    ```\n    aaa\n    ```"),
        "<pre><code>```\naaa\n```\n</code></pre>",
        "should not support indenteding the opening sequence w/ 4 spaces"
    );

    assert_eq!(
        to_html("```\naaa\n  ```"),
        "<pre><code>aaa\n</code></pre>",
        "should support an indented closing sequence"
    );

    assert_eq!(
        to_html("   ```\naaa\n  ```"),
        "<pre><code>aaa\n</code></pre>",
        "should support a differently indented closing sequence than the opening sequence"
    );

    assert_eq!(
        to_html("```\naaa\n    ```\n"),
        "<pre><code>aaa\n    ```\n</code></pre>\n",
        "should not support an indented closing sequence w/ 4 spaces"
    );

    assert_eq!(
        to_html("``` ```\naaa"),
        "<p><code> </code>\naaa</p>",
        "should not support grave accents in the opening fence after the opening sequence"
    );

    assert_eq!(
        to_html("~~~~~~\naaa\n~~~ ~~\n"),
        "<pre><code>aaa\n~~~ ~~\n</code></pre>\n",
        "should not support spaces in the closing sequence"
    );

    assert_eq!(
        to_html("foo\n```\nbar\n```\nbaz"),
        "<p>foo</p>\n<pre><code>bar\n</code></pre>\n<p>baz</p>",
        "should support interrupting paragraphs"
    );

    assert_eq!(
        to_html("foo\n---\n~~~\nbar\n~~~\n# baz"),
        "<h2>foo</h2>\n<pre><code>bar\n</code></pre>\n<h1>baz</h1>",
        "should support interrupting other content"
    );

    assert_eq!(
        to_html("```ruby\ndef foo(x)\n  return 3\nend\n```"),
        "<pre><code class=\"language-ruby\">def foo(x)\n  return 3\nend\n</code></pre>",
        "should support the info string as a `language-` class (1)"
    );

    assert_eq!(
        to_html("````;\n````"),
        "<pre><code class=\"language-;\"></code></pre>",
        "should support the info string as a `language-` class (2)"
    );

    assert_eq!(
        to_html("~~~~    ruby startline=3 $%@#$\ndef foo(x)\n  return 3\nend\n~~~~~~~"),
        "<pre><code class=\"language-ruby\">def foo(x)\n  return 3\nend\n</code></pre>",
        "should support the info string as a `language-` class, but not the meta string"
    );

    assert_eq!(
        to_html("``` aa ```\nfoo"),
        "<p><code>aa</code>\nfoo</p>",
        "should not support grave accents in the meta string"
    );

    assert_eq!(
        to_html("~~~ aa ``` ~~~\nfoo\n~~~"),
        "<pre><code class=\"language-aa\">foo\n</code></pre>",
        "should support grave accents and tildes in the meta string of tilde fenced code"
    );

    assert_eq!(
        to_html("```\n``` aaa\n```"),
        "<pre><code>``` aaa\n</code></pre>",
        "should not support info string on closing sequences"
    );

    // Our own:
    assert_eq!(
        to_html("```  "),
        "<pre><code></code></pre>\n",
        "should support an eof after whitespace, after the start fence sequence"
    );

    assert_eq!(
        to_html("```  js\nalert(1)\n```"),
        "<pre><code class=\"language-js\">alert(1)\n</code></pre>",
        "should support whitespace between the sequence and the info string"
    );

    assert_eq!(
        to_html("```js"),
        "<pre><code class=\"language-js\"></code></pre>\n",
        "should support an eof after the info string"
    );

    assert_eq!(
        to_html("```  js \nalert(1)\n```"),
        "<pre><code class=\"language-js\">alert(1)\n</code></pre>",
        "should support whitespace after the info string"
    );

    assert_eq!(
        to_html("```\n  "),
        "<pre><code>  \n</code></pre>\n",
        "should support an eof after whitespace in content"
    );

    assert_eq!(
        to_html("  ```\n "),
        "<pre><code></code></pre>\n",
        "should support an eof in the prefix, in content"
    );

    assert_eq!(
        to_html("```j\\+s&copy;"),
        "<pre><code class=\"language-j+s©\"></code></pre>\n",
        "should support character escapes and character references in info strings"
    );

    assert_eq!(
        to_html("```a\\&b\0c"),
        "<pre><code class=\"language-a&amp;b�c\"></code></pre>\n",
        "should encode dangerous characters in languages"
    );

    assert_eq!(
      to_html("   ```\naaa\n    ```"),
      "<pre><code>aaa\n ```\n</code></pre>\n",
      "should not support a closing sequence w/ too much indent, regardless of opening sequence (1)"
    );

    assert_eq!(
        to_html("> ```\n>\n>\n>\n\na"),
        "<blockquote>\n<pre><code>\n\n\n</code></pre>\n</blockquote>\n<p>a</p>",
        "should not support a closing sequence w/ too much indent, regardless of opening sequence (2)"
    );

    assert_eq!(
        to_html("> ```a\nb"),
        "<blockquote>\n<pre><code class=\"language-a\"></code></pre>\n</blockquote>\n<p>b</p>",
        "should not support lazyness (1)"
    );

    assert_eq!(
        to_html("> a\n```b"),
        "<blockquote>\n<p>a</p>\n</blockquote>\n<pre><code class=\"language-b\"></code></pre>\n",
        "should not support lazyness (2)"
    );

    assert_eq!(
        to_html("> ```a\n```"),
        "<blockquote>\n<pre><code class=\"language-a\"></code></pre>\n</blockquote>\n<pre><code></code></pre>\n",
        "should not support lazyness (3)"
    );

    assert_eq!(
        to_html_with_options(
            "```",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs {
                        code_fenced: false,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<p>```</p>",
        "should support turning off code (fenced)"
    );

    assert_eq!(
        to_mdast(
            "```js extra\nconsole.log(1)\nconsole.log(2)\n```",
            &Default::default()
        )?,
        Node::Root(Root {
            children: vec![Node::Code(Code {
                lang: Some("js".into()),
                meta: Some("extra".into()),
                value: "console.log(1)\nconsole.log(2)".into(),
                position: Some(Position::new(1, 1, 0, 4, 4, 45))
            })],
            position: Some(Position::new(1, 1, 0, 4, 4, 45))
        }),
        "should support code (fenced) as `Code`s in mdast"
    );

    assert_eq!(
        to_mdast("```\nasd", &Default::default())?,
        Node::Root(Root {
            children: vec![Node::Code(Code {
                lang: None,
                meta: None,
                value: "asd".into(),
                position: Some(Position::new(1, 1, 0, 2, 4, 7))
            })],
            position: Some(Position::new(1, 1, 0, 2, 4, 7))
        }),
        "should support code (fenced) w/o closing fence in mdast"
    );

    assert_eq!(
        to_mdast("```\rasd\r```", &Default::default())?,
        Node::Root(Root {
            children: vec![Node::Code(Code {
                lang: None,
                meta: None,
                value: "asd".into(),
                position: Some(Position::new(1, 1, 0, 3, 4, 11))
            })],
            position: Some(Position::new(1, 1, 0, 3, 4, 11))
        }),
        "should support code (fenced) w/o CR line endings"
    );

    assert_eq!(
        to_mdast("```\r\nasd\r\n```", &Default::default())?,
        Node::Root(Root {
            children: vec![Node::Code(Code {
                lang: None,
                meta: None,
                value: "asd".into(),
                position: Some(Position::new(1, 1, 0, 3, 4, 13))
            })],
            position: Some(Position::new(1, 1, 0, 3, 4, 13))
        }),
        "should support code (fenced) w/o CR+LF line endings"
    );

    Ok(())
}
