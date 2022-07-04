extern crate micromark;
use micromark::micromark;

#[test]
fn hard_break_trailing() {
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
        micromark("*foo\\\nbar*"),
        "<p><em>foo<br />\nbar</em></p>",
        "should support escape hard breaks in emphasis"
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

    // // To do: turning things off.
    // assert_eq!(
    //   micromark("a  \nb", {extensions: [{disable: {null: ["hardBreakTrailing"]}}]}),
    //   "<p>a\nb</p>",
    //   "should support turning off hard break (trailing)"
    // );
}
