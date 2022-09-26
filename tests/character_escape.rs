extern crate micromark;
use micromark::{
    mdast::{Node, Paragraph, Position, Root, Text},
    micromark, micromark_to_mdast, micromark_with_options, Constructs, Options,
};
use pretty_assertions::assert_eq;

#[test]
fn character_escape() -> Result<(), String> {
    let danger = Options {
        allow_dangerous_html: true,
        allow_dangerous_protocol: true,
        ..Options::default()
    };

    assert_eq!(
        micromark(
            "\\!\\\"\\#\\$\\%\\&\\'\\(\\)\\*\\+\\,\\-\\.\\/\\:\\;\\<\\=\\>\\?\\@\\[\\\\\\]\\^\\_\\`\\{\\|\\}\\~"),
        "<p>!&quot;#$%&amp;'()*+,-./:;&lt;=&gt;?@[\\]^_`{|}~</p>",
        "should support escaped ascii punctuation"
    );

    assert_eq!(
        micromark("\\→\\A\\a\\ \\3\\φ\\«"),
        "<p>\\→\\A\\a\\ \\3\\φ\\«</p>",
        "should not support other characters after a backslash"
    );

    assert_eq!(
        micromark(
            "\\*not emphasized*\n\\<br/> not a tag\n\\[not a link](/foo)\n\\`not code`\n1\\. not a list\n\\* not a list\n\\# not a heading\n\\[foo]: /url \"not a reference\"\n\\&ouml; not a character entity"),
        "<p>*not emphasized*\n&lt;br/&gt; not a tag\n[not a link](/foo)\n`not code`\n1. not a list\n* not a list\n# not a heading\n[foo]: /url &quot;not a reference&quot;\n&amp;ouml; not a character entity</p>",
        "should escape other constructs"
    );

    assert_eq!(
        micromark("foo\\\nbar"),
        "<p>foo<br />\nbar</p>",
        "should escape a line break"
    );

    assert_eq!(
        micromark("`` \\[\\` ``"),
        "<p><code>\\[\\`</code></p>",
        "should not escape in text code"
    );

    assert_eq!(
        micromark("    \\[\\]"),
        "<pre><code>\\[\\]\n</code></pre>",
        "should not escape in indented code"
    );

    assert_eq!(
        micromark("<http://example.com?find=\\*>"),
        "<p><a href=\"http://example.com?find=%5C*\">http://example.com?find=\\*</a></p>",
        "should not escape in autolink"
    );

    assert_eq!(
        micromark_with_options("<a href=\"/bar\\/)\">", &danger)?,
        "<a href=\"/bar\\/)\">",
        "should not escape in flow html"
    );

    assert_eq!(
        micromark("[foo](/bar\\* \"ti\\*tle\")"),
        "<p><a href=\"/bar*\" title=\"ti*tle\">foo</a></p>",
        "should escape in resource and title"
    );

    assert_eq!(
        micromark("[foo]: /bar\\* \"ti\\*tle\"\n\n[foo]"),
        "<p><a href=\"/bar*\" title=\"ti*tle\">foo</a></p>",
        "should escape in definition resource and title"
    );

    assert_eq!(
        micromark("``` foo\\+bar\nfoo\n```"),
        "<pre><code class=\"language-foo+bar\">foo\n</code></pre>",
        "should escape in fenced code info"
    );

    assert_eq!(
        micromark_with_options(
            "\\> a",
            &Options {
                constructs: Constructs {
                    character_escape: false,
                    ..Constructs::default()
                },
                ..Options::default()
            }
        )?,
        "<p>\\&gt; a</p>",
        "should support turning off character escapes"
    );

    assert_eq!(
        micromark_to_mdast("a \\* b", &Options::default())?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![Node::Text(Text {
                    value: "a * b".to_string(),
                    position: Some(Position::new(1, 1, 0, 1, 7, 6))
                }),],
                position: Some(Position::new(1, 1, 0, 1, 7, 6))
            })],
            position: Some(Position::new(1, 1, 0, 1, 7, 6))
        }),
        "should support character escapes as `Text`s in mdast"
    );

    Ok(())
}
