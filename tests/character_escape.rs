extern crate micromark;
use micromark::{micromark, micromark_with_options, CompileOptions};

const DANGER: &CompileOptions = &CompileOptions {
    allow_dangerous_html: true,
    allow_dangerous_protocol: true,
};

#[test]
fn character_escape() {
    assert_eq!(
        micromark(
        "\\!\\\"\\#\\$\\%\\&\\'\\(\\)\\*\\+\\,\\-\\.\\/\\:\\;\\<\\=\\>\\?\\@\\[\\\\\\]\\^\\_\\`\\{\\|\\}\\~"
        ),
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
            "\\*not emphasized*\n\\<br/> not a tag\n\\[not a link](/foo)\n\\`not code`\n1\\. not a list\n\\* not a list\n\\# not a heading\n\\[foo]: /url \"not a reference\"\n\\&ouml; not a character entity"
        ),
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
        micromark_with_options("<a href=\"/bar\\/)\">", DANGER),
        "<a href=\"/bar\\/)\">",
        "should not escape in flow html"
    );

    // To do: link (reference).
    // assert_eq!(
    //     micromark("[foo](/bar\\* \"ti\\*tle\")"),
    //     "<p><a href=\"/bar*\" title=\"ti*tle\">foo</a></p>",
    //     "should escape in resource and title"
    // );

    // To do: link (reference).
    // assert_eq!(
    //     micromark("[foo]: /bar\\* \"ti\\*tle\"\n\n[foo]"),
    //     "<p><a href=\"/bar*\" title=\"ti*tle\">foo</a></p>",
    //     "should escape in definition resource and title"
    // );

    assert_eq!(
        micromark("``` foo\\+bar\nfoo\n```"),
        "<pre><code class=\"language-foo+bar\">foo\n</code></pre>",
        "should escape in fenced code info"
    );

    //   // To do: extensions
    //   assert_eq!(
    //     micromark("\\> a", {extensions: [{disable: {null: ["characterEscape"]}}]}),
    //     "<p>\\&gt; a</p>",
    //     "should support turning off character escapes"
    //   );
}
